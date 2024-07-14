mod stategies;

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::base::AppState;
use crate::bot::stategies::mean_reversion::mean_reversion_strategy;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BotStrategy {
    MeanReversion
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BotConfig {
    pub trading_strategy: BotStrategy,
    pub symbols: Vec<String>,
    pub lookback: usize,
    pub threshold: f64,
    pub risk_per_trade: f64,
    pub max_position: usize,
}

pub struct Bot {
    config: BotConfig,
    handle : Option<tokio::task::JoinHandle<()>>
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
                BotStrategy::MeanReversion => mean_reversion_strategy(config).await,
                _ => ()
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