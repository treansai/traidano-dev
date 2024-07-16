use serde::{Deserialize, Serialize};

const ORDER_URL: &'static str = "https://";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Side {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Type {
    #[serde(rename = "limit")]
    Limit,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TimeInForce {
    #[serde(rename = "day")]
    Day,
    #[serde(rename = "gtc")]
    GoodUntilCancel,
}
