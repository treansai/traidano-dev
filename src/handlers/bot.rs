use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use traidano::bot::{Bot, BotConfig};
use crate::base::AppState;

pub async fn create_bot(
    State(state): State<Arc<AppState>>,
    Json(config): Json<BotConfig>
) -> impl IntoResponse {
    let mut bot_manager = state.bot_manager.lock().unwrap();
    bot_manager.create_bot(config.clone(), state.clone()).await;
    (StatusCode::CREATED, Json(config))
}

pub async fn stop_bot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>
) -> impl IntoResponse {
    let mut bot_manager = state.bot_manager.lock().unwrap();
    bot_manager.stop_bot(&id).await;
    StatusCode::OK
}

pub async fn remove_bot(
    State(state): State<Arc<AppState>>,
Path(id): Path<String>
) -> impl IntoResponse {
    let mut bot_manager = state.bot_manager.lock().unwrap();
    bot_manager.remove_bot(&id).await;
    StatusCode::OK
}

pub async fn get_bot(
    State(state): State<Arc<AppState>>,
    Path(id) : Path<String>
) -> impl IntoResponse {
    let manager = state.bot_manager.lock().unwrap();
    let bot = manager.bots.get(&id).unwrap();
    (StatusCode::OK, Json(bot))
}

pub async fn get_bots(
    State(state): State<Arc<AppState>>
) -> impl IntoResponse{
    let manager = state.bot_manager.lock().unwrap();
    let bots = &manager.bots;
    (StatusCode::OK, Json(bots))
}