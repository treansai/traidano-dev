use axum::http::StatusCode;
use hyper::http::Error as HttpError;
use hyper::Error as HyperError;
use hyper_util::client::legacy::Error as LegacyHyperError;
use serde_json::Error as JsonError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("HTTP client error: {0}")]
    Hyper(#[from] HyperError),

    #[error("HTTP client error: {0}")]
    LegacyHyper(#[from] LegacyHyperError),

    #[error("HTTP request build error: {0}")]
    HttpBuild(#[from] HttpError),

    #[error("JSON deserialization error: {0}")]
    Json(#[from] Error),

    #[error("API returned an error status: {0}")]
    ApiError(StatusCode),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("JSON error: {0}")]
    Json(#[from] JsonError),

    // Add other error types as needed
    // #[error("Database error: {0}")]
    // Database(#[from] DatabaseError),
    #[error("Failed to acquire lock")]
    LockError,
    #[error("Bot not found")]
    BotNotFound,
}
