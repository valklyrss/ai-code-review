pub mod issue_api;
pub mod repo_api;
pub mod system_api;
pub mod task_api;

use crate::config::AppConfig;
use axum::{routing::{get, post}, Router};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: SqlitePool,
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/repos", get(repo_api::list_repos).post(repo_api::create_repo))
        .route("/api/repos/:id", get(repo_api::get_repo_detail).put(repo_api::update_repo).delete(repo_api::delete_repo))
        .route("/api/repos/:id/test", post(repo_api::test_repo))
        .route("/api/repos/:id/sync", post(repo_api::sync_repo))
        .route("/api/repos/:id/scan", post(repo_api::scan_repo_now))
        .route("/api/repos/:id/commits", get(repo_api::list_commits))
        .route("/api/repos/:id/commits/:commit_id/scan", post(repo_api::scan_commit))
        .route("/api/tasks", get(task_api::list_tasks))
        .route("/api/tasks/:id", get(task_api::get_task))
        .route("/api/tasks/:id/retry", post(task_api::retry_task))
        .route("/api/issues", get(issue_api::list_issues))
        .route("/api/issues/:id/status", axum::routing::put(issue_api::update_status))
        .route("/api/system/health", get(system_api::health))
        .route("/api/system/config-summary", get(system_api::config_summary))
        .with_state(state)
}
