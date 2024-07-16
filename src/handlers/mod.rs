use std::sync::{Arc, Mutex};
use axum::body::Body;
use axum::http::Method;
use serde::de::DeserializeOwned;
use crate::base::{AppState, Client, RateLimiter};

pub mod bot;
mod market;
mod account;

pub async fn rate_limited_request<T>(
    app_state: &AppState,
    method: Method,
    path: &str,
    body: Body) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned
{
    app_state.rate_limiter.lock().unwrap().acquire().await;
    app_state.alpaca_client.send::<T>(
        method,
        path,
        body
    )
}
