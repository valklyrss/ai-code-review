use crate::{config::AppConfig, error::AppResult, gitx::command::run_git, model::repo::RepoConfig};
use std::path::PathBuf;

pub fn repo_local_path(cfg: &AppConfig, repo: &RepoConfig) -> PathBuf {
    let safe = repo.id.replace(['/', '\\', ':'], "_");
    PathBuf::from(&cfg.git.repo_base_dir).join(format!("{safe}.git"))
}

pub async fn ensure_mirror_repo(cfg: &AppConfig, repo: &RepoConfig) -> AppResult<PathBuf> {
    let path = repo_local_path(cfg, repo);
    if !path.exists() {
        run_git(&cfg.git.command_path, &["clone", "--mirror", &repo.repo_url, path.to_string_lossy().as_ref()], None, cfg.scanner.git_command_timeout_seconds).await?;
    }
    Ok(path)
}

pub async fn fetch_mirror_repo(cfg: &AppConfig, repo_path: &std::path::Path) -> AppResult<()> {
    run_git(&cfg.git.command_path, &["fetch", "--prune"], Some(repo_path), cfg.scanner.git_command_timeout_seconds).await?;
    Ok(())
}

