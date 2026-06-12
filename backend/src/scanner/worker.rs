use crate::{
    api::AppState,
    error::{AppError, AppResult},
    gitx::{command, mirror},
    mail,
    model::{commit::ReviewCommit, issue::ReviewIssue, repo::RepoConfig, task::ReviewTask},
    review::ai_client::{AiReviewer, OpenAiCompatibleClient},
    util::time::now,
};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

pub fn start(state: AppState) {
    let workers = state.config.scanner.max_concurrent_tasks.max(1);
    for _ in 0..workers {
        let s = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = run_once(&s).await {
                    tracing::error!("worker tick failed: {e}");
                }
                sleep(Duration::from_secs(5)).await;
            }
        });
    }
}

pub async fn run_once(state: &AppState) -> AppResult<()> {
    let task = sqlx::query_as::<_, ReviewTask>("SELECT * FROM review_task WHERE status='WAITING' ORDER BY created_at LIMIT 1")
        .fetch_optional(&state.db).await?;
    if let Some(task) = task {
        if let Err(e) = execute_task(state, &task).await {
            let t = now();
            sqlx::query("UPDATE review_task SET status='FAILED',error_msg=?,finished_at=? WHERE id=?")
                .bind(e.to_string()).bind(t).bind(&task.id).execute(&state.db).await?;
        }
    }
    Ok(())
}

