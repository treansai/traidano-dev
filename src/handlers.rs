use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::os::macos::raw::stat;
use axum::{extract::State, http::StatusCode, Json, response, serve};
use num_decimal::num_rational::BigRational;
use num_decimal::Num;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::body::Body;
use axum::extract::Path;
use axum::http::Response;
use axum::response::IntoResponse;
use axum_macros::debug_handler;
use hyper::Method;
use serde_json::json;
use thiserror::Error;
use tracing::{error, info, instrument};
use traidano::{OrderError, OrderResponse};
use traidano::models::account::Account;
use crate::base::AppState;
use crate::trade::Order;

#[instrument(skip(state))]
pub(crate) async fn get_account(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("app_events: get account information");
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

    let account: Account = serde_json::from_value(response.clone())
        .expect("error in deserialization");

    tracing::info!(
        account_id=?account.id,
        "Retrieved account information"
    );


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

/// Create order
#[instrument(skip(state))]
#[debug_handler]
pub async fn create_order(
    State(state): State<Arc<AppState>>,
    Json(request): Json<Order>
) -> response::Response {
    info!("receive '{:?}' order", &request.side);

    match state.alpaca_client
        .send::<serde_json::Value>(
            Method::POST,
            "orders",
            Body::from(serde_json::to_string(&request).unwrap())
        )
        .await
    {
        Ok(response) => {
            info!("order created");
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Error creating order: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to create order"}))).into_response()
        }
    }
}

#[debug_handler]
#[instrument(skip(state))]
pub async fn get_all_order(
    Path(param) : Path<HashMap<String, String>>,
    State(state): State<Arc<AppState>>
) -> response::Response {
    info!("get all order");

    match state.alpaca_client.send::<serde_json::Value>(
        Method::GET,
        "orders",
        Body::empty()
    ).await {
        Ok(response) => {
            info!("order created");
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Error creating order: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to create order"}))).into_response()
        }
    }
}



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