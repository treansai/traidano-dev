use apca::api::v2::order;
use apca::api::v2::order::CreateReqInit;
use apca::Client;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct Config {
    api_key: String,
    api_secret: String
}
pub struct AppState {
    pub alpaca_client: Client
}
#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderRequest {
    pub create_req_init: CreateReqInit,
    pub symbol: String,
    pub quantity: i32,
    pub side: order::Side
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
