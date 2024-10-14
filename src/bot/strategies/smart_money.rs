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
use std::sync::Arc;
use std::time::Duration;
use opentelemetry::KeyValue;
use tokio::time::interval;

// Define a structure to hold market data
struct MarketData {
    price: f64,
    volume: f64,
}

// Function to identify support and resistance levels
fn identify_support_resistance(prices: &[f64], window: usize) -> (f64, f64) {
    let mut support = f64::MAX;
    let mut resistance = f64::MIN;

    for window in prices.windows(window) {
        let min = window.iter().fold(f64::MAX, |a, &b| a.min(b));
        let max = window.iter().fold(f64::MIN, |a, &b| a.max(b));

        support = support.min(min);
        resistance = resistance.max(max);
    }

    (support, resistance)
}

// Function to detect volume anomalies
fn detect_volume_anomaly(volumes: &[f64], threshold: f64) -> bool {
    let avg_volume = volumes.iter().sum::<f64>() / volumes.len() as f64;
    let last_volume = volumes.last().unwrap();

    last_volume > &(avg_volume * threshold)
}

// Function to analyze order flow
fn analyze_order_flow(data: &[MarketData], window: usize) -> bool {
    let price_changes: Vec<f64> = data.windows(2).map(|w| w[1].price - w[0].price).collect();

    let volume_weighted_price_changes: Vec<f64> = data
        .windows(2)
        .map(|w| (w[1].price - w[0].price) * w[1].volume)
        .collect();

    let sum_price_changes = price_changes.iter().sum::<f64>();
    let sum_volume_weighted_changes = volume_weighted_price_changes.iter().sum::<f64>();

    sum_volume_weighted_changes > sum_price_changes
}

pub async fn smart_money_strategy(state: Arc<AppState>, config: BotConfig) {
    let mut interval = interval(Duration::from_secs(300)); // Check every 5 minutes
    let support_gauge = state.meter.f64_gauge("support_gauge")
        .with_description("The support value gauge")
        .init();
    let resistance_gauge = state.meter.f64_gauge("resistance_gauge")
        .with_description("The resistance value gauge")
        .init();
    let sell_order_hist = state.meter.f64_histogram("sell_order_hist")
        .init();
    let buy_order_hist = state.meter.f64_histogram("buy_order_hist")
        .init();


    loop {
        interval.tick().await;

        let should_execute = match &config.market {
            MarketType::Crypto => true,
            MarketType::Equity => match is_market_open(&state).await {
                Ok(true) => true,
                Ok(false) => {
                    tracing::info!("Equity market is closed. Waiting for next check.");
                    tokio::time::sleep(Duration::from_secs(3600)).await;
                    continue;
                }
                Err(e) => {
                    tracing::error!("Failed to check if market is open: {:?}", e);
                    continue;
                }
            },
        };

        if should_execute {
            for symbol in &config.symbols {
                let request_type = match &config.market {
                    MarketType::Crypto => "crypto_data",
                    MarketType::Equity => "stock_data",
                };

                // Fetch historical data
                let bars = match get_bars(
                    state.as_ref(),
                    &[symbol.clone()],
                    &config.timeframes[0],
                    config.lookback.max(config.volatility_window),
                    config.volatility_window,
                    request_type,
                )
                .await
                {
                    Ok(bars) => bars.get(symbol).unwrap().clone(),
                    Err(e) => {
                        tracing::error!("Failed to get historical data for {}: {:?}", symbol, e);
                        continue;
                    }
                };

                if bars.len() < config.lookback {
                    tracing::warn!("Not enough data for {}", symbol);
                    continue;
                }

                let prices: Vec<f64> = bars.iter().map(|bar| bar.close_price).collect();
                let volumes: Vec<f64> = bars.iter().map(|bar| bar.volume as f64).collect();
                let market_data: Vec<MarketData> = bars
                    .iter()
                    .map(|bar| MarketData {
                        price: bar.close_price,
                        volume: bar.volume as f64,
                    })
                    .collect();

                // Identify support and resistance
                let (support, resistance) = identify_support_resistance(&prices, 20);
                support_gauge.record(support.clone(), &[
                    KeyValue::new("bot_id", format!("{}", config.id)),
                    KeyValue::new("bot_name", format!("{}", config.name)),
                ]);

                resistance_gauge.record(resistance.clone(), &[
                    KeyValue::new("bot_id", format!("{}", config.id)),
                    KeyValue::new("bot_name", format!("{}", config.name)),
                ]);


                tracing::debug!("support value :{}, resistance value : {}", support.clone(), resistance.clone());


                // Detect volume anomaly
                let volume_anomaly = detect_volume_anomaly(&volumes, 2.0);
                tracing::debug!("volue anomaly {}", volume_anomaly.clone());


                // Analyze order flow
                let bullish_order_flow = analyze_order_flow(&market_data, 20);
                tracing::debug!("bullish_order_flow {}", bullish_order_flow.clone());

                let last_price = *prices.last().unwrap();

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

                let current_position = positions
                    .iter()
                    .find(|p| p.symbol == *symbol)
                    .map(|p| p.qty)
                    .unwrap_or(0.0);

                tracing::info!("The current position is: {}", current_position.clone());

                // Trading logic
                if last_price <= support
                    && volume_anomaly
                    && bullish_order_flow
                    && current_position <= 0.0
                {
                    // Potential smart money accumulation, consider buying
                    let qty = calculate_position_size(&account, last_price, config.risk_per_trade);
                    
                    tracing::debug!("position 'qty' calculated {}", qty.clone());
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

                        create_order(State(state.clone()), Json(order)).await;
                        buy_order_hist.record(
                            qty as f64,
                            &[
                                KeyValue::new("bot_id", format!("{}", config.id)),
                                KeyValue::new("bot_name", format!("{}", config.name)),
                            ]);
                        tracing::info!("Buy order placed: {} shares of {}", qty, symbol);
                    }
                } else if last_price >= resistance
                    && volume_anomaly
                    && !bullish_order_flow
                    && current_position >= 0.0
                {
                    // Potential smart money distribution, consider selling
                    let qty = calculate_position_size(&account, last_price, config.risk_per_trade);

                    tracing::debug!("position 'qty' calculated {}", qty.clone());
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
                        create_order(State(state.clone()), Json(order)).await;
                        sell_order_hist.record(
                            qty as f64,
                                                &[
                                                    KeyValue::new("bot_id", format!("{}", config.id)),
                                                    KeyValue::new("bot_name", format!("{}", config.name)),
                                                ]);
                        tracing::info!("Sell order placed: {} shares of {}", qty, symbol);
                    }
                }
            }
        }
    }
}
