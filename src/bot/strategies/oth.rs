use crate::base::AppState;
use crate::bot::{BotConfig, MarketType};
use crate::core::functions::calculate_position_size;
use crate::handlers::account;
use crate::handlers::account::get_account;
use crate::handlers::bar::get_bars;
use crate::handlers::market::{get_positions, is_market_open};
use crate::handlers::order::create_order;
use crate::models::order::Order;
use crate::models::trade::{Side, TimeInForce, Type};
use axum::extract::State;
use axum::Json;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use traidano::RequestType;

/// This is a mean reversion bot
pub async fn mean_reversion_strategy(state: Arc<AppState>, config: BotConfig) {
    // checking interval in sec (every 1min)
    let mut interval = interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        match config.clone().market {
            MarketType::Crypto => {}
            MarketType::Equity => {
                // Todo: evoid app orther than handler to communicate with alcapa api
                // todo: change is_open_market to handler ang get bars
                if let Ok(true) = is_market_open(&state).await {
                    let mut all_signals = HashMap::new();

                    for timeframe in &config.timeframes {
                        // todo: to implemen
                        match get_bars(
                            state.as_ref(),
                            &config.symbols,
                            timeframe,
                            config.lookback.max(config.volatility_window),
                            config.volatility_window,
                            "crypto_data",
                        )
                        .await
                        {
                            Ok(all_bars) => {
                                for (symbol, bars) in all_bars {
                                    if bars.len() < config.lookback.max(config.volatility_window) {
                                        tracing::warn!(
                                            "Not enouth data for {} on timeframe {}",
                                            symbol,
                                            timeframe
                                        );
                                        continue;
                                    }

                                    let prices: Vec<f64> =
                                        bars.iter().map(|bar| bar.close_price).collect();
                                    let mean = prices.iter().take(config.lookback).sum::<f64>()
                                        / config.lookback as f64;
                                    let last_price = *prices.last().unwrap();

                                    // todo: volaitility by choice
                                    //let curr_volatility = calculate_volatility(&prices[prices.len() - config.volatility_window..]);
                                    //let curr_volatility = calculate_volatility(&prices[prices.len() - config.volatility_window]);

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
                    let account = get_account(&state).await.unwrap();

                    // get position
                    let positions = get_positions(state.as_ref()).await.unwrap();

                    let current_positions: HashMap<String, f64> =
                        positions.into_iter().map(|p| (p.symbol, p.qty)).collect();

                    for (symbol, signal) in all_signals {
                        if signal.abs() == config.timeframes.len() as i32 {
                            let side = if signal > 0 { Side::Buy } else { Side::Sell };
                            let current_position = *current_positions.get(&symbol).unwrap_or(&0.0);

                            if (side == Side::Buy && current_position <= 0.0)
                                || (side == Side::Sell && current_position >= 0.0)
                            {
                                let last_price = match get_bars(
                                    &state,
                                    &[symbol.clone()],
                                    "1Min",
                                    1,
                                    config.volatility_window,
                                    "crypto_data",
                                )
                                .await
                                {
                                    Ok(bars) => bars[&symbol][0].close_price,
                                    Err(e) => {
                                        tracing::error!(
                                            "Failed to get current price for {}: {:?}",
                                            symbol,
                                            e
                                        );
                                        continue;
                                    }
                                };

                                let qty = calculate_position_size(
                                    &account,
                                    last_price,
                                    config.risk_per_trade,
                                );

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

                                    create_order(State(state.clone()), Json(order)).await;
                                    tracing::info!(
                                        "Order placed: {:?} {} shares of {}",
                                        side,
                                        qty,
                                        symbol
                                    );
                                }
                            }
                        }
                    }
                } else {
                    tracing::info!("Market is closed. Waiting for next check.");
                    tokio::time::sleep(Duration::from_secs(3600)).await;
                }
            }
        }
    }
}
