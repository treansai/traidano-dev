use std::sync::Arc;
use std::time::Duration;
use crate::base::AppState;
use crate::bot::{BotConfig, MarketType};
use crate::handlers::market::is_market_open;

pub mod mean_reversion;
mod moving_avarage;
pub mod smart_money;
mod oth;

async fn should_execute(state: &Arc<AppState>, config: &BotConfig) -> Option<bool> {
    let should_execute = match config.market {
        MarketType::Crypto => true, // Crypto markets are typically always open
        MarketType::Equity => {
            match is_market_open(&state).await {
                Ok(true) => true,
                Ok(false) => {
                    tracing::info!("Equity market is closed. Waiting for next check.");
                    tokio::time::sleep(Duration::from_secs(3600)).await;
                    return None;
                }
                Err(e) => {
                    tracing::error!("Failed to check if market is open: {:?}", e);
                    return None;
                }
            }
        }
    };
    Some(should_execute)
}