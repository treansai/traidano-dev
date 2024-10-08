use crate::base::AppState;
use crate::error::{Error, RequestError};
use crate::handlers::rate_limited_request;
use crate::models::position::Position;
use crate::models::Clock;
use axum::body::Body;
use axum::extract::State;
use axum::http::Method;
use std::sync::Arc;
use traidano::RequestType;

pub async fn get_positions(state: &AppState) -> Result<Vec<Position>, RequestError> {
    match rate_limited_request::<Vec<Position>>(state, Method::GET, "positions", Body::empty(), RequestType::Order)
        .await
    {
        Ok(positions) => Ok(positions),
        Err(e) => {
            tracing::error!("Cannot get positions: {}", e);
            Err(e)
        }
    }
}

pub async fn is_market_open(state: &AppState) -> Result<bool, RequestError> {
    let clock = rate_limited_request::<Clock>(state, Method::GET, "clock", Body::empty(), RequestType::Order).await?;
    Ok(clock.is_open)
}
