use apca::api::v2::order::{Class, StopLoss, TakeProfit, TimeInForce, Type};
use apca::Client;
use axum::http::StatusCode;
use num_decimal::Num;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct Config {
    pub api_key: String,
    pub api_secret: String,
}
pub struct AppState {
    pub alpaca_client: Client,
}
#[derive(Debug, Default, Clone, Deserialize)]
pub struct CreateOrderRequest {
    pub symbol: String,
    pub quantity: i32,
    pub side: String,
    pub class: Class,
    pub type_: Type,
    pub time_in_force: TimeInForce,
    pub limit_price: Option<Num>,
    pub stop_price: Option<Num>,
    pub trail_price: Option<Num>,
    pub trail_percent: Option<Num>,
    pub take_profit: Option<TakeProfit>,
    pub stop_loss: Option<StopLoss>,
    pub extended_hours: bool,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: String,
    pub status: String,
}

#[derive(Error, Debug)]
pub enum OrderError {
    #[error("Failed to create order: {0}")]
    CreationFailed(String),
    #[error("Invalid order parameters: {0}")]
    InvalidParameters(String),
}

impl From<OrderError> for StatusCode {
    fn from(error: OrderError) -> Self {
        match error {
            OrderError::CreationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            OrderError::InvalidParameters(_) => StatusCode::BAD_REQUEST,
        }
    }
}
