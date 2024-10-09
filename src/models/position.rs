use serde::{Deserialize, Serialize};
use serde_this_or_that::as_f64;
#[derive(Clone, Serialize, Deserialize)]
pub struct Position {
    pub asset_id: String,
    pub symbol: String,
    exchange: String,
    asset_class: String,
    avg_entry_price: String,
    #[serde(deserialize_with = "as_f64")]
    pub qty: f64,
}
