use crate::base::AppState;
use crate::bot::{Bot, BotConfig};
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
