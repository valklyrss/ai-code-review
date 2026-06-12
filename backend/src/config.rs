use crate::{error::{AppError, AppResult}, util::mask::mask_secret};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub scanner: ScannerConfig,
    pub git: GitConfig,
    pub ai: AiConfig,
    pub mail: MailConfig,
    pub review: ReviewConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerConfig { pub host: String, pub port: u16 }
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DatabaseConfig { pub url: String }
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScannerConfig {
    pub interval_seconds: u64,
    pub max_concurrent_tasks: usize,
    pub max_diff_lines: usize,
    pub max_file_diff_lines: usize,
    pub git_command_timeout_seconds: u64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GitConfig { pub command_path: String, pub repo_base_dir: String }
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AiConfig {
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub timeout_seconds: u64,
    pub temperature: f32,
    pub max_tokens: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MailConfig {
    pub enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub use_tls: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReviewConfig {
    pub default_prompt_name: String,
    pub serious_levels: Vec<String>,
    pub allowed_extensions: Vec<String>,
    pub ignore_paths: Vec<String>,
    pub ignore_extensions: Vec<String>,
}

impl AppConfig {
    pub fn load() -> AppResult<Self> {
        let args: Vec<String> = env::args().collect();
        let mut path = "./config.yaml".to_string();
        if let Some(i) = args.iter().position(|a| a == "--config") {
            path = args.get(i + 1).ok_or_else(|| AppError::Config("--config requires a path".into()))?.clone();
        }
        let text = fs::read_to_string(&path)
            .map_err(|e| AppError::Config(format!("failed to read config file {}: {}", path, e)))?;
        let cfg: AppConfig = serde_yaml::from_str(&text)
            .map_err(|e| AppError::Config(format!("failed to parse config file {}: {}", path, e)))?;
        Ok(cfg)
    }

    pub fn public_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "server": self.server,
            "database": {"url": self.database.url},
            "scanner": self.scanner,
            "git": self.git,
            "ai": {"provider": self.ai.provider, "base_url": self.ai.base_url, "model": self.ai.model, "timeout_seconds": self.ai.timeout_seconds, "temperature": self.ai.temperature, "max_tokens": self.ai.max_tokens, "api_key": mask_secret(&self.ai.api_key)},
            "mail": {"enabled": self.mail.enabled, "smtp_host": self.mail.smtp_host, "smtp_port": self.mail.smtp_port, "username": self.mail.username, "password": mask_secret(&self.mail.password), "from": self.mail.from, "use_tls": self.mail.use_tls},
            "review": self.review
        })
    }

    pub fn ensure_dirs(&self) -> AppResult<()> {
        if let Some(parent) = Path::new(self.database.url.trim_start_matches("sqlite:")).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(&self.git.repo_base_dir)?;
        Ok(())
    }
}
