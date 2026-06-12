use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct RepoConfig {
    pub id: String,
    pub repo_name: String,
    pub repo_url: String,
    pub auth_type: String,
    pub username: Option<String>,
    pub access_token: Option<String>,
    pub branch_pattern: Option<String>,
    pub scan_interval_seconds: Option<i64>,
    pub enabled: i64,
    pub owner_email: Option<String>,
    pub sync_status: String,
    pub sync_progress: i64,
    pub sync_message: Option<String>,
    pub sync_started_at: Option<String>,
    pub sync_finished_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RepoInput {
    pub repo_name: String,
    pub repo_url: String,
    pub auth_type: String,
    pub username: Option<String>,
    pub access_token: Option<String>,
    pub branch_pattern: Option<String>,
    pub scan_interval_seconds: Option<i64>,
    pub enabled: Option<bool>,
    pub owner_email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RemoteBranch {
    pub branch_name: String,
    pub commit_id: String,
}
