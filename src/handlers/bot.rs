use crate::base::AppState;
use crate::bot::{Bot, BotConfig, BotInfo};
use crate::dao;
use crate::dao::bot::get_all_running_bot;
use crate::error::Error;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use uuid::Uuid;

pub async fn create_bot(
    State(state): State<Arc<AppState>>,
    Json(mut config): Json<BotConfig>,
) -> impl IntoResponse {
    tracing::info!("Create bot request");

    // Generate a new UUID for the bot if not provided
    config.id = Uuid::new_v4().to_string();

    match dao::bot::create_bot(&state.db.clone(), config.clone()).await {
        Ok(bot_id) => {
            tracing::debug!("Bot {} saved to database", bot_id);

            let mut bot_manager = state.bot_manager.lock().await;

            bot_manager.create_bot(config.clone(), state.clone()).await;
            tracing::info!("Bot {} created", bot_id);
            (StatusCode::CREATED, Json(config)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create bot in database: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create bot in database",
            )
                .into_response()
        }
    }
}

pub async fn stop_bot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut bot_manager = state.bot_manager.lock().await;

    let id =  dao::bot::kill_bot(&state.db.clone(), id).await.unwrap_or("Bot not stopped".to_string());

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
    tracing::info!("getting bots");
    let bot_infos: HashMap<String, BotInfo> = manager
        .bots
        .iter()
        .map(|(key, bot)| (key.clone(), BotInfo::from(bot)))
        .collect();

    // len bot
    let n_bots = bot_infos.clone().len();
    tracing::info!("{} bot{} found", n_bots, if n_bots > 0 { "s" } else { "" });

    Json(bot_infos)
}
