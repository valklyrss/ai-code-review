use crate::error::AppResult;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn connect(database_url: &str) -> AppResult<SqlitePool> {
    if database_url.starts_with("sqlite:") {
        let path = database_url.trim_start_matches("sqlite:");
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        if !std::path::Path::new(path).exists() {
            std::fs::File::create(path)?;
        }
    }
    let pool = SqlitePoolOptions::new().max_connections(5).connect(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await.map_err(|e| crate::error::AppError::Other(format!("migration error: {e}")))?;
    Ok(pool)
}

