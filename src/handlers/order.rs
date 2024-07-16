use crate::base::AppState;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{Method, StatusCode};
use axum::response::IntoResponse;
use axum::{response, Json};
use axum_macros::debug_handler;
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info, instrument};
use traidano::models::order::{Order, OrderParams};

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

    let mut url_query: String = "orders?".to_string();
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
