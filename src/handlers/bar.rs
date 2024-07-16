use crate::base::AppState;
use crate::handlers::rate_limited_request;
use axum::body::Body;
use axum::http::Method;
use std::collections::HashMap;

use traidano::models::bar::Bar;

pub async fn get_bars(
    state: &AppState,
    symbols: &[String],
    timeframe: &str,
    limit: usize,
) -> Result<HashMap<String, Vec<Bar>>, Box<dyn std::error::Error>> {
    let symbols_str = symbols.join(",");
    let path = format!("bars/{}?symbols={}&limit={}", timeframe, symbols_str, limit);
    let bars: serde_json::Value =
        rate_limited_request(state, Method::GET, &path, Body::empty()).await?;
    let mut result = HashMap::new();
    for symbol in symbols {
        result.insert(
            symbol.clone(),
            serde_json::from_value(bars[symbol].clone())?,
        );
    }
    Ok(result)
}
