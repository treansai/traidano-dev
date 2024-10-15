use std::fmt::Debug;
// main.rs
use crate::base::AppState;
use crate::bot::bot_manager::BotManager;
use crate::core::rate_limiter::RateLimiter;
use crate::handlers::account::get_http_account;
use crate::handlers::bot::{create_bot, get_bot, get_bots, remove_bot, stop_bot};
use crate::handlers::order::{create_order, get_all_order};
use anyhow::Context;
use axum::handler::Handler;
use axum::response::IntoResponse;
use axum::Json;
use axum::{routing::get, routing::post, Router, ServiceExt};
use base::{ApiConfig, Client};
use opentelemetry::metrics::MeterProvider;
use opentelemetry::trace::TracerProvider;
use opentelemetry::trace::{TraceContextExt, Tracer, TracerProvider as _};
use opentelemetry::{global, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_prometheus::exporter as prometheus_exporter;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_stdout as stdout;
use prometheus::{Encoder, TextEncoder};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use log::LevelFilter;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing::instrument::WithSubscriber;
use tracing::{debug, info, trace, Level, Subscriber};
use tracing::{error, span};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{fmt, Registry};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_subscriber::fmt::writer::MakeWriterExt;
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
    // Create tracer
    let tracer_provider = init_tracer_provider().unwrap();
    global::set_tracer_provider(tracer_provider.clone());

    // let tracer = global::tracer_provider()
    //     .tracer_builder("basic")
    //     .build();

    let tracer = tracer_provider.tracer("traindano");

    let telemetry = OpenTelemetryLayer::new(tracer);

    // logger
    let logger_provider = init_logs().unwrap();
    let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    // meter
    let meter_provider = init_metrics().unwrap();
    global::set_meter_provider(meter_provider.clone());
    let meter = global::meter_with_version("basic", Some("v1.0"), Some("schema_url"), None);

    // get env
    let running_env = std::env::var("ENV").unwrap_or("dev".to_string());
    info!("running app in {}", running_env.clone());
    let trace_level = match running_env.as_str() {
        "dev" => "trace",
        "prod" => "info",
        "debug" => "debug",
        _ => "info"
    };

    // filter
    let filter = EnvFilter::new(trace_level)
        .add_directive("hyper=error".parse().unwrap())
        .add_directive("tonic=error".parse().unwrap())
        .add_directive("tower_http=debug".parse().unwrap())
        .add_directive("reqwest=error".parse().unwrap());

    tracing_subscriber::registry()
        .with(telemetry)
        .with(logger_layer)
        .with(fmt::layer())
        .with(filter)
        .init();

    // Get vars
    let base_url =
        std::env::var("BASE_URL").unwrap_or("https://paper-api.alpaca.markets/v2/".to_string());
    let stream_url =
        std::env::var("STREAM_URL").unwrap_or("https://paper-api.alpaca.markets/v2/".to_string());
    let stock_data_url = std::env::var("STOCK_DATA_URL")
        .unwrap_or("https://data.alpaca.markets/v2/stocks".to_string());
    let crypto_data_url = std::env::var("CRYPTO_DATA_URL")
        .unwrap_or("https://data.alpaca.markets/v1beta3/crypto/".to_string());
    let api_key = std::env::var("API_KEY").unwrap_or("PKHN2MOBD64Q1N4AUNIZ".to_string());
    let secret_key = std::env::var("SECRET_KEY")
        .unwrap_or("HKqNazp498yYZHQVfaBU3ubF52JyVHFNxjH2ijoq".to_string());

    let api_config = ApiConfig {
        base_url,
        stream_url,
        stock_data_url,
        crypto_data_url,
        api_key,
        secret_key,
    };

    // alpaca client
    let client = Client::builder().config(api_config).build().unwrap();

    // postgres pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgresql://postgres:postgres@localhost:5432/postgres".to_string());
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .map_err(|e| {
            eprintln!("Failed to connect to the database: {}", e);
            std::process::exit(1);
        })
        .unwrap();

    let mut bot_manager = BotManager::new();

    // shared state
    let state = AppState {
        alpaca_client: client,
        db: db.clone(),
        bot_manager: Mutex::new(bot_manager),
        rate_limiter: Arc::new(Mutex::new(RateLimiter::new(200.0 / 60.0, 50.0))),
        //tracer,
        meter,
    };

    let shared_state = Arc::new(state);
    shared_state
        .bot_manager
        .lock()
        .await
        .init(&db, shared_state.clone())
        .await;

    // the app server
    let app = Router::new()
        // base
        .route("/", get(base_handler))
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
    let host = std::env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or("9494".to_string());
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();

    info!("App is running");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    global::shutdown_tracer_provider();
    logger_provider.shutdown().unwrap();
    meter_provider.shutdown().unwrap();
}

async fn metrics_handler() -> String {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = prometheus::default_registry().gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

#[tracing::instrument]
pub async fn base_handler() -> impl IntoResponse {
    trace!("base url reached");
    Json("Hello Traidano").into_response()
}
