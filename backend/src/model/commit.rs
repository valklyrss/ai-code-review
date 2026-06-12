use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ReviewCommit {
    pub id: String,
    pub task_id: String,
    pub commit_id: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub commit_msg: Option<String>,
    pub commit_time: Option<String>,
}
