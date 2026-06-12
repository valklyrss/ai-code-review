use crate::{
    api::AppState,
    error::AppResult,
    model::ai::{AiSetting, AiSettingInput, AiSettingPublic},
    util::{mask::mask_secret, time::now},
};
use axum::{extract::State, Json};
use serde_json::json;
use uuid::Uuid;

pub async fn get_setting(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let setting = active_setting(&state).await?;
    Ok(Json(json!({
        "setting": setting.map(to_public),
        "presets": presets()
    })))
}

pub async fn save_setting(State(state): State<AppState>, Json(input): Json<AiSettingInput>) -> AppResult<Json<serde_json::Value>> {
    let current = active_setting(&state).await?;
    let t = now();
    let api_key = match (&current, input.api_key.clone().unwrap_or_default()) {
        (Some(existing), value) if value.trim().is_empty() || value.trim() == "******" => existing.api_key.clone(),
        (_, value) => Some(value),
    };

    sqlx::query("UPDATE ai_setting SET enabled=0 WHERE enabled=1").execute(&state.db).await?;

    let id = current.map(|s| s.id).unwrap_or_else(|| Uuid::new_v4().to_string());
    sqlx::query(
        "INSERT INTO ai_setting(id,name,provider,base_url,api_key,model,timeout_seconds,temperature,max_tokens,enabled,created_at,updated_at)
         VALUES(?,?,?,?,?,?,?,?,?,?,?,?)
         ON CONFLICT(id) DO UPDATE SET
         name=excluded.name,provider=excluded.provider,base_url=excluded.base_url,api_key=excluded.api_key,model=excluded.model,
         timeout_seconds=excluded.timeout_seconds,temperature=excluded.temperature,max_tokens=excluded.max_tokens,enabled=excluded.enabled,updated_at=excluded.updated_at"
    )
        .bind(&id)
        .bind(input.name)
        .bind(input.provider)
        .bind(input.base_url)
        .bind(api_key)
        .bind(input.model)
        .bind(input.timeout_seconds)
        .bind(input.temperature)
        .bind(input.max_tokens)
        .bind(if input.enabled.unwrap_or(true) { 1 } else { 0 })
        .bind(&t)
        .bind(&t)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({"ok": true, "setting": active_setting(&state).await?.map(to_public)})))
}

pub async fn seed_from_config(state: &AppState) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ai_setting").fetch_one(&state.db).await?;
    if count > 0 {
        return Ok(());
    }
    let t = now();
    sqlx::query("INSERT INTO ai_setting(id,name,provider,base_url,api_key,model,timeout_seconds,temperature,max_tokens,enabled,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(Uuid::new_v4().to_string())
        .bind("默认配置")
        .bind(&state.config.ai.provider)
        .bind(&state.config.ai.base_url)
        .bind(&state.config.ai.api_key)
        .bind(&state.config.ai.model)
        .bind(state.config.ai.timeout_seconds as i64)
        .bind(state.config.ai.temperature as f64)
        .bind(state.config.ai.max_tokens as i64)
        .bind(1)
        .bind(&t)
        .bind(&t)
        .execute(&state.db)
        .await?;
    Ok(())
}

async fn active_setting(state: &AppState) -> AppResult<Option<AiSetting>> {
    Ok(sqlx::query_as::<_, AiSetting>("SELECT * FROM ai_setting WHERE enabled=1 ORDER BY updated_at DESC LIMIT 1")
        .fetch_optional(&state.db)
        .await?)
}

fn to_public(setting: AiSetting) -> AiSettingPublic {
    AiSettingPublic {
        id: setting.id,
        name: setting.name,
        provider: setting.provider,
        base_url: setting.base_url,
        api_key_masked: mask_secret(setting.api_key.as_deref().unwrap_or_default()),
        model: setting.model,
        timeout_seconds: setting.timeout_seconds,
        temperature: setting.temperature,
        max_tokens: setting.max_tokens,
        enabled: setting.enabled,
        created_at: setting.created_at,
        updated_at: setting.updated_at,
    }
}

fn presets() -> serde_json::Value {
    json!([
        {"name":"DeepSeek","provider":"openai-compatible","base_url":"https://api.deepseek.com/v1","model":"deepseek-chat"},
        {"name":"OpenAI","provider":"openai-compatible","base_url":"https://api.openai.com/v1","model":"gpt-4o-mini"},
        {"name":"通义千问 DashScope","provider":"openai-compatible","base_url":"https://dashscope.aliyuncs.com/compatible-mode/v1","model":"qwen-plus"},
        {"name":"智谱 GLM","provider":"openai-compatible","base_url":"https://open.bigmodel.cn/api/paas/v4","model":"glm-4-flash"},
        {"name":"Moonshot Kimi","provider":"openai-compatible","base_url":"https://api.moonshot.cn/v1","model":"moonshot-v1-8k"},
        {"name":"Ollama 本地","provider":"openai-compatible","base_url":"http://127.0.0.1:11434/v1","model":"qwen2.5-coder:7b"},
        {"name":"通用 OpenAI-compatible","provider":"openai-compatible","base_url":"","model":""}
    ])
}

