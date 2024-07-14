use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id : String,
    pub equity: f64,
    pub buying_power: f64
}