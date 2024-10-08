use crate::base::{AppState, Client};
use crate::error::{Error, RequestError};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::de::{DeserializeOwned, StdError};
use std::sync::{Arc, Mutex};
use traidano::RequestType;

pub mod account;
pub mod bar;
pub mod bot;
pub mod market;
pub mod order;

pub async fn rate_limited_request<T>(
    app_state: &AppState,
    method: Method,
    path: &str,
    body: Body,
    request_type: RequestType
) -> Result<T, RequestError>
where
    T: DeserializeOwned,
{
    // Acquire the rate limiter lock
    let mut guard = app_state.rate_limiter.lock().await;
    guard.acquire().await;

    drop(guard);

    // Send the request using the Alpaca client
    app_state.alpaca_client.send::<T>(method, path, body, request_type).await
}

impl IntoResponse for RequestError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            RequestError::Hyper(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            RequestError::HttpBuild(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            RequestError::Json(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            RequestError::ApiError(status) => (status, "API Error".to_string()),
            // Add other variants as needed
            _ => (StatusCode::NOT_FOUND, "NotFound".to_string()),
        };

        (status, error_message).into_response()
    }
}
