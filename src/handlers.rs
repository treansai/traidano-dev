use std::error::Error;
use std::future::Future;
use axum::{extract::State, http::StatusCode, Json};
use num_decimal::num_rational::BigRational;
use num_decimal::Num;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::body::Body;
use axum::response::IntoResponse;
use hyper::Method;
use thiserror::Error;
use tracing::{error, info, instrument};
use traidano::{OrderError, OrderResponse};
use traidano::models::account::Account;
use crate::base::AppState;

#[instrument(skip(state))]
pub(crate) async fn get_account(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let response = state
        .alpaca_client
        .send::<serde_json::Value>(
            Method::GET,
            "account",
            Body::empty(), // Use Body::empty() for GET requests
        )
        .await
        .map_err(|e| {
            tracing::error!(e);
            println!(" error : {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(response))
}


// /// Get Account information
// #[instrument(skip(state))]
// pub async fn get_account(
//     State(state): State<Arc<AppState>>,
// ) -> Result<Json<account::Account>, StatusCode> {
//     tracing::info!("app_events: get account information");
//     let client = &state.alpaca_client;
//
//     match client.issue::<account::Get>(&()).await {
//         Ok(account) => {
//             tracing::info!(
//                 account_id = ?account.id,
//                 "Retrieved account information"
//             );
//             Ok(Json(account))
//         }
//         Err(e) => {
//             tracing::error!("Failed to get account: {:?}", e);
//             Err(StatusCode::INTERNAL_SERVER_ERROR)
//         }
//     }
// }
//
// #[instrument(skip(state), fields(symbol = %request.symbol, side = %request.side))]
// pub async fn create_order(
//     Json(request): Json<CreateOrderRequest>,
//     State(state): State<Arc<AppState>>,
// ) -> Result<Json<OrderResponse>, OrderError> {
//     if order::Side::from(request.side) == order::Side::Buy {
//         info!("Received buy order request");
//     } else {
//         info!("Received sell order request");
//     }
//
//     let order_request = request.create_req_init.init(
//         request.symbol,
//         request.side,
//         order::Amount::quantity(request.quantity),
//     );
//
//     let order = state
//         .alpaca_client
//         .issue::<order::Create>(&order_request)
//         .await
//         .map_err(|e| {
//             error!("Failed to create order: {:?}", e);
//             OrderError::CreationFailed(e.to_string())
//         })?;
//
//     info!(order_id = %order.id, "Buy order created successfully");
//
//     Ok(Json(OrderResponse {
//         id: order.id.to_string(),
//         status: order.status.to_string(),
//     }))
// }