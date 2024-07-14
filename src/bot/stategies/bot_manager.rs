use std::collections::HashMap;
use crate::bot::Bot;

pub struct BotManager {
    pub bots : HashMap<String, Bot>,
}

impl BotManager {
    pub fn new() -> Self {
        BotManager {
            bots : HashMap::new()
        }
    }
}