use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ReviewTask {
    pub id: String,
    pub repo_id: String,
    pub repo_name: String,
    pub branch_name: String,
    pub old_commit_id: Option<String>,
    pub new_commit_id: String,
    pub status: String,
    pub result: Option<String>,
    pub risk_level: Option<String>,
    pub commit_count: i64,
    pub file_count: i64,
    pub issue_count: i64,
    pub high_count: i64,
    pub critical_count: i64,
    pub email_sent: i64,
    pub error_msg: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ReviewFile {
    pub id: String,
    pub task_id: String,
    pub file_path: String,
    pub change_type: Option<String>,
    pub additions: i64,
    pub deletions: i64,
    pub diff_content: Option<String>,
    pub skipped: i64,
    pub skip_reason: Option<String>,
}

