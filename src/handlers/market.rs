use crate::base::AppState;
use axum::extract::State;
use std::sync::Arc;

pub async fn get_positions(State(state): State<Arc<AppState>>) {

    todo!()
}

pub async fn is_market_open(State(state): State<Arc<AppState>>) {
    todo!()
}




