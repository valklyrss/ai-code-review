use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ReviewIssue {
    pub id: String,
    pub task_id: String,
    pub file_id: Option<String>,
    pub file_path: String,
    pub line_no: Option<i64>,
    pub issue_level: String,
    pub issue_type: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub suggestion: Option<String>,
    pub status: Option<String>,
    pub need_email: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIssueStatus {
    pub status: String,
}

