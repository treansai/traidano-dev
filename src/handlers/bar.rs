use crate::base::AppState;
use crate::error::RequestError;
use crate::handlers::rate_limited_request;
use crate::models::bar::Bar;
use axum::body::Body;
use axum::http::Method;
use std::collections::HashMap;

pub async fn get_bars(
    state: &AppState,
    symbols: &[String],
    timeframe: &str,
    limit: usize,
) -> Result<HashMap<String, Vec<Bar>>, RequestError> {
    let symbols_str = symbols.join(",");
    let path = format!("bars/{}?symbols={}&limit={}", timeframe, symbols_str, limit);
    let bars =
        rate_limited_request::<HashMap<String, Vec<Bar>>>(state, Method::GET, &path, Body::empty())
            .await
            .unwrap();
    Ok(bars)
}
