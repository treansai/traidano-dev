use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bar {
    pub close_price: f32,
    pub high_price: f32,
    pub low_price: f32,
    pub n: u32,
    pub open_price: f32,
    pub timestamp: chrono::DateTime<Utc>,
    pub volume: u32,
    pub wv: f32,
}


