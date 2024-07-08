const ORDER_URL: &'static str = "https://";

#[derive(Debug, Clone, Deserialize)]
pub enum Side {
    Buy = "buy",
    Sell = "sell",
}

pub enum Type {
    Limit = "limit",
}

pub enum TimeInForce {
    Day = "day",
    GoodUntilCancel = "gtc",
}

// Todo: Add stop loss and take profit and order class
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Order<T, B> {
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

pub struct AlpacaClient {
    pub api_info: ApiInfo,
    pub client: Htt,
}
