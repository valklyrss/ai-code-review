use crate::{error::{AppError, AppResult}, model::repo::RemoteBranch, util::mask::mask_url};
use serde::Serialize;
use std::{path::Path, time::Duration};
use tokio::{process::Command, time::timeout};

#[derive(Debug)]
pub struct GitOutput { pub stdout: String }

pub async fn run_git(git_path: &str, args: &[&str], cwd: Option<&Path>, timeout_seconds: u64) -> AppResult<GitOutput> {
    let safe_args: Vec<String> = args.iter().map(|a| mask_url(a)).collect();
    tracing::debug!(?safe_args, "running git");
    let mut cmd = Command::new(git_path);
    cmd.args(args);
    if let Some(cwd) = cwd { cmd.current_dir(cwd); }
    let child = cmd.output();
    let output = timeout(Duration::from_secs(timeout_seconds), child)
        .await
        .map_err(|_| AppError::Git(format!("git command timed out after {}s: {:?}", timeout_seconds, safe_args)))??;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        return Err(AppError::Git(format!("git command failed {:?}: {}", safe_args, mask_url(&stderr))));
    }
    Ok(GitOutput { stdout })
}

pub async fn ls_remote_heads(git_path: &str, repo_url: &str, timeout_seconds: u64) -> AppResult<Vec<RemoteBranch>> {
    let out = run_git(git_path, &["ls-remote", "--heads", repo_url], None, timeout_seconds).await?;
    Ok(parse_remote_heads(&out.stdout))
}

fn parse_remote_heads(text: &str) -> Vec<RemoteBranch> {
    text.lines().filter_map(|line| {
        let (commit, reference) = line.split_once('\t')?;
        let branch_name = reference.strip_prefix("refs/heads/")?.to_string();
        Some(RemoteBranch { branch_name, commit_id: commit.to_string() })
    }).collect()
}

pub async fn diff_name_status(git_path: &str, repo_path: &Path, old_commit: &str, new_commit: &str, timeout_seconds: u64) -> AppResult<Vec<(String, String)>> {
    let range = format!("{old_commit}..{new_commit}");
    let out = run_git(git_path, &["diff", "--name-status", &range], Some(repo_path), timeout_seconds).await?;
    Ok(out.stdout.lines().filter_map(|l| {
        let mut parts = l.split_whitespace();
        let status = parts.next()?.to_string();
        let file = parts.last()?.to_string();
        Some((status, file))
    }).collect())
}

pub async fn diff_file(git_path: &str, repo_path: &Path, old_commit: &str, new_commit: &str, file_path: &str, timeout_seconds: u64) -> AppResult<String> {
    let range = format!("{old_commit}..{new_commit}");
    let out = run_git(git_path, &["diff", "--find-renames", "--unified=80", &range, "--", file_path], Some(repo_path), timeout_seconds).await?;
    Ok(out.stdout)
}

#[derive(Debug, Clone)]
pub struct GitCommit { pub commit_id: String, pub author_name: String, pub author_email: String, pub commit_time: String, pub commit_msg: String }

