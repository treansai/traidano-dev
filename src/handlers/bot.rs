use crate::base::AppState;
use crate::bot::{Bot, BotConfig, BotInfo};
use crate::error::Error;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

pub async fn create_bot(
    State(state): State<Arc<AppState>>,
    Json(config): Json<BotConfig>,
) -> impl IntoResponse {
    tracing::info!("Create bot request");
    let mut bot_manager = state.bot_manager.lock().await;
    bot_manager.create_bot(config.clone(), state.clone()).await;
    tracing::info!("Bot {} created", config.clone().id);
    (StatusCode::CREATED, Json(config))
}

pub async fn stop_bot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut bot_manager = state.bot_manager.lock().await;
    bot_manager.stop_bot(&id).await;
    StatusCode::OK
}

pub async fn remove_bot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut bot_manager = state.bot_manager.lock().await;
    bot_manager.remove_bot(&id).await;
    StatusCode::OK
}

pub async fn get_bot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let manager = state.bot_manager.lock().await;

    match manager.bots.get(&id) {
        Some(bot) => {
            let bot_info = BotInfo::from(bot);
            Json(bot_info).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Bot not Found").into_response(),
    }
}

pub async fn get_bots(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let manager = state.bot_manager.lock().await;

    let bot_infos: HashMap<String, BotInfo> = manager
        .bots
        .iter()
        .map(|(key, bot)| (key.clone(), BotInfo::from(bot)))
        .collect();

    Json(bot_infos)
}
