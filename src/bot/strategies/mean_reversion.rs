use crate::base::AppState;
use crate::bot::{is_market_open, BotConfig, MarketType};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

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
                if let Ok(true) = is_market_open(&state.alpaca_client, &state.rate_limiter).await {
                    let mut all_signals = HashMap::new();

                    for timeframe in &config.timeframes {
                        // todo: to implements
                        // match get_bars {  }
                    }
                }
                // get account
                // let account = match get {  };

                // get position
                let positions;
                for (symbol, signal) in all_signals {}
            }
        }
    }
    todo!()
}
