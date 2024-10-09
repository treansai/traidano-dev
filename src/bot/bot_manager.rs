use crate::base::AppState;
use crate::bot::{Bot, BotConfig};
use crate::dao::bot::get_all_running_bot;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;

pub struct BotManager {
    pub bots: HashMap<String, Bot>,
}

impl BotManager {
    pub fn new() -> Self {
        BotManager {
            bots: HashMap::new(),
        }
    }

    ///init bot from db
    pub async fn init(&mut self, db: &PgPool, app_state: Arc<AppState>) {
        match get_all_running_bot(db).await {
            Ok(bots) => {
                for bot_info in bots {
                    if bot_info.clone().is_running {
                        tracing::info!("Initializing bot {} ...", &bot_info.config.id);

                        self.create_bot(bot_info.config.clone(), app_state.clone())
                            .await;

                        tracing::info!("Bot initialized");
                    } else {
                        tracing::info!("bot {} is not in running state", bot_info.config.id);
                    }
                }
            }
            Err(_) => {
                tracing::error!("Cannot initialized bots");
            }
        }
    }

    /// Create new bot to a manager
    pub async fn create_bot(&mut self, config: BotConfig, app_state: Arc<AppState>) {
        let mut bot = Bot::new(config.clone());
        bot.start(app_state).await;
        self.bots.insert(config.id.clone(), bot);
    }

    pub async fn stop_bot(&mut self, id: &str) {
        if let Some(bot) = self.bots.get_mut(id) {
            bot.stop().await;
        }
    }

    pub async fn remove_bot(&mut self, id: &str) {
        if let Some(mut bot) = self.bots.remove(id) {
            bot.stop().await;
        }
    }
}
