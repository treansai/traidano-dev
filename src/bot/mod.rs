pub mod bot_manager;
mod strategies;

use std::fmt;
use std::fmt::{Formatter, write};
use crate::base::{AppState, Client};
use crate::bot::strategies::mean_reversion::mean_reversion_strategy;
use crate::core::rate_limiter::RateLimiter;
use axum::body::Body;
use axum::http::Method;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BotStrategy {
    MeanReversion,
}

impl fmt::Display for BotStrategy{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            BotStrategy::MeanReversion => write!(f, "MeanReversion")
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MarketType {
    Crypto,
    Equity,
}

impl fmt::Display for MarketType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
       match *self {
           MarketType::Equity => write!(f, "Equity"),
           MarketType::Crypto => write!(f, "Crypto")
       }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BotConfig {
    pub id: String,
    pub name: String,
    pub market: MarketType,
    pub trading_strategy: BotStrategy,
    pub symbols: Vec<String>,
    pub lookback: usize,
    pub threshold: f64,
    pub risk_per_trade: f64,
    pub max_positions: usize,
    pub timeframes: Vec<String>,
    pub volatility_window: usize,
    pub volatility_threshold: f64,
}

pub struct Bot {
    pub config: BotConfig,
    pub handle: Option<tokio::task::JoinHandle<()>>,
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

#[derive(Debug, Clone, Serialize)]
pub struct BotInfo {
    pub config: BotConfig,
    pub is_running: bool,
}

impl From<&Bot> for BotInfo {
    fn from(bot: &Bot) -> Self {
        BotInfo {
            config: bot.config.clone(),
            is_running: bot.handle.is_some(),
        }
    }
}
