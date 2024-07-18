use serde::{Deserialize, Serialize};
use serde_this_or_that::{as_f64};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    #[serde(deserialize_with = "as_f64")]
    pub equity: f64,
    #[serde(deserialize_with = "as_f64")]
    pub buying_power: f64,
}