pub async fn execute_task(state: &AppState, task: &ReviewTask) -> AppResult<()> {
    let started = now();
    sqlx::query("UPDATE review_task SET status='RUNNING',started_at=? WHERE id=?").bind(&started).bind(&task.id).execute(&state.db).await?;
    let repo = sqlx::query_as::<_, RepoConfig>("SELECT * FROM repo_config WHERE id=?").bind(&task.repo_id).fetch_one(&state.db).await?;
    let repo_path = mirror::ensure_mirror_repo(&state.config, &repo).await?;
    mirror::fetch_mirror_repo(&state.config, &repo_path).await?;

    let old_commit = task.old_commit_id.clone().ok_or_else(|| AppError::Other("task old_commit_id is empty".into()))?;
    let mut base = old_commit.clone();
    let mut note = None;
    if !command::is_ancestor(&state.config.git.command_path, &repo_path, &old_commit, &task.new_commit_id, state.config.scanner.git_command_timeout_seconds).await? {
        base = command::merge_base(&state.config.git.command_path, &repo_path, &old_commit, &task.new_commit_id, state.config.scanner.git_command_timeout_seconds).await?;
        note = Some(format!("detected force push, using merge-base {base}"));
    }

    let commits = command::log_between(&state.config.git.command_path, &repo_path, &base, &task.new_commit_id, state.config.scanner.git_command_timeout_seconds).await?;
    for c in &commits {
        sqlx::query("INSERT INTO review_commit(id,task_id,commit_id,author_name,author_email,commit_msg,commit_time) VALUES(?,?,?,?,?,?,?)")
            .bind(Uuid::new_v4().to_string()).bind(&task.id).bind(&c.commit_id).bind(&c.author_name).bind(&c.author_email).bind(&c.commit_msg).bind(&c.commit_time).execute(&state.db).await?;
    }

    let files = command::diff_name_status(&state.config.git.command_path, &repo_path, &base, &task.new_commit_id, state.config.scanner.git_command_timeout_seconds).await?;
    let ai = OpenAiCompatibleClient::new(state.config.clone())?;
    let mut total_diff_lines = 0usize;
    let mut issue_count = 0i64;
    let mut high_count = 0i64;
    let mut critical_count = 0i64;
    let mut worst = "INFO".to_string();

    for (change_type, file_path) in files.iter() {
        let (skip, reason) = should_skip(state, file_path);
        let mut diff = String::new();
        let mut skipped = skip;
        let mut skip_reason = reason;
        if !skipped {
            diff = command::diff_file(&state.config.git.command_path, &repo_path, &base, &task.new_commit_id, file_path, state.config.scanner.git_command_timeout_seconds).await?;
            let lines = diff.lines().count();
            if lines > state.config.scanner.max_file_diff_lines {
                skipped = true;
                skip_reason = Some("DIFF_TOO_LARGE".into());
            } else if total_diff_lines + lines > state.config.scanner.max_diff_lines {
                skipped = true;
                skip_reason = Some("TASK_DIFF_TOO_LARGE".into());
            } else {
                total_diff_lines += lines;
            }
        }
        let file_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO review_file(id,task_id,file_path,change_type,diff_content,skipped,skip_reason) VALUES(?,?,?,?,?,?,?)")
            .bind(&file_id).bind(&task.id).bind(file_path).bind(change_type).bind(if diff.is_empty() { None } else { Some(diff.as_str()) }).bind(if skipped {1} else {0}).bind(&skip_reason)
            .execute(&state.db).await?;
        if skipped { continue; }
        let result = ai.review_file(file_path, &diff).await?;
        tracing::debug!(summary=?result.summary, file_path=%file_path, "ai review parsed");
        for issue in result.issues {
            let level = issue.level.to_uppercase();
            if level == "HIGH" { high_count += 1; }
            if level == "CRITICAL" { critical_count += 1; }
            if rank(&level) > rank(&worst) { worst = level.clone(); }
            issue_count += 1;
            sqlx::query("INSERT INTO review_issue(id,task_id,file_id,file_path,line_no,issue_level,issue_type,title,description,suggestion,status,need_email,created_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)")
                .bind(Uuid::new_v4().to_string()).bind(&task.id).bind(&file_id).bind(file_path).bind(issue.line).bind(&level).bind(issue.issue_type).bind(issue.title).bind(issue.description).bind(issue.suggestion).bind("TODO").bind(if issue.need_email {1} else {0}).bind(now())
                .execute(&state.db).await?;
        }
    }

    let result = if critical_count > 0 || high_count > 0 { "FAIL" } else if issue_count > 0 { "WARN" } else { "PASS" };
    if issue_count == 0 { worst = "INFO".into(); }
    let finished = now();
    sqlx::query("UPDATE review_task SET status='SUCCESS',result=?,risk_level=?,commit_count=?,file_count=?,issue_count=?,high_count=?,critical_count=?,error_msg=?,finished_at=? WHERE id=?")
        .bind(result).bind(&worst).bind(commits.len() as i64).bind(files.len() as i64).bind(issue_count).bind(high_count).bind(critical_count).bind(note).bind(&finished).bind(&task.id)
        .execute(&state.db).await?;

    if high_count > 0 || critical_count > 0 {
        let updated = sqlx::query_as::<_, ReviewTask>("SELECT * FROM review_task WHERE id=?").bind(&task.id).fetch_one(&state.db).await?;
        let db_commits = sqlx::query_as::<_, ReviewCommit>("SELECT * FROM review_commit WHERE task_id=?").bind(&task.id).fetch_all(&state.db).await?;
        let issues = sqlx::query_as::<_, ReviewIssue>("SELECT * FROM review_issue WHERE task_id=? AND issue_level IN ('HIGH','CRITICAL')").bind(&task.id).fetch_all(&state.db).await?;
        match mail::send_alert(&state.config, &updated, &db_commits, &issues, repo.owner_email.as_deref()).await {
            Ok(true) => { sqlx::query("UPDATE review_task SET email_sent=1 WHERE id=?").bind(&task.id).execute(&state.db).await?; }
            Ok(false) => {}
            Err(e) => tracing::error!("send alert email failed: {e}"),
        }
    }
    Ok(())
}

fn should_skip(state: &AppState, path: &str) -> (bool, Option<String>) {
    if state.config.review.ignore_paths.iter().any(|p| path.contains(p)) {
        return (true, Some("IGNORE_PATH".into()));
    }
    if state.config.review.ignore_extensions.iter().any(|e| path.ends_with(e)) {
        return (true, Some("IGNORE_EXTENSION".into()));
    }
    if !state.config.review.allowed_extensions.iter().any(|e| path.ends_with(e)) {
        return (true, Some("EXTENSION_NOT_ALLOWED".into()));
    }
    (false, None)
}

fn rank(level: &str) -> i32 {
    match level {
        "CRITICAL" => 5,
        "HIGH" => 4,
        "MEDIUM" => 3,
        "LOW" => 2,
        _ => 1,
    }
}
