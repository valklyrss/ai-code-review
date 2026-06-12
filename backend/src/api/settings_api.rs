use crate::{
    api::AppState,
    error::{AppError, AppResult},
    model::settings::{
        MailSetting, MailSettingInput, MailSettingPublic, ReviewSetting, ReviewSettingInput,
        ReviewSettingRow, ScannerSetting, ScannerSettingInput,
    },
    util::{mask::mask_secret, time::now},
};
use axum::{extract::State, Json};
use serde_json::json;

const SETTINGS_ID: &str = "default";

pub async fn get_settings(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(json!({
        "scanner": get_scanner_setting(&state).await?,
        "mail": to_mail_public(get_mail_setting(&state).await?),
        "review": get_review_setting(&state).await?
    })))
}

pub async fn save_scanner(State(state): State<AppState>, Json(input): Json<ScannerSettingInput>) -> AppResult<Json<serde_json::Value>> {
    let t = now();
    sqlx::query(
        "INSERT INTO scanner_setting(id,interval_seconds,max_concurrent_tasks,max_diff_lines,max_file_diff_lines,git_command_timeout_seconds,updated_at)
         VALUES(?,?,?,?,?,?,?)
         ON CONFLICT(id) DO UPDATE SET interval_seconds=excluded.interval_seconds,max_concurrent_tasks=excluded.max_concurrent_tasks,
         max_diff_lines=excluded.max_diff_lines,max_file_diff_lines=excluded.max_file_diff_lines,git_command_timeout_seconds=excluded.git_command_timeout_seconds,updated_at=excluded.updated_at"
    )
        .bind(SETTINGS_ID)
        .bind(input.interval_seconds.max(5))
        .bind(input.max_concurrent_tasks.max(1))
        .bind(input.max_diff_lines.max(1))
        .bind(input.max_file_diff_lines.max(1))
        .bind(input.git_command_timeout_seconds.max(5))
        .bind(&t)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({"ok": true, "scanner": get_scanner_setting(&state).await?})))
}

pub async fn save_mail(State(state): State<AppState>, Json(input): Json<MailSettingInput>) -> AppResult<Json<serde_json::Value>> {
    let current = get_mail_setting(&state).await.ok();
    let password = match (&current, input.password.clone().unwrap_or_default()) {
        (Some(existing), value) if value.trim().is_empty() || value.trim() == "******" => existing.password.clone(),
        (_, value) => Some(value),
    };
    let t = now();
    sqlx::query(
        "INSERT INTO mail_setting(id,enabled,smtp_host,smtp_port,username,password,from_addr,use_tls,updated_at)
         VALUES(?,?,?,?,?,?,?,?,?)
         ON CONFLICT(id) DO UPDATE SET enabled=excluded.enabled,smtp_host=excluded.smtp_host,smtp_port=excluded.smtp_port,
         username=excluded.username,password=excluded.password,from_addr=excluded.from_addr,use_tls=excluded.use_tls,updated_at=excluded.updated_at"
    )
        .bind(SETTINGS_ID)
        .bind(if input.enabled { 1 } else { 0 })
        .bind(input.smtp_host)
        .bind(input.smtp_port)
        .bind(input.username)
        .bind(password)
        .bind(input.from_addr)
        .bind(if input.use_tls { 1 } else { 0 })
        .bind(&t)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({"ok": true, "mail": to_mail_public(get_mail_setting(&state).await?)})))
}

pub async fn save_review(State(state): State<AppState>, Json(input): Json<ReviewSettingInput>) -> AppResult<Json<serde_json::Value>> {
    let t = now();
    sqlx::query(
        "INSERT INTO review_setting(id,default_prompt_name,serious_levels,allowed_extensions,ignore_paths,ignore_extensions,updated_at)
         VALUES(?,?,?,?,?,?,?)
         ON CONFLICT(id) DO UPDATE SET default_prompt_name=excluded.default_prompt_name,serious_levels=excluded.serious_levels,
         allowed_extensions=excluded.allowed_extensions,ignore_paths=excluded.ignore_paths,ignore_extensions=excluded.ignore_extensions,updated_at=excluded.updated_at"
    )
        .bind(SETTINGS_ID)
        .bind(input.default_prompt_name)
        .bind(serde_json::to_string(&input.serious_levels).map_err(|e| AppError::Other(e.to_string()))?)
        .bind(serde_json::to_string(&input.allowed_extensions).map_err(|e| AppError::Other(e.to_string()))?)
        .bind(serde_json::to_string(&input.ignore_paths).map_err(|e| AppError::Other(e.to_string()))?)
        .bind(serde_json::to_string(&input.ignore_extensions).map_err(|e| AppError::Other(e.to_string()))?)
        .bind(&t)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({"ok": true, "review": get_review_setting(&state).await?})))
}

