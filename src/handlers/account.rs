use crate::base::AppState;
use crate::error::RequestError;
use crate::handlers::rate_limited_request;
use crate::models::account::Account;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Method, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tracing::instrument;

async fn rate_limited_get_account<T>(state: &Arc<AppState>) -> Result<T, RequestError>
where
    T: DeserializeOwned,
{
    rate_limited_request::<T>(state, Method::GET, "account", Body::empty()).await
}

#[instrument(skip(state))]
pub async fn get_account(state: &Arc<AppState>) -> Result<Account, RequestError> {
    tracing::info!("internal_request: get account information");
    tracing::trace!("rate limit: {}", state.rate_limiter.lock().await.rate);
    let response = rate_limited_get_account::<Account>(state).await?;

    Ok(response)
}

#[instrument(skip(state))]
pub async fn get_http_account(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    tracing::info!("app_events: get account information");
    tracing::info!("rate limit: {}", state.rate_limiter.lock().await.rate);
    match rate_limited_get_account::<Account>(&state).await {
        Ok(account) => {
            tracing::info!(
               account_id = ?account.id,
                "Retrieved account information"
            );
            Json(account).into_response()
        }
        Err(e) => {
            tracing::error!("Cannot get account information {}", e.to_string());
            e.into_response()
        }
    }
}
