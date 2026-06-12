use crate::{api::AppState, error::AppResult};
use axum::{extract::State, Json};
use serde_json::json;

pub async fn health(State(_state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(json!({"status": "UP"})))
}

pub async fn config_summary(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(state.config.public_summary()))
}

