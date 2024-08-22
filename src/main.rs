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
use std::time::Duration;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::trace::{TraceError, TracerProvider};
use opentelemetry_prometheus::PrometheusExporter;
use opentelemetry::{metrics::MeterProvider, KeyValue};
use opentelemetry_otlp::{MetricsExporter, WithExportConfig};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use opentelemetry_sdk::trace::Config as SdkTraceConfig;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_prometheus::exporter as prometheus_exporter;
use prometheus::{Registry as PrometheusRegistry};
use prometheus::{TextEncoder, Encoder};
use serde::Serialize;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::task::unconstrained;
use tower_http::trace::TraceLayer;
use tracing::instrument::WithSubscriber;
use tracing::{error, span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::Registry;
use traidano::{init_logs, init_metrics, init_tracer_provider};

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
    // Create a Prometheus registry
    let prometheus_registry = prometheus::Registry::new();

    let prometheus_exporter = prometheus_exporter()
        .with_registry(prometheus_registry)
        .build()
        .unwrap();

    let provider = SdkMeterProvider::builder().with_reader(prometheus_exporter).build();

    // init tracer_provider
    let tracer_provider = init_tracer_provider().expect("error to init trace provider");

    // init metrics
    let metrics = init_metrics().expect("error to initialize metrics provider");

    // init logs
    let logs = init_logs().expect("error to initialize log provider");


    // Set up tracing with OpenTelemetry
    let base_url = std::env::var("BASE_URL").expect("base url must be set");
    let stream_url = std::env::var("STREAM_URL").expect("STREAM_URL must be set");
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");
    let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY must be set");

    let api_config = ApiConfig {
        base_url,
        stream_url,
        api_key,
        secret_key,
    };

    // alpaca client
    let client = Client::builder().config(api_config).build().unwrap();

    // postgres pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .map_err(|e| {
            eprintln!("Failed to connect to the database: {}", e);
            std::process::exit(1);
        }).unwrap();

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
        .route("/metrics", get(metrics_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);

    // listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9494")
        .await
        .unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}



async fn metrics_handler() -> String {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = prometheus::default_registry().gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}