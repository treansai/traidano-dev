pub mod bot_manager;
mod strategies;

use crate::base::{AppState, Client, RateLimiter};
use crate::bot::strategies::mean_reversion::mean_reversion_strategy;
use axum::body::Body;
use axum::http::Method;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BotStrategy {
    MeanReversion,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MarketType {
    Crypto,
    Equity,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BotConfig {
    pub id: String,
    pub market: MarketType,
    pub trading_strategy: BotStrategy,
    pub symbols: Vec<String>,
    pub lookback: usize,
    pub threshold: f64,
    pub risk_per_trade: f64,
    pub max_position: usize,
    pub timeframes: Vec<String>,
    pub volatility_window: usize,
    pub volatility_threshold: f64,
}

pub struct Bot {
    config: BotConfig,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl Bot {
    pub fn new(config: BotConfig) -> Self {
        Bot {
            config,
            handle: None,
        }
    }

    pub async fn start(&mut self, state: Arc<AppState>) {
        let config = self.config.clone();
        let handle = tokio::spawn(async move {
            match config.trading_strategy {
                BotStrategy::MeanReversion => mean_reversion_strategy(state, config).await,
                _ => (),
            }
        });
        self.handle = Some(handle);
    }

    pub async fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort()
        }
    }
}

pub async fn is_market_open(
    client: &Client,
    rate_limiter: &Arc<Mutex<RateLimiter>>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let clock: serde_json::Value = client.send(Method::GET, "clock", Body::empty()).await?;

    Ok(clock["is_open"].as_bool().unwrap())
}
