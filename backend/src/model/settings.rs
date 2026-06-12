use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScannerSetting {
    pub id: String,
    pub interval_seconds: i64,
    pub max_concurrent_tasks: i64,
    pub max_diff_lines: i64,
    pub max_file_diff_lines: i64,
    pub git_command_timeout_seconds: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerSettingInput {
    pub interval_seconds: i64,
    pub max_concurrent_tasks: i64,
    pub max_diff_lines: i64,
    pub max_file_diff_lines: i64,
    pub git_command_timeout_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MailSetting {
    pub id: String,
    pub enabled: i64,
    pub smtp_host: Option<String>,
    pub smtp_port: i64,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_addr: String,
    pub use_tls: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailSettingInput {
    pub enabled: bool,
    pub smtp_host: Option<String>,
    pub smtp_port: i64,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_addr: String,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReviewSettingRow {
    pub id: String,
    pub default_prompt_name: String,
    pub serious_levels: String,
    pub allowed_extensions: String,
    pub ignore_paths: String,
    pub ignore_extensions: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSetting {
    pub id: String,
    pub default_prompt_name: String,
    pub serious_levels: Vec<String>,
    pub allowed_extensions: Vec<String>,
    pub ignore_paths: Vec<String>,
    pub ignore_extensions: Vec<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSettingInput {
    pub default_prompt_name: String,
    pub serious_levels: Vec<String>,
    pub allowed_extensions: Vec<String>,
    pub ignore_paths: Vec<String>,
    pub ignore_extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MailSettingPublic {
    pub id: String,
    pub enabled: i64,
    pub smtp_host: Option<String>,
    pub smtp_port: i64,
    pub username: Option<String>,
    pub password_masked: String,
    pub from_addr: String,
    pub use_tls: i64,
    pub updated_at: String,
}

