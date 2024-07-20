use axum::http::StatusCode;
use base::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod base;
pub mod bot;
pub mod core;
pub mod dao;
pub mod error;
pub mod handlers;
pub mod models;

pub struct Config {
    pub api_key: String,
    pub api_secret: String,
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
