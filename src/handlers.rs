use crate::base::AppState;
use crate::trade::Order;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::Response;
use axum::response::IntoResponse;
use axum::{extract::State, http::StatusCode, response, serve, Json};
use axum_macros::debug_handler;
use hyper::Method;
use num_decimal::num_rational::BigRational;
use num_decimal::Num;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::os::macos::raw::stat;
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info, instrument};
use traidano::models::account::Account;
use traidano::models::order::OrderParams;
use traidano::{OrderError, OrderResponse};

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

    let account: Account =
        serde_json::from_value(response.clone()).expect("error in deserialization");

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
    Json(request): Json<Order>,
) -> response::Response {
    info!("receive '{:?}' order", &request.side);

    match state
        .alpaca_client
        .send::<serde_json::Value>(
            Method::POST,
            "orders",
            Body::from(serde_json::to_string(&request).unwrap()),
        )
        .await
    {
        Ok(response) => {
            info!("order created");
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Error creating order: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create order"})),
            )
                .into_response()
        }
    }
}

pub async fn get_params(Query(params): Query<OrderParams>) {
    let query = params.query();
    println!("{}", query);
}

#[debug_handler]
#[instrument(skip(state, params))]
pub async fn get_all_order(
    Query(params): Query<OrderParams>,
    State(state): State<Arc<AppState>>,
) -> response::Response {
    info!("get all order");

    let mut url_query : String = "orders?".to_string();
    url_query.push_str(&params.query());

    match state
        .alpaca_client
        .send::<serde_json::Value>(Method::GET, url_query.as_str(), Body::empty())
        .await
    {
        Ok(response) => {
            info!("order created");
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Error creating order: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create order"})),
            )
                .into_response()
        }
    }
}
