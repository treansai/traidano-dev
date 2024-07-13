use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bar {
    #[serde(rename = "c")]
    pub close_price: f32,
    #[serde(rename = "h")]
    pub high_price: f32,
    #[serde(rename = "l")]
    pub low_price: f32,
    #[serde(rename = "n")]
    pub n: u32,
    #[serde(rename = "o")]
    pub open_price: f32,
    #[serde(rename = "t")]
    pub timestamp: String,
    #[serde(rename = "v")]
    pub volume: u32,
    #[serde(rename = "wv")]
    pub wv: f32,
}
