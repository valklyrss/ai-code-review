use crate::{api::AppState, error::AppResult, gitx::command::ls_remote_heads, model::repo::{RepoConfig, RepoInput}, scanner::scheduler, util::time::now};
use axum::{extract::{Path, State}, Json};
use serde_json::json;
use uuid::Uuid;

pub async fn list_repos(State(state): State<AppState>) -> AppResult<Json<Vec<RepoConfig>>> {
    let repos = sqlx::query_as::<_, RepoConfig>("SELECT * FROM repo_config ORDER BY created_at DESC").fetch_all(&state.db).await?;
    Ok(Json(repos))
}

pub async fn create_repo(State(state): State<AppState>, Json(input): Json<RepoInput>) -> AppResult<Json<RepoConfig>> {
    let id = Uuid::new_v4().to_string();
    let t = now();
    sqlx::query("INSERT INTO repo_config(id,repo_name,repo_url,auth_type,username,access_token,branch_pattern,scan_interval_seconds,enabled,owner_email,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(&id).bind(&input.repo_name).bind(&input.repo_url).bind(&input.auth_type).bind(&input.username).bind(&input.access_token)
        .bind(input.branch_pattern.as_deref().unwrap_or("*")).bind(input.scan_interval_seconds.unwrap_or(60))
        .bind(if input.enabled.unwrap_or(true) {1} else {0}).bind(&input.owner_email).bind(&t).bind(&t).execute(&state.db).await?;
    get_repo(&state, &id).await.map(Json)
}

pub async fn update_repo(State(state): State<AppState>, Path(id): Path<String>, Json(input): Json<RepoInput>) -> AppResult<Json<RepoConfig>> {
    sqlx::query("UPDATE repo_config SET repo_name=?,repo_url=?,auth_type=?,username=?,access_token=?,branch_pattern=?,scan_interval_seconds=?,enabled=?,owner_email=?,updated_at=? WHERE id=?")
        .bind(&input.repo_name).bind(&input.repo_url).bind(&input.auth_type).bind(&input.username).bind(&input.access_token)
        .bind(input.branch_pattern.as_deref().unwrap_or("*")).bind(input.scan_interval_seconds.unwrap_or(60))
        .bind(if input.enabled.unwrap_or(true) {1} else {0}).bind(&input.owner_email).bind(now()).bind(&id).execute(&state.db).await?;
    get_repo(&state, &id).await.map(Json)
}

pub async fn delete_repo(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM repo_config WHERE id=?").bind(&id).execute(&state.db).await?;
    Ok(Json(json!({"ok": true})))
}

pub async fn test_repo(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<serde_json::Value>> {
    let repo = get_repo(&state, &id).await?;
    let branches = ls_remote_heads(&state.config.git.command_path, &repo.repo_url, state.config.scanner.git_command_timeout_seconds).await?;
    Ok(Json(json!({"ok": true, "branches": branches.into_iter().map(|b| json!({"branch_name": b.branch_name, "commit_id": b.commit_id})).collect::<Vec<_>>()})))
}

pub async fn scan_repo_now(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<serde_json::Value>> {
    let repo = get_repo(&state, &id).await?;
    scheduler::scan_repo(&state, &repo).await?;
    Ok(Json(json!({"ok": true})))
}

async fn get_repo(state: &AppState, id: &str) -> AppResult<RepoConfig> {
    Ok(sqlx::query_as::<_, RepoConfig>("SELECT * FROM repo_config WHERE id=?").bind(id).fetch_one(&state.db).await?)
}

