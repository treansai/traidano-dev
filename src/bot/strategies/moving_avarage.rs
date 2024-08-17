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
use ta::indicators::ExponentialMovingAverage;
use ta::Next;
use crate::bot::strategies::should_execute;

pub async fn moving_average_strategy(state: Arc<AppState>, config: BotConfig) {
    let mut interval = interval(Duration::from_secs(60));

    // Initialize EMA for each symbol
    let mut short_emas: HashMap<String, ExponentialMovingAverage> = HashMap::new();
    let mut long_emas: HashMap<String, ExponentialMovingAverage> = HashMap::new();

    for symbol in &config.symbols {
        short_emas.insert(symbol.clone(), ExponentialMovingAverage::new(10).unwrap());
        long_emas.insert(symbol.clone(), ExponentialMovingAverage::new(30).unwrap());
    }

    loop {
        interval.tick().await;

        let should_execute = match should_execute(&state, &config).await {
            Some(value) => value,
            None => continue,
        };

        if should_execute {
            // Get historical data for all symbols
            let all_bars = match get_bars(state.as_ref(), &config.symbols, &config.timeframes[0], 50).await {
                Ok(bars) => bars,
                Err(e) => {
                    tracing::error!("Failed to get historical data: {:?}", e);
                    continue;
                }
            };

            // Get account information
            let account = match get_account(&state).await {
                Ok(acc) => acc,
                Err(e) => {
                    tracing::error!("Failed to get account information: {:?}", e);
                    continue;
                }
            };

            // Get current positions
            let positions = match get_positions(state.as_ref()).await {
                Ok(pos) => pos,
                Err(e) => {
                    tracing::error!("Failed to get positions: {:?}", e);
                    continue;
                }
            };

            let current_positions: HashMap<String, f64> =
                positions.into_iter().map(|p| (p.symbol, p.qty)).collect();

            for (symbol, bars) in all_bars {
                if bars.len() < 50 {
                    tracing::warn!("Not enough data for {}", symbol);
                    continue;
                }

                let prices: Vec<f64> = bars.iter().map(|bar| bar.close_price).collect();
                let last_price = *prices.last().unwrap();

                let short_ema = short_emas.get_mut(&symbol).unwrap();
                let long_ema = long_emas.get_mut(&symbol).unwrap();

                let short_ema_value = prices.iter().fold(0.0, |_, &price| short_ema.next(price));
                let long_ema_value = prices.iter().fold(0.0, |_, &price| long_ema.next(price));

                let current_position = *current_positions.get(&symbol).unwrap_or(&0.0);

                if short_ema_value > long_ema_value && current_position <= 0.0 {
                    // Buy signal
                    let qty = calculate_position_size(&account, last_price, config.risk_per_trade);

                    if qty > 0.0 {
                        let order = Order {
                            symbol: symbol.clone(),
                            qty: Some(qty as i32),
                            side: Side::Buy,
                            order_type: Type::Limit,
                            time_in_force: TimeInForce::Day,
                            limit_price: Some((last_price * 1.001) as i32),
                            ..Order::default()
                        };

                        match create_order(State(state.clone()), Json(order)).await {
                            Ok(_) => tracing::info!("Buy order placed: {} shares of {}", qty, symbol),
                            Err(e) => tracing::error!("Failed to place buy order for {}: {:?}", symbol, e),
                        }
                    }
                } else if short_ema_value < long_ema_value && current_position >= 0.0 {
                    // Sell signal
                    let qty = calculate_position_size(&account, last_price, config.risk_per_trade);

                    if qty > 0.0 {
                        let order = Order {
                            symbol: symbol.clone(),
                            qty: Some(qty as i32),
                            side: Side::Sell,
                            order_type: Type::Limit,
                            time_in_force: TimeInForce::Day,
                            limit_price: Some((last_price * 0.999) as i32),
                            ..Order::default()
                        };

                        match create_order(State(state.clone()), Json(order)).await {
                            Ok(_) => tracing::info!("Sell order placed: {} shares of {}", qty, symbol),
                            Err(e) => tracing::error!("Failed to place sell order for {}: {:?}", symbol, e),
                        }
                    }
                }
            }
        }
    }
}

