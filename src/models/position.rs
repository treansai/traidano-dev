use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Position {
    pub asset_id: String,
    pub symbol: String,
    exchange: String,
    asset_class: String,
    avg_entry_price: String,
    pub qty: f64,
}
