use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct BotConfig {
    config: Config,
    balance: f64,
    asset_holdings: f64,
    client: Client
}