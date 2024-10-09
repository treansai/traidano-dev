use crate::base::AppState;
use crate::error::RequestError;
use crate::handlers::rate_limited_request;
use crate::models::bar::Bar;
use axum::body::Body;
use axum::http::{Method, StatusCode};
use std::collections::HashMap;
use traidano::RequestType;

pub async fn get_bars(
    state: &AppState,
    symbols: &[String],
    timeframe: &str,
    limit: usize,
    request_type: &str
) -> Result<HashMap<String, Vec<Bar>>, RequestError> {

    let request_type = RequestType::from(request_type);
    match request_type {
        RequestType::Order => {
            tracing::error!("Cannot get bar of historical data from order query type");
            Err(RequestError::ApiError(StatusCode::INTERNAL_SERVER_ERROR))
        }
        _ => {
            let symbols_str = symbols.join(",");
            let path =  match &request_type {
                RequestType::StockData => format!("bars/{}?symbols={}&limit={}", timeframe, symbols_str, limit),
                RequestType::CryptoData => format!("us/bars?symbols={}&timeframe={}&limit={}", symbols_str, timeframe, limit),
                _ => "".to_string()
            };
            let response =
                rate_limited_request::<serde_json::Value>(state, Method::GET, &path, Body::empty(), request_type)
                    .await?;
            // Extract the "bars" field from the response
            let bars_field = response.get("bars").ok_or_else(|| {
                tracing::error!("Missing 'bars' field in response");
                RequestError::ApiError(StatusCode::INTERNAL_SERVER_ERROR)
            })?;


            let res: HashMap<String, Vec<Bar>> = serde_json::from_value(bars_field.clone()).unwrap_or_else(|err| {
                tracing::error!("Cannot deserialize bar value: {}", err);
                HashMap::new()
            });
            Ok(res)
        }
    }
}

