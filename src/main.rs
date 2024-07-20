use crate::base::AppState;
use crate::bot::bot_manager::BotManager;
use crate::configuration::BaseConfig;
use crate::core::rate_limiter::RateLimiter;
use crate::handlers::account::{get_account, get_http_account};
use crate::handlers::bot::{create_bot, get_bot, get_bots, remove_bot, stop_bot};
use crate::handlers::order::{create_order, get_all_order};
use anyhow::Context;
use axum::handler::Handler;
use axum::routing::delete;
use axum::{routing::get, routing::post, Router, ServiceExt};
use base::{ApiConfig, Client};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::unconstrained;
use tracing::instrument::WithSubscriber;

pub mod base;
pub mod bot;
mod configuration;
pub mod core;
pub mod dao;
pub mod error;
pub mod handler;
pub mod handlers;
pub mod models;
pub mod trade;

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

    // postgres pool
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect("postgresql://postgres:postgres@localhost:5432/postgres")
        .await
        .unwrap();

    // shared state
    let state = AppState {
        alpaca_client: client,
        db,
        bot_manager: Mutex::new(BotManager::new()),
        rate_limiter: Arc::new(Mutex::new(RateLimiter::new(200.0 / 60.0, 50.0))),
    };
    let shared_state = Arc::new(state);

    // the app server
    let app = Router::new()
        // account
        .route("/account", get(get_http_account))
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
