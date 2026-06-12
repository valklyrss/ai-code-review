use crate::{api::AppState, error::AppResult, gitx::{command::ls_remote_heads, mirror}, model::repo::RepoConfig, util::time::now};
use sqlx::Row;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

pub fn start(state: AppState) {
    tokio::spawn(async move {
        loop {
            if let Err(e) = scan_all(&state).await {
                tracing::error!("scanner failed: {e}");
            }
            sleep(Duration::from_secs(state.config.scanner.interval_seconds)).await;
        }
    });
}

pub async fn scan_all(state: &AppState) -> AppResult<()> {
    let repos = sqlx::query_as::<_, RepoConfig>("SELECT * FROM repo_config WHERE enabled=1")
        .fetch_all(&state.db).await?;
    for repo in repos {
        if let Err(e) = scan_repo(state, &repo).await {
            tracing::error!(repo_id=%repo.id, repo_name=%repo.repo_name, "scan repo failed: {e}");
        }
    }
    Ok(())
}

pub async fn scan_repo(state: &AppState, repo: &RepoConfig) -> AppResult<()> {
    let branches = ls_remote_heads(&state.config.git.command_path, &repo.repo_url, state.config.scanner.git_command_timeout_seconds).await?;
    let repo_path = mirror::ensure_mirror_repo(&state.config, repo).await?;
    mirror::fetch_mirror_repo(&state.config, &repo_path).await?;
    for branch in branches.into_iter().filter(|b| branch_matches(repo.branch_pattern.as_deref().unwrap_or("*"), &b.branch_name)) {
        let row = sqlx::query("SELECT id,last_commit_id FROM repo_branch_state WHERE repo_id=? AND branch_name=?")
            .bind(&repo.id).bind(&branch.branch_name).fetch_optional(&state.db).await?;
        let t = now();
        if let Some(row) = row {
            let old: Option<String> = row.try_get("last_commit_id")?;
            if old.as_deref() == Some(branch.commit_id.as_str()) {
                sqlx::query("UPDATE repo_branch_state SET last_scan_time=?,updated_at=? WHERE repo_id=? AND branch_name=?")
                    .bind(&t).bind(&t).bind(&repo.id).bind(&branch.branch_name).execute(&state.db).await?;
                continue;
            }
            if let Some(old_commit) = old {
                let task_id = Uuid::new_v4().to_string();
                sqlx::query("INSERT INTO review_task(id,repo_id,repo_name,branch_name,old_commit_id,new_commit_id,status,created_at) VALUES(?,?,?,?,?,?,?,?)")
                    .bind(&task_id).bind(&repo.id).bind(&repo.repo_name).bind(&branch.branch_name).bind(&old_commit).bind(&branch.commit_id).bind("WAITING").bind(&t)
                    .execute(&state.db).await?;
            }
            sqlx::query("UPDATE repo_branch_state SET last_commit_id=?,last_scan_time=?,updated_at=? WHERE repo_id=? AND branch_name=?")
                .bind(&branch.commit_id).bind(&t).bind(&t).bind(&repo.id).bind(&branch.branch_name).execute(&state.db).await?;
        } else {
            sqlx::query("INSERT INTO repo_branch_state(id,repo_id,branch_name,last_commit_id,last_scan_time,updated_at) VALUES(?,?,?,?,?,?)")
                .bind(Uuid::new_v4().to_string()).bind(&repo.id).bind(&branch.branch_name).bind(&branch.commit_id).bind(&t).bind(&t)
                .execute(&state.db).await?;
        }
    }
    Ok(())
}

fn branch_matches(pattern: &str, branch: &str) -> bool {
    pattern.split(',').map(str::trim).any(|p| p == "*" || p == branch || (p.ends_with('*') && branch.starts_with(&p[..p.len() - 1])))
}
