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
use axum::response::IntoResponse;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_prometheus::PrometheusExporter;
use opentelemetry::{metrics::MeterProvider, KeyValue};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{Registry as PrometheusRegistry};
use prometheus::{TextEncoder, Encoder};
use tokio::sync::Mutex;
use tokio::task::unconstrained;
use tracing::instrument::WithSubscriber;
use tracing::{error, span};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

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
    // create a new prometheus registry
    let registry = prometheus::Registry::new();

    // configure OpenTelemetry to use this registry
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build().unwrap();

    // Initialize OpenTelemetry Tracer Provider
    let provider = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    let tracer = provider.tracer("traidano");

    // Set up tracing with OpenTelemetry
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default().with(telemetry);
    // init tracing
    // Trace executed code
    tracing::subscriber::with_default(subscriber, || {
        // Spans will be sent to the configured OpenTelemetry exporter
        let root = span!(tracing::Level::TRACE, "app_start", work_units = 2);
        let _enter = root.enter();

        error!("This event will be logged in the root span.");
    });

    tracing::info!("App start");

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

    let mut bot_manager = BotManager::new();

    // shared state
    let state = AppState {
        alpaca_client: client,
        db: db.clone(),
        bot_manager: Mutex::new(bot_manager),
        rate_limiter: Arc::new(Mutex::new(RateLimiter::new(200.0 / 60.0, 50.0))),
    };

    let shared_state = Arc::new(state);
    shared_state.bot_manager.lock().await.init(&db, shared_state.clone()).await;

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
        // instrumentation
        .route("/metrics", get(move || async move {
            let mut buffer = Vec::new();
            let encoder = prometheus::TextEncoder::new();
            let metric_family = registry.gather();
            encoder.encode(&metric_family, &mut buffer).unwrap();
            let response = String::from_utf8(buffer).unwrap();
            response.into_response()
        }))
        .with_state(shared_state);

    // listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9494")
        .await
        .unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
