use crate::base::AppState;
use crate::handlers::rate_limited_request;
use axum::body::Body;
use axum::extract::State;
use axum::http::Method;
use std::sync::Arc;
use traidano::models::position::Position;

pub async fn get_positions(state: &AppState) -> Result<Vec<Position>, Box<dyn std::error::Error>> {
    rate_limited_request(state, Method::GET, "positions", Body::empty())
}

pub async fn is_market_open(state: &AppState) -> Result<bool, Box<dyn std::error::Error>> {
    let clock: serde_json::Value = rate_limited_request(state, Method::GET, "clock", Body::empty());
    Ok(clock["is_open"].as_bool().unwrap_or(false))
}
