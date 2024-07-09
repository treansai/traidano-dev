use serde::Deserialize;

const ORDER_URL: &'static str = "https://";

#[derive(Debug, Clone, Deserialize)]
pub enum Side {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Type {
    Limit,
}

#[derive(Debug, Clone, Deserialize)]
pub enum TimeInForce {
    #[serde(rename = "day")]
    Day,

    #[serde(rename = "gtc")]
    GoodUntilCancel,
}

// Todo: Add stop loss and take profit and order class
#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    pub symbole: String,
    pub qty: Option<i32>,
    pub national: Option<i32>,
    pub side: Side,
    pub order_type: Type,
    pub time_in_force: TimeInForce,
    pub limit_price: Option<i32>,
    pub stop_price: Option<i32>,
    pub trail_price: Option<i32>,
    pub trail_percent: Option<i32>,
    pub extended_hours: Option<bool>,
    pub client_order_id: Option<String>,
}
