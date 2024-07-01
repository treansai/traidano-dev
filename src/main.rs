extern crate core;

use std::sync::Arc;
use apca::{ApiInfo, Client, api::v2::{account}, RequestError};
use apca::api::v2::account::{Account, GetError};
use axum::{Router, routing::get};
use axum::handler::Handler;
use axum::routing::post;
use tokio::task::unconstrained;
use tracing::instrument::WithSubscriber;
use traidano::AppState;
use crate::configuration::BaseConfig;
use crate::handlers::{create_order, get_account};
mod trade;
mod configuration;
mod handlers;


#[tokio::main]
async fn main() {
    // init tracing
    tracing_subscriber::fmt::init();

    tracing::info!("App start");
    // connection to
    // configuration of api
    let config = configuration::build_config().expect("cannot load configuration");
    let api_config = config.api_config;
    let api_config = ApiInfo::from_parts(
        api_config.base_url,
        api_config.api_key,
        api_config.secret
    ).unwrap();

    // alpaca client
    let client = Client::new(api_config);

    // shared state
    let state = AppState {
        alpaca_client: client
    };
    let shared_state = Arc::new(state);

    // the app server
    let app = Router::new()
        .route("/account", get(get_account))
        .route("/order", post(create_order))
        .with_state(shared_state);

    // listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
