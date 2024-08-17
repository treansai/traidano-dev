use crate::base::AppState;
use crate::bot::{BotConfig, MarketType};
use crate::core::functions::calculate_position_size;
use crate::handlers::account::get_account;
use crate::handlers::bar::get_bars;
use crate::handlers::market::{get_positions, is_market_open};
use crate::handlers::order::create_order;
use crate::models::order::Order;
use crate::models::trade::{Side, TimeInForce, Type};
use axum::extract::State;
use axum::Json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use crate::bot::strategies::should_execute;

/// This is a mean reversion bot for both crypto and equity markets
pub async fn mean_reversion_strategy(state: Arc<AppState>, config: BotConfig) {
    // checking interval in sec (every 1min)
    let mut interval = interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        let should_execute = match should_execute(&state, &config).await {
            Some(value) => value,
            None => continue,
        };

        if should_execute {
            let mut all_signals = HashMap::new();

            for timeframe in &config.timeframes {
                match get_bars(
                    state.as_ref(),
                    &config.symbols,
                    timeframe,
                    config.lookback.max(config.volatility_window),
                )
                    .await
                {
                    Ok(all_bars) => {
                        for (symbol, bars) in all_bars {
                            if bars.len() < config.lookback.max(config.volatility_window) {
                                tracing::warn!(
                                    "Not enough data for {} on timeframe {}",
                                    symbol,
                                    timeframe
                                );
                                continue;
                            }

                            let prices: Vec<f64> = bars.iter().map(|bar| bar.close_price).collect();
                            let mean = prices.iter().take(config.lookback).sum::<f64>()
                                / config.lookback as f64;
                            let last_price = *prices.last().unwrap();

                            let deviation = (last_price - mean).abs() / mean;
                            let signal = if last_price > mean { -1 } else { 1 };
                            all_signals
                                .entry(symbol.clone())
                                .and_modify(|e: &mut i32| *e += signal)
                                .or_insert(signal);
                        }
                    }
                    Err(e) => tracing::error!(
                        "Failed to get bars for timeframe {}: {:?}",
                        timeframe,
                        e
                    ),
                }
            }

            // get account
            let account = match get_account(&state).await {
                Ok(acc) => acc,
                Err(e) => {
                    tracing::error!("Failed to get account information: {:?}", e);
                    continue;
                }
            };

            // get positions
            let positions = match get_positions(state.as_ref()).await {
                Ok(pos) => pos,
                Err(e) => {
                    tracing::error!("Failed to get positions: {:?}", e);
                    continue;
                }
            };

            let current_positions: HashMap<String, f64> =
                positions.into_iter().map(|p| (p.symbol, p.qty)).collect();

            for (symbol, signal) in all_signals {
                if signal.abs() == config.timeframes.len() as i32 {
                    let side = if signal > 0 { Side::Buy } else { Side::Sell };
                    let current_position = *current_positions.get(&symbol).unwrap_or(&0.0);

                    if (side == Side::Buy && current_position <= 0.0)
                        || (side == Side::Sell && current_position >= 0.0)
                    {
                        let last_price = match get_bars(&state, &[symbol.clone()], "1Min", 1).await {
                            Ok(bars) => bars[&symbol][0].close_price,
                            Err(e) => {
                                tracing::error!("Failed to get current price for {}: {:?}", symbol, e);
                                continue;
                            }
                        };

                        let qty = calculate_position_size(&account, last_price, config.risk_per_trade);

                        if qty > 0.0 {
                            let order = Order {
                                symbol: symbol.clone(),
                                qty: Some(qty as i32),
                                side: side.clone(),
                                order_type: Type::Limit,
                                time_in_force: TimeInForce::Day,
                                limit_price: Some(if side == Side::Buy {
                                    (last_price * 1.001) as i32
                                } else {
                                    (last_price * 0.999) as i32
                                }),
                                ..Order::default()
                            };

                            match create_order(State(state.clone()), Json(order)).await {
                                Ok(_) => tracing::info!("Order placed: {:?} {} shares of {}", side, qty, symbol),
                                Err(e) => tracing::error!("Failed to place order for {}: {:?}", symbol, e),
                            }
                        }
                    }
                }
            }
        }
    }
}