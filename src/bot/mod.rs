pub mod bot_manager;
mod strategies;

use crate::base::{AppState, Client};
use crate::bot::strategies::mean_reversion::mean_reversion_strategy;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{write, Formatter};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use crate::bot::strategies::smart_money::smart_money_strategy;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BotStrategy {
    MeanReversion,
    SmartMoney
}

impl fmt::Display for BotStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            BotStrategy::MeanReversion => write!(f, "MeanReversion"),
            BotStrategy::SmartMoney => write!(f, "SmartMoney")
        }
    }
}

impl FromStr for BotStrategy {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MeanReversion" => Ok(BotStrategy::MeanReversion),
            "SmartMoney" => Ok(BotStrategy::SmartMoney),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MarketType {
    Crypto,
    Equity,
}
impl FromStr for MarketType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Crypto" => Ok(MarketType::Crypto),
            "Equity" => Ok(MarketType::Equity),
            _ => Err(()),
        }
    }
}
impl fmt::Display for MarketType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            MarketType::Equity => write!(f, "Equity"),
            MarketType::Crypto => write!(f, "Crypto"),
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
                BotStrategy::SmartMoney => smart_money_strategy(state, config).await,
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