#[derive(Debug, Clone, Serialize)]
pub struct LocalBranch {
    pub branch_name: String,
    pub commit_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphCommit {
    pub graph: String,
    pub commit_id: String,
    pub short_id: String,
    pub parent_count: usize,
    pub is_merge: bool,
    pub author_name: String,
    pub author_email: String,
    pub commit_time: String,
    pub subject: String,
}

pub async fn log_between(git_path: &str, repo_path: &Path, old_commit: &str, new_commit: &str, timeout_seconds: u64) -> AppResult<Vec<GitCommit>> {
    let range = format!("{old_commit}..{new_commit}");
    let out = run_git(git_path, &["log", "--format=%H%x1f%an%x1f%ae%x1f%ai%x1f%s", &range], Some(repo_path), timeout_seconds).await?;
    Ok(out.stdout.lines().filter_map(|l| {
        let p: Vec<&str> = l.split('\x1f').collect();
        if p.len() < 5 { return None; }
        Some(GitCommit { commit_id: p[0].into(), author_name: p[1].into(), author_email: p[2].into(), commit_time: p[3].into(), commit_msg: p[4].into() })
    }).collect())
}

pub async fn is_ancestor(git_path: &str, repo_path: &Path, old_commit: &str, new_commit: &str, timeout_seconds: u64) -> AppResult<bool> {
    let mut cmd = Command::new(git_path);
    cmd.args(["merge-base", "--is-ancestor", old_commit, new_commit]).current_dir(repo_path);
    let out = timeout(Duration::from_secs(timeout_seconds), cmd.output()).await.map_err(|_| AppError::Git("merge-base timed out".into()))??;
    Ok(out.status.success())
}

pub async fn merge_base(git_path: &str, repo_path: &Path, old_commit: &str, new_commit: &str, timeout_seconds: u64) -> AppResult<String> {
    let out = run_git(git_path, &["merge-base", old_commit, new_commit], Some(repo_path), timeout_seconds).await?;
    Ok(out.stdout.trim().to_string())
}

pub async fn local_heads(git_path: &str, repo_path: &Path, timeout_seconds: u64) -> AppResult<Vec<LocalBranch>> {
    let out = run_git(git_path, &["for-each-ref", "--format=%(refname:strip=2)%09%(objectname)", "refs/heads"], Some(repo_path), timeout_seconds).await?;
    Ok(out.stdout.lines().filter_map(|line| {
        let (branch_name, commit_id) = line.split_once('\t')?;
        Some(LocalBranch { branch_name: branch_name.to_string(), commit_id: commit_id.to_string() })
    }).collect())
}

pub async fn log_graph(git_path: &str, repo_path: &Path, branch: &str, limit: usize, timeout_seconds: u64) -> AppResult<Vec<GraphCommit>> {
    let max_count = format!("--max-count={}", limit.clamp(1, 500));
    let reference = format!("refs/heads/{branch}");
    let out = run_git(
        git_path,
        &["log", "--graph", "--date=iso-strict", "--pretty=format:%H%x1f%h%x1f%P%x1f%an%x1f%ae%x1f%ad%x1f%s", &max_count, &reference],
        Some(repo_path),
        timeout_seconds,
    ).await?;
    Ok(out.stdout.lines().filter_map(parse_graph_commit_line).collect())
}

fn parse_graph_commit_line(line: &str) -> Option<GraphCommit> {
    let hash_start = line.find(|c: char| c.is_ascii_hexdigit())?;
    let graph = line[..hash_start].to_string();
    let rest = &line[hash_start..];
    let parts: Vec<&str> = rest.split('\x1f').collect();
    if parts.len() < 7 { return None; }
    let parent_count = parts[2].split_whitespace().count();
    Some(GraphCommit {
        graph,
        commit_id: parts[0].to_string(),
        short_id: parts[1].to_string(),
        parent_count,
        is_merge: parent_count > 1,
        author_name: parts[3].to_string(),
        author_email: parts[4].to_string(),
        commit_time: parts[5].to_string(),
        subject: parts[6].to_string(),
    })
}

pub async fn parent_count(git_path: &str, repo_path: &Path, commit: &str, timeout_seconds: u64) -> AppResult<usize> {
    let out = run_git(git_path, &["rev-list", "--parents", "-n", "1", commit], Some(repo_path), timeout_seconds).await?;
    Ok(out.stdout.split_whitespace().skip(1).count())
}

pub async fn first_parent(git_path: &str, repo_path: &Path, commit: &str, timeout_seconds: u64) -> AppResult<Option<String>> {
    let out = run_git(git_path, &["rev-list", "--parents", "-n", "1", commit], Some(repo_path), timeout_seconds).await?;
    let parts: Vec<&str> = out.stdout.split_whitespace().collect();
    Ok(parts.get(1).map(|s| s.to_string()))
}
