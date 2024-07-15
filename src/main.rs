use crate::base::{AppState, RateLimiter};
use crate::configuration::BaseConfig;
use crate::handler::{create_order, get_account, get_all_order};
use crate::handlers::bot::{create_bot, get_bot, get_bots, remove_bot, stop_bot};
use axum::handler::Handler;
use axum::routing::delete;
use axum::{routing::get, routing::post, Router, ServiceExt};
use base::{ApiConfig, Client};
use std::sync::{Arc, Mutex};
use tokio::task::unconstrained;
use tracing::instrument::WithSubscriber;
use traidano::bot::bot_manager::BotManager;

mod base;
mod configuration;
mod handler;
mod handlers;
mod trade;
mod core;

#[tokio::main]
async fn main() {
    // init tracing
    tracing_subscriber::fmt::init();

    tracing::info!("App start");
    // connection to
    // configuration of api
    // let config = configuration::build_config().expect("cannot load configuration");
    // let api_config = config.api_config;
    // let api_config =
    //     ApiInfo::from_parts(api_config.base_url, api_config.api_key, api_config.secret).unwrap();

    let api_config = ApiConfig {
        base_url: "https://paper-api.alpaca.markets/v2/".to_string(),
        steam_url: None,
        api_key: "PKGA4DTIP5MZM8H0KQJL".to_string(),
        secret_key: "bEBazJLr2BdbyKDLNMPQKrSxGzwELRBYGICg5Jh1".to_string(),
    };

    // alpaca client
    let client = Client::builder().config(api_config).build().unwrap();

    // shared state
    let state = AppState {
        alpaca_client: client,
        bot_manager: Mutex::new(BotManager::new()),
        rate_limiter: Arc::new(Mutex::new(RateLimiter {})),
    };
    let shared_state = Arc::new(state);

    // the app server
    let app = Router::new()
        // account
        .route("/account", get(get_account))
        // orders
        .route("/orders", post(create_order).get(get_all_order))
        // bot manager
        .route("/bots", post(create_bot).get(get_bots))
        .route("/bots/:id", get(get_bot).delete(remove_bot))
        .route("/bots/:id/stop", post(stop_bot))
        .with_state(shared_state);

    // listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
