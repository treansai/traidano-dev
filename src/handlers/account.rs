use crate::base::AppState;
use crate::handlers::rate_limited_request;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Method, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;
use tracing::instrument;
use traidano::models::account::Account;

async fn rate_limited_get_account(state: &Arc<AppState>) -> Result<Account, StatusCode> {
    rate_limited_request(&state, Method::GET, "accounts", Body::empty())
        .await
        .map_err(|e| {
            tracing::error!(e);
            println!(" error : {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[instrument(skip(state))]
pub async fn get_account(state: &Arc<AppState>) -> Result<Account, StatusCode> {
    tracing::info!("internal_request: get account information");
    tracing::trace!("rate limit: {}", state.rate_limiter.lock().unwrap().rate);
    let response: Account = rate_limited_get_account(&state).await?;

    Ok(response)
}

#[instrument(skip(state))]
pub async fn get_http_account(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("app_events: get account information");
    tracing::trace!("rate limit: {}", &state.rate_limiter.lock().unwrap().rate);
    let response = rate_limited_get_account(&state).await?;

    let account: Account =
        serde_json::from_value(response.clone()).expect("error in deserialization");

    tracing::info!(
        account_id=?account.id,
        "Retrieved account information"
    );

    Ok(Json(response))
}
