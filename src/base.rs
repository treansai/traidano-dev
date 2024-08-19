use crate::bot::bot_manager::BotManager;
use crate::core::rate_limiter::RateLimiter;
use crate::error::Error;
use crate::error::RequestError;
use axum::body::Body;
use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::{client::conn::http1::handshake, Method, Request, Uri};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioIo;
use native_tls::TlsConnector;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct ClientBuildError;

/// Configuration of the api
///
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct ApiConfig {
    pub base_url: String,
    pub stream_url: String,
    pub api_key: String,
    pub secret_key: String,
}

pub struct Client {
    pub api_config: ApiConfig,
}

pub struct ClientBuilder {
    config: Option<ApiConfig>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self { config: None }
    }

    pub fn config(mut self, config: ApiConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn build(self) -> Result<Client, ClientBuildError> {
        let config = match self.config {
            Some(conf) => conf,
            None => {
                tracing::error!("Base url required");
                return Err(ClientBuildError);
            }
        };

        Ok(Client { api_config: config })
    }
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub async fn send<T>(&self, method: Method, path: &str, body: Body) -> Result<T, RequestError>
    where
        T: DeserializeOwned,
    {
        use hyper_util::{client::legacy::Client, rt::TokioExecutor};
        let https = HttpsConnector::new();
        let client = Client::builder(TokioExecutor::new()).build(https);

        let mut full_url = self.api_config.base_url.clone();
        full_url.push_str(path);

        let req = Request::builder()
            .method(method)
            .uri(&full_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("APCA-API-KEY-ID", &self.api_config.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_config.secret_key)
            .body(body)
            .map_err(RequestError::HttpBuild)?;

        tracing::debug!("request  send : {:?}", req);
        let res = client
            .request(req)
            .await
            .map_err(RequestError::LegacyHyper)?;

        tracing::debug!("Response status: {}", res.status());
        tracing::debug!("Response: {:#?}\n", res);

        if res.status().is_success() {
            let body_bytes = res
                .into_body()
                .collect()
                .await
                .map_err(RequestError::Hyper)?
                .to_bytes();

            let response: T = serde_json::from_slice(&body_bytes)
                .map_err(|e| RequestError::Json(Error::Json(e)))?;
            Ok(response)
        } else {
            Err(RequestError::ApiError(res.status()))
        }
    }
}

pub struct AppState {
    pub alpaca_client: Client,
    pub db: PgPool,
    pub bot_manager: Mutex<BotManager>,
    pub rate_limiter: Arc<Mutex<RateLimiter>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_client() {
        let api_config = ApiConfig {
            base_url: "base".to_string(),
            steam_url: None,
            api_key: "key".to_string(),
            secret_key: "secret".to_string(),
        };

        let client = Client::builder().config(api_config.clone()).build();

        assert_eq!(client.unwrap().api_config, api_config)
    }
}
