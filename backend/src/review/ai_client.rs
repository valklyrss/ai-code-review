use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
    model::ai::AiSetting,
    review::{
        parser::{parse_ai_json, AiReviewResult},
        prompt,
    },
};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

#[async_trait]
pub trait AiReviewer: Send + Sync {
    async fn review_file(&self, file_path: &str, diff_content: &str) -> AppResult<AiReviewResult>;
}

#[derive(Clone)]
pub struct OpenAiCompatibleClient {
    cfg: AppConfig,
    ai: EffectiveAiConfig,
    http: Client,
}

#[derive(Clone)]
struct EffectiveAiConfig {
    base_url: String,
    api_key: String,
    model: String,
    timeout_seconds: u64,
    temperature: f32,
    max_tokens: u32,
}

impl OpenAiCompatibleClient {
    pub fn new(cfg: AppConfig) -> AppResult<Self> {
        Self::with_ai(
            cfg.clone(),
            EffectiveAiConfig {
                base_url: cfg.ai.base_url.clone(),
                api_key: cfg.ai.api_key.clone(),
                model: cfg.ai.model.clone(),
                timeout_seconds: cfg.ai.timeout_seconds,
                temperature: cfg.ai.temperature,
                max_tokens: cfg.ai.max_tokens,
            },
        )
    }

    pub async fn from_db(pool: &sqlx::SqlitePool, cfg: AppConfig) -> AppResult<Self> {
        let setting = sqlx::query_as::<_, AiSetting>(
            "SELECT * FROM ai_setting WHERE enabled=1 ORDER BY updated_at DESC LIMIT 1",
        )
        .fetch_optional(pool)
        .await?;

        if let Some(setting) = setting {
            tracing::info!(
                ai_name = %setting.name,
                ai_base_url = %setting.base_url,
                ai_model = %setting.model,
                "using database AI setting"
            );
            return Self::with_ai(
                cfg,
                EffectiveAiConfig {
                    base_url: setting.base_url,
                    api_key: setting.api_key.unwrap_or_default(),
                    model: setting.model,
                    timeout_seconds: setting.timeout_seconds.max(1) as u64,
                    temperature: setting.temperature as f32,
                    max_tokens: setting.max_tokens.max(1) as u32,
                },
            );
        }

        tracing::info!("using YAML AI setting fallback");
        Self::new(cfg)
    }

    fn with_ai(cfg: AppConfig, ai: EffectiveAiConfig) -> AppResult<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(ai.timeout_seconds))
            .build()
            .map_err(|e| AppError::Ai(format!("failed to build ai http client: {e}")))?;
        Ok(Self { cfg, ai, http })
    }
}

#[async_trait]
impl AiReviewer for OpenAiCompatibleClient {
    async fn review_file(&self, file_path: &str, diff_content: &str) -> AppResult<AiReviewResult> {
        if self.ai.api_key.trim().is_empty() {
            return Ok(AiReviewResult {
                summary: Some("AI api_key 为空，已跳过实际审核".into()),
                issues: vec![],
            });
        }

        let url = format!("{}/chat/completions", self.ai.base_url.trim_end_matches('/'));
        let body = json!({
            "model": self.ai.model,
            "messages": [
                {"role": "system", "content": prompt::system_prompt(&self.cfg.review.default_prompt_name)},
                {"role": "user", "content": prompt::user_prompt(file_path, diff_content)}
            ],
            "temperature": self.ai.temperature,
            "max_tokens": self.ai.max_tokens
        });

        let resp: serde_json::Value = self
            .http
            .post(url)
            .bearer_auth(&self.ai.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Ai(format!("ai request failed: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::Ai(format!("ai http status error: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Ai(format!("ai response json error: {e}")))?;

        let content = resp["choices"][0]["message"]["content"].as_str().unwrap_or("");
        parse_ai_json(content).map_err(|e| {
            let snippet: String = content.chars().take(500).collect();
            AppError::Ai(format!("failed to parse ai json: {e}; raw snippet: {snippet}"))
        })
    }
}

