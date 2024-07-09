use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::{client::conn::http1::handshake, Method, Request, Uri};
use hyper_util::rt::TokioIo;
use serde::de::DeserializeOwned;
use std::error::Error;
use axum::body::Body;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use native_tls::TlsConnector;

#[derive(Debug)]
pub struct ClientBuildError;

/// Configuration of the api
///
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct ApiConfig {
    pub base_url: String,
    pub steam_url: Option<String>,
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

    pub async fn send<T>(&self, method: Method, path: &str, body: Body) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned,
    {
        let mut full_url = self.api_config.base_url.clone();
        full_url.push_str(path);

        let url = full_url.parse::<Uri>()?;

        let host = url.host().expect("Url has no host");
        let port = url.port_u16().unwrap_or(443); // Changed to 443 for HTTPS
        let addr = format!("{}:{}", host, port);


        let tls_connector = TlsConnector::new()?;

        let tcp_stream = TcpStream::connect(addr).await?;
        let tls_stream = tls_connector.connect(host, tcp_stream).unwrap();
        let io = TokioIo::new(tls_stream);

        let (mut sender, conn) = handshake(io).await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });

        let req = Request::builder()
            .method(method)
            .uri(&full_url) // Use the full URL here
            .header("Host", host) // Add the Host header
            .header("Accept", "application/json")
            .header("APCA-API-KEY-ID", &self.api_config.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_config.secret_key)
            .body(body)?; // Use the provided body

        println!("Request: {:?}", req);
        let res = sender.send_request(req).await?;

        println!("Response: {}", res.status());
        println!("Headers: {:#?}\n", res.headers());

        let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
        // Print the response body for debugging
        println!("Response body: {:?}", String::from_utf8_lossy(&body_bytes));
        let response: T = serde_json::from_slice(&body_bytes)?;

        Ok(response)
    }


}

pub struct AppState {
    pub alpaca_client: Client,
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