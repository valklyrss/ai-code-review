use crate::{config::AppConfig, error::{AppError, AppResult}, review::{parser::{parse_ai_json, AiReviewResult}, prompt}};
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
    http: Client,
}

impl OpenAiCompatibleClient {
    pub fn new(cfg: AppConfig) -> AppResult<Self> {
        let http = Client::builder().timeout(Duration::from_secs(cfg.ai.timeout_seconds)).build()
            .map_err(|e| AppError::Ai(format!("failed to build ai http client: {e}")))?;
        Ok(Self { cfg, http })
    }
}

#[async_trait]
impl AiReviewer for OpenAiCompatibleClient {
    async fn review_file(&self, file_path: &str, diff_content: &str) -> AppResult<AiReviewResult> {
        if self.cfg.ai.api_key.trim().is_empty() {
            return Ok(AiReviewResult { summary: Some("AI api_key 为空，已跳过实际审核".into()), issues: vec![] });
        }
        let url = format!("{}/chat/completions", self.cfg.ai.base_url.trim_end_matches('/'));
        let body = json!({
            "model": self.cfg.ai.model,
            "messages": [
                {"role": "system", "content": prompt::system_prompt(&self.cfg.review.default_prompt_name)},
                {"role": "user", "content": prompt::user_prompt(file_path, diff_content)}
            ],
            "temperature": self.cfg.ai.temperature,
            "max_tokens": self.cfg.ai.max_tokens
        });
        let resp: serde_json::Value = self.http.post(url)
            .bearer_auth(&self.cfg.ai.api_key)
            .json(&body)
            .send().await.map_err(|e| AppError::Ai(format!("ai request failed: {e}")))?
            .error_for_status().map_err(|e| AppError::Ai(format!("ai http status error: {e}")))?
            .json().await.map_err(|e| AppError::Ai(format!("ai response json error: {e}")))?;
        let content = resp["choices"][0]["message"]["content"].as_str().unwrap_or("");
        parse_ai_json(content).map_err(|e| {
            let snippet: String = content.chars().take(500).collect();
            AppError::Ai(format!("failed to parse ai json: {e}; raw snippet: {snippet}"))
        })
    }
}

