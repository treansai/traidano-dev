use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::collections::HashMap;
use std::iter::StepBy;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct BaseConfig {
    #[serde(rename = "api-config")]
    pub api_config: ApiConfig,
}

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub stream_url: String,
    pub stock_data_url: String,
    pub crypto_data_url: String,
    pub forex_data_url: Option<String>,
    pub api_key: Option<String>,
    pub secret: Option<String>,
}

/// Build the configuration of the api
pub fn build_config() -> Result<BaseConfig, ConfigError> {
    let base_path = std::env::var("CONF_DIR").unwrap_or(
        std::env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    );
    tracing::event!(tracing::Level::INFO, "{}", base_path);

    let conf_dir = PathBuf::from(base_path).join("conf");
    let settings = Config::builder()
        .add_source(File::from(conf_dir.join("config.yaml")))
        .build()?;

    let conf = settings.try_deserialize::<BaseConfig>()?;
    Ok(conf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_config() {
        let config = ApiConfig {
            base_url: "".to_string(),
            stream_url: "".to_string(),
            stock_data_url: "".to_string(),
            crypto_data_url: "".to_string(),
            forex_data_url: None,
            api_key: Some("api_key".to_string()),
            secret: Some("secret_key".to_string()),
        };
        assert_eq!(config.api_key.unwrap(), "api_key")
    }

    #[test]
    fn read_config_from_yaml() {
        let config = build_config();
        assert_eq!(
            config.expect("error in config building").api_config.api_key,
            Some("api_key".to_string())
        )
    }
}