pub async fn seed_from_config(state: &AppState) -> AppResult<()> {
    let t = now();
    if sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM scanner_setting").fetch_one(&state.db).await? == 0 {
        sqlx::query("INSERT INTO scanner_setting(id,interval_seconds,max_concurrent_tasks,max_diff_lines,max_file_diff_lines,git_command_timeout_seconds,updated_at) VALUES(?,?,?,?,?,?,?)")
            .bind(SETTINGS_ID)
            .bind(state.config.scanner.interval_seconds as i64)
            .bind(state.config.scanner.max_concurrent_tasks as i64)
            .bind(state.config.scanner.max_diff_lines as i64)
            .bind(state.config.scanner.max_file_diff_lines as i64)
            .bind(state.config.scanner.git_command_timeout_seconds as i64)
            .bind(&t)
            .execute(&state.db)
            .await?;
    }
    if sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM mail_setting").fetch_one(&state.db).await? == 0 {
        sqlx::query("INSERT INTO mail_setting(id,enabled,smtp_host,smtp_port,username,password,from_addr,use_tls,updated_at) VALUES(?,?,?,?,?,?,?,?,?)")
            .bind(SETTINGS_ID)
            .bind(if state.config.mail.enabled { 1 } else { 0 })
            .bind(&state.config.mail.smtp_host)
            .bind(state.config.mail.smtp_port as i64)
            .bind(&state.config.mail.username)
            .bind(&state.config.mail.password)
            .bind(&state.config.mail.from)
            .bind(if state.config.mail.use_tls { 1 } else { 0 })
            .bind(&t)
            .execute(&state.db)
            .await?;
    }
    if sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM review_setting").fetch_one(&state.db).await? == 0 {
        sqlx::query("INSERT INTO review_setting(id,default_prompt_name,serious_levels,allowed_extensions,ignore_paths,ignore_extensions,updated_at) VALUES(?,?,?,?,?,?,?)")
            .bind(SETTINGS_ID)
            .bind(&state.config.review.default_prompt_name)
            .bind(serde_json::to_string(&state.config.review.serious_levels).map_err(|e| AppError::Other(e.to_string()))?)
            .bind(serde_json::to_string(&state.config.review.allowed_extensions).map_err(|e| AppError::Other(e.to_string()))?)
            .bind(serde_json::to_string(&state.config.review.ignore_paths).map_err(|e| AppError::Other(e.to_string()))?)
            .bind(serde_json::to_string(&state.config.review.ignore_extensions).map_err(|e| AppError::Other(e.to_string()))?)
            .bind(&t)
            .execute(&state.db)
            .await?;
    }
    Ok(())
}

pub async fn get_scanner_setting(state: &AppState) -> AppResult<ScannerSetting> {
    Ok(sqlx::query_as::<_, ScannerSetting>("SELECT * FROM scanner_setting WHERE id=?")
        .bind(SETTINGS_ID)
        .fetch_one(&state.db)
        .await?)
}

pub async fn get_mail_setting(state: &AppState) -> AppResult<MailSetting> {
    Ok(sqlx::query_as::<_, MailSetting>("SELECT * FROM mail_setting WHERE id=?")
        .bind(SETTINGS_ID)
        .fetch_one(&state.db)
        .await?)
}

pub async fn get_review_setting(state: &AppState) -> AppResult<ReviewSetting> {
    let row = sqlx::query_as::<_, ReviewSettingRow>("SELECT * FROM review_setting WHERE id=?")
        .bind(SETTINGS_ID)
        .fetch_one(&state.db)
        .await?;
    Ok(ReviewSetting {
        id: row.id,
        default_prompt_name: row.default_prompt_name,
        serious_levels: parse_vec(&row.serious_levels)?,
        allowed_extensions: parse_vec(&row.allowed_extensions)?,
        ignore_paths: parse_vec(&row.ignore_paths)?,
        ignore_extensions: parse_vec(&row.ignore_extensions)?,
        updated_at: row.updated_at,
    })
}

fn parse_vec(value: &str) -> AppResult<Vec<String>> {
    serde_json::from_str(value).map_err(|e| AppError::Other(format!("invalid setting json array: {e}")))
}

fn to_mail_public(setting: MailSetting) -> MailSettingPublic {
    MailSettingPublic {
        id: setting.id,
        enabled: setting.enabled,
        smtp_host: setting.smtp_host,
        smtp_port: setting.smtp_port,
        username: setting.username,
        password_masked: mask_secret(setting.password.as_deref().unwrap_or_default()),
        from_addr: setting.from_addr,
        use_tls: setting.use_tls,
        updated_at: setting.updated_at,
    }
}

