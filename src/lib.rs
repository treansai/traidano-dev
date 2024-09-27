// lib.rs
use axum::http::StatusCode;
use once_cell::sync::Lazy;
use opentelemetry::KeyValue;
use opentelemetry::logs::LogError;
use opentelemetry::metrics::MetricsError;
use opentelemetry_otlp::{ExportConfig, HttpExporterBuilder, Protocol, WithExportConfig};
use opentelemetry_sdk::{logs, runtime, trace as sdktrace, Resource};
use opentelemetry_sdk::trace::Config as SdkTraceConfig;
use opentelemetry::trace::{TraceError, TracerProvider};
use opentelemetry_prometheus::exporter;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::runtime::Tokio;
use serde::Serialize;
use thiserror::Error;

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::new(vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        "traidano",
    )])
});


pub fn init_tracer_provider() -> Result<sdktrace::TracerProvider, TraceError> {
    let otlp_endpoint =  format!("{}/v1/traces", std::env::var("OTLP_ENDPOINT").expect("OTLP_ENDPOINT must be set"));
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_protocol(Protocol::HttpBinary)
                .with_endpoint(otlp_endpoint)
        )
        .with_trace_config(SdkTraceConfig::default().with_resource(RESOURCE.clone()))
        .install_batch(Tokio)
}

pub fn init_metrics() -> Result<SdkMeterProvider, MetricsError> {
    let otlp_endpoint = format!("{}/v1/metrics", std::env::var("OTLP_ENDPOINT").expect("OTLP_ENDPOINT must be set"));
    println!("{}", &otlp_endpoint);
    let exporter_config = ExportConfig {
        endpoint: otlp_endpoint,
        ..ExportConfig::default()
    };
    opentelemetry_otlp::new_pipeline()
        .metrics(Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_protocol(Protocol::HttpBinary)
                .with_export_config(exporter_config),
        )
        .with_resource(RESOURCE.clone())
        .build()
}
pub fn init_logs() -> Result<logs::LoggerProvider, LogError> {
    let otlp_endpoint = format!("{}/v1/logs", std::env::var("OTLP_ENDPOINT").expect("OTLP_ENDPOINT must be set"));
    println!("{}", &otlp_endpoint);
    opentelemetry_otlp::new_pipeline()
        .logging()
        .with_resource(RESOURCE.clone())
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_protocol(Protocol::HttpBinary)
                .with_endpoint(otlp_endpoint)
        )
        .install_batch(Tokio)
}

pub struct Config {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: String,
    pub status: String,
}

#[derive(Error, Debug)]
pub enum OrderError {
    #[error("Failed to create order: {0}")]
    CreationFailed(String),
    #[error("Invalid order parameters: {0}")]
    InvalidParameters(String),
}

impl From<OrderError> for StatusCode {
    fn from(error: OrderError) -> Self {
        match error {
            OrderError::CreationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            OrderError::InvalidParameters(_) => StatusCode::BAD_REQUEST,
        }
    }
}