use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
    model::{commit::ReviewCommit, issue::ReviewIssue, settings::MailSetting, task::ReviewTask},
};
use lettre::{message::Mailbox, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::collections::BTreeSet;

pub async fn send_alert(
    cfg: &AppConfig,
    mail: &MailSetting,
    task: &ReviewTask,
    commits: &[ReviewCommit],
    issues: &[ReviewIssue],
    owner_email: Option<&str>,
) -> AppResult<bool> {
    if mail.enabled != 1 {
        return Ok(false);
    }
    let mut recipients = BTreeSet::new();
    for c in commits {
        if let Some(email) = &c.author_email {
            if !email.trim().is_empty() {
                recipients.insert(email.trim().to_string());
            }
        }
    }
    if let Some(email) = owner_email {
        if !email.trim().is_empty() {
            recipients.insert(email.trim().to_string());
        }
    }
    if recipients.is_empty() {
        return Ok(false);
    }
    let subject = format!(
        "【AI代码审核告警】【{}】{}/{} 检测到严重问题",
        task.risk_level.clone().unwrap_or_default(),
        task.repo_name,
        task.branch_name
    );
    let body = format!(
        "仓库: {}\n分支: {}\nold_commit: {}\nnew_commit: {}\n提交人: {}\nHIGH: {}\nCRITICAL: {}\n详情: http://{}:{}/tasks/{}\n\n问题摘要:\n{}",
        task.repo_name,
        task.branch_name,
        task.old_commit_id.clone().unwrap_or_default(),
        task.new_commit_id,
        commits.iter().filter_map(|c| c.author_email.clone()).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>().join(", "),
        task.high_count,
        task.critical_count,
        cfg.server.host,
        cfg.server.port,
        task.id,
        issues.iter().map(|i| format!("- [{}] {} {}", i.issue_level, i.file_path, i.title.clone().unwrap_or_default())).collect::<Vec<_>>().join("\n")
    );
    let mut builder = Message::builder()
        .from(mail.from_addr.parse::<Mailbox>().map_err(|e| AppError::Other(format!("invalid mail.from: {e}")))?)
        .subject(subject);
    for r in recipients {
        builder = builder.to(r.parse::<Mailbox>().map_err(|e| AppError::Other(format!("invalid recipient {r}: {e}")))?);
    }
    let msg = builder.body(body).map_err(|e| AppError::Other(format!("build email failed: {e}")))?;
    let creds = lettre::transport::smtp::authentication::Credentials::new(
        mail.username.clone().unwrap_or_default(),
        mail.password.clone().unwrap_or_default(),
    );
    let smtp_host = mail.smtp_host.clone().unwrap_or_default();
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)
        .map_err(|e| AppError::Other(format!("smtp relay error: {e}")))?
        .port(mail.smtp_port as u16)
        .credentials(creds)
        .build();
    mailer.send(msg).await.map_err(|e| AppError::Other(format!("send email failed: {e}")))?;
    Ok(true)
}

