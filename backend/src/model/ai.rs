use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AiSetting {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub timeout_seconds: i64,
    pub temperature: f64,
    pub max_tokens: i64,
    pub enabled: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AiSettingInput {
    pub name: String,
    pub provider: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub timeout_seconds: i64,
    pub temperature: f64,
    pub max_tokens: i64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct AiSettingPublic {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub base_url: String,
    pub api_key_masked: String,
    pub model: String,
    pub timeout_seconds: i64,
    pub temperature: f64,
    pub max_tokens: i64,
    pub enabled: i64,
    pub created_at: String,
    pub updated_at: String,
}

