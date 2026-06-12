use crate::{api::AppState, error::{AppError, AppResult}, gitx::{command::{self, ls_remote_heads}, mirror}, model::{repo::{RepoConfig, RepoInput}, task::ReviewTask}, scanner::scheduler, util::time::now};
use axum::{extract::{Path, Query, State}, Json};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

pub async fn list_repos(State(state): State<AppState>) -> AppResult<Json<Vec<RepoConfig>>> {
    let repos = sqlx::query_as::<_, RepoConfig>("SELECT * FROM repo_config ORDER BY created_at DESC").fetch_all(&state.db).await?;
    Ok(Json(repos))
}

pub async fn get_repo_detail(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<RepoConfig>> {
    Ok(Json(get_repo(&state, &id).await?))
}

pub async fn create_repo(State(state): State<AppState>, Json(input): Json<RepoInput>) -> AppResult<Json<RepoConfig>> {
    let id = Uuid::new_v4().to_string();
    let t = now();
    sqlx::query("INSERT INTO repo_config(id,repo_name,repo_url,auth_type,username,access_token,branch_pattern,scan_interval_seconds,enabled,owner_email,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(&id).bind(&input.repo_name).bind(&input.repo_url).bind(&input.auth_type).bind(&input.username).bind(&input.access_token)
        .bind(input.branch_pattern.as_deref().unwrap_or("*")).bind(input.scan_interval_seconds.unwrap_or(60))
        .bind(if input.enabled.unwrap_or(true) {1} else {0}).bind(&input.owner_email).bind(&t).bind(&t).execute(&state.db).await?;
    start_repo_sync(state.clone(), id.clone());
    get_repo(&state, &id).await.map(Json)
}

pub async fn update_repo(State(state): State<AppState>, Path(id): Path<String>, Json(input): Json<RepoInput>) -> AppResult<Json<RepoConfig>> {
    sqlx::query("UPDATE repo_config SET repo_name=?,repo_url=?,auth_type=?,username=?,access_token=?,branch_pattern=?,scan_interval_seconds=?,enabled=?,owner_email=?,updated_at=? WHERE id=?")
        .bind(&input.repo_name).bind(&input.repo_url).bind(&input.auth_type).bind(&input.username).bind(&input.access_token)
        .bind(input.branch_pattern.as_deref().unwrap_or("*")).bind(input.scan_interval_seconds.unwrap_or(60))
        .bind(if input.enabled.unwrap_or(true) {1} else {0}).bind(&input.owner_email).bind(now()).bind(&id).execute(&state.db).await?;
    start_repo_sync(state.clone(), id.clone());
    get_repo(&state, &id).await.map(Json)
}

pub async fn delete_repo(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM repo_config WHERE id=?").bind(&id).execute(&state.db).await?;
    Ok(Json(json!({"ok": true})))
}

pub async fn test_repo(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<serde_json::Value>> {
    let repo = get_repo(&state, &id).await?;
    ensure_not_syncing(&repo)?;
    let branches = ls_remote_heads(&state.config.git.command_path, &repo.repo_url, state.config.scanner.git_command_timeout_seconds).await?;
    Ok(Json(json!({"ok": true, "branches": branches.into_iter().map(|b| json!({"branch_name": b.branch_name, "commit_id": b.commit_id})).collect::<Vec<_>>()})))
}

pub async fn sync_repo(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<serde_json::Value>> {
    let repo = get_repo(&state, &id).await?;
    ensure_not_syncing(&repo)?;
    start_repo_sync(state, id);
    Ok(Json(json!({"ok": true})))
}

pub async fn scan_repo_now(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<serde_json::Value>> {
    let repo = get_repo(&state, &id).await?;
    ensure_not_syncing(&repo)?;
    scheduler::scan_repo(&state, &repo).await?;
    Ok(Json(json!({"ok": true})))
}

#[derive(Debug, Deserialize)]
pub struct CommitQuery {
    branch: Option<String>,
    limit: Option<usize>,
}

pub async fn list_commits(State(state): State<AppState>, Path(id): Path<String>, Query(q): Query<CommitQuery>) -> AppResult<Json<serde_json::Value>> {
    let repo = get_repo(&state, &id).await?;
    ensure_synced(&repo)?;
    let repo_path = mirror::repo_local_path(&state.config, &repo);
    if !repo_path.exists() {
        return Err(AppError::BadRequest("本地仓库不存在，请先拉取仓库".into()));
    }
    let branches = command::local_heads(&state.config.git.command_path, &repo_path, state.config.scanner.git_command_timeout_seconds).await?;
    let branch = q.branch.or_else(|| branches.first().map(|b| b.branch_name.clone())).unwrap_or_default();
    let commits = if branch.is_empty() {
        vec![]
    } else {
        command::log_graph(&state.config.git.command_path, &repo_path, &branch, q.limit.unwrap_or(100), state.config.scanner.git_command_timeout_seconds).await?
    };
    tracing::info!(
        repo_id=%repo.id,
        repo_name=%repo.repo_name,
        branch=%branch,
        branch_count=branches.len(),
        commit_count=commits.len(),
        "loaded repository commit graph"
    );
    let commit_ids: Vec<String> = commits.iter().map(|c| c.commit_id.clone()).collect();
    let tasks = if commit_ids.is_empty() {
        vec![]
    } else {
        let placeholders = std::iter::repeat("?").take(commit_ids.len()).collect::<Vec<_>>().join(",");
        let sql = format!("SELECT * FROM review_task WHERE repo_id=? AND new_commit_id IN ({placeholders}) ORDER BY created_at DESC");
        let mut query = sqlx::query_as::<_, ReviewTask>(&sql).bind(&repo.id);
        for commit_id in &commit_ids {
            query = query.bind(commit_id);
        }
        query.fetch_all(&state.db).await?
    };
    Ok(Json(json!({"repo": repo, "branches": branches, "selected_branch": branch, "commits": commits, "tasks": tasks})))
}

#[derive(Debug, Deserialize)]
pub struct ScanCommitInput {
    branch: Option<String>,
}

pub async fn scan_commit(
    State(state): State<AppState>,
    Path((id, commit_id)): Path<(String, String)>,
    Json(input): Json<ScanCommitInput>,
) -> AppResult<Json<serde_json::Value>> {
    let repo = get_repo(&state, &id).await?;
    ensure_synced(&repo)?;
    let repo_path = mirror::repo_local_path(&state.config, &repo);
    let parent_count = command::parent_count(&state.config.git.command_path, &repo_path, &commit_id, state.config.scanner.git_command_timeout_seconds).await?;
    if parent_count > 1 {
        return Err(AppError::BadRequest("Merge commit 不需要单独扫描，请扫描具体业务提交".into()));
    }
    let parent = command::first_parent(&state.config.git.command_path, &repo_path, &commit_id, state.config.scanner.git_command_timeout_seconds).await?
        .ok_or_else(|| AppError::BadRequest("根提交没有父提交，V1 暂不支持直接审核根提交".into()))?;
    let branch_name = input.branch.unwrap_or_else(|| "manual".to_string());
    let task_id = Uuid::new_v4().to_string();
    tracing::info!(
        repo_id=%repo.id,
        repo_name=%repo.repo_name,
        branch=%branch_name,
        old_commit=%parent,
        new_commit=%commit_id,
        task_id=%task_id,
        "created manual commit review task"
    );
    sqlx::query("INSERT INTO review_task(id,repo_id,repo_name,branch_name,old_commit_id,new_commit_id,status,created_at) VALUES(?,?,?,?,?,?,?,?)")
        .bind(&task_id)
        .bind(&repo.id)
        .bind(&repo.repo_name)
        .bind(&branch_name)
        .bind(&parent)
        .bind(&commit_id)
        .bind("WAITING")
        .bind(now())
        .execute(&state.db)
        .await?;
    Ok(Json(json!({"ok": true, "task_id": task_id})))
}

async fn get_repo(state: &AppState, id: &str) -> AppResult<RepoConfig> {
    Ok(sqlx::query_as::<_, RepoConfig>("SELECT * FROM repo_config WHERE id=?").bind(id).fetch_one(&state.db).await?)
}

fn ensure_not_syncing(repo: &RepoConfig) -> AppResult<()> {
    if repo.sync_status == "SYNCING" || repo.sync_status == "PENDING" {
        return Err(AppError::BadRequest("仓库正在拉取中，请等待完成后再操作".into()));
    }
    Ok(())
}

fn ensure_synced(repo: &RepoConfig) -> AppResult<()> {
    ensure_not_syncing(repo)?;
    if repo.sync_status != "SUCCESS" {
        return Err(AppError::BadRequest("仓库尚未拉取完成，请先拉取仓库".into()));
    }
    Ok(())
}

fn start_repo_sync(state: AppState, repo_id: String) {
    tokio::spawn(async move {
        if let Err(e) = sync_repo_mirror(&state, &repo_id).await {
            tracing::error!(repo_id=%repo_id, "repo sync failed: {e}");
            let _ = set_sync_status(&state, &repo_id, "FAILED", 100, &e.to_string(), true).await;
        }
    });
}

async fn sync_repo_mirror(state: &AppState, repo_id: &str) -> AppResult<()> {
    set_sync_status(state, repo_id, "PENDING", 5, "等待拉取仓库", false).await?;
    let repo = get_repo(state, repo_id).await?;
    tracing::info!(repo_id=%repo.id, repo_name=%repo.repo_name, "starting repository mirror sync");

    set_sync_status(state, repo_id, "SYNCING", 20, "正在执行 git clone --mirror / 检查本地 mirror", false).await?;
    let repo_path = mirror::ensure_mirror_repo(&state.config, &repo).await?;
    tracing::info!(repo_id=%repo.id, repo_path=%repo_path.display(), "mirror repository is ready");

    set_sync_status(state, repo_id, "SYNCING", 75, "正在执行 git fetch --prune", false).await?;
    mirror::fetch_mirror_repo(&state.config, &repo_path).await?;

    set_sync_status(state, repo_id, "SUCCESS", 100, "仓库拉取完成", true).await?;
    tracing::info!(repo_id=%repo.id, repo_name=%repo.repo_name, "repository mirror sync finished");
    Ok(())
}

async fn set_sync_status(state: &AppState, repo_id: &str, status: &str, progress: i64, message: &str, finished: bool) -> AppResult<()> {
    let t = now();
    if finished {
        sqlx::query("UPDATE repo_config SET sync_status=?,sync_progress=?,sync_message=?,sync_finished_at=?,updated_at=? WHERE id=?")
            .bind(status).bind(progress).bind(message).bind(&t).bind(&t).bind(repo_id).execute(&state.db).await?;
    } else {
        sqlx::query("UPDATE repo_config SET sync_status=?,sync_progress=?,sync_message=?,sync_started_at=?,sync_finished_at=NULL,updated_at=? WHERE id=?")
            .bind(status).bind(progress).bind(message).bind(&t).bind(&t).bind(repo_id).execute(&state.db).await?;
    }
    Ok(())
}
