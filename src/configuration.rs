use std::collections::HashMap;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BaseConfig {
    #[serde(rename = "api-config")]
    pub api_config: ApiConfig
}

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub base_url : String,
    pub api_key : String,
    pub secret : String
}

/// Build the configuration of the api
pub fn build_config() -> Result<BaseConfig, ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    tracing::event!(tracing::Level::INFO,"{}", base_path.to_str().unwrap());

    let conf_dir = base_path.join("conf");
    let settings = Config::builder()
        .add_source(File::from(
            conf_dir.join("config.yaml"),
        ))
        .build()?;

    let conf = settings.try_deserialize::<BaseConfig>()?;
    Ok(conf)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_config(){
        let config = ApiConfig{ base_url: "".to_string(), api_key: "api_key".to_string(), secret: "secret_key".to_string() };
        assert_eq!(config.api_key, "api_key")
    }

    #[test]
    fn read_config_from_yaml() {
        let config = build_config();
        assert_eq!(config.expect("error in config building").api_config.api_key, "api_key")
    }
}

