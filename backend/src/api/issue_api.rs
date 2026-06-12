use crate::{api::AppState, error::AppResult, model::issue::{ReviewIssue, UpdateIssueStatus}};
use axum::{extract::{Path, Query, State}, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;

#[derive(Deserialize)]
pub struct IssueQuery {
    page: Option<i64>,
    page_size: Option<i64>,
    level: Option<String>,
    status: Option<String>,
    repo_id: Option<String>,
}

#[derive(Serialize)]
pub struct Page<T> { items: Vec<T>, total: i64 }

pub async fn list_issues(State(state): State<AppState>, Query(q): Query<IssueQuery>) -> AppResult<Json<Page<ReviewIssue>>> {
    let mut sql = "SELECT i.* FROM review_issue i JOIN review_task t ON i.task_id=t.id WHERE 1=1".to_string();
    let mut count_sql = "SELECT COUNT(*) as total FROM review_issue i JOIN review_task t ON i.task_id=t.id WHERE 1=1".to_string();
    if q.level.is_some() { sql.push_str(" AND i.issue_level=?"); count_sql.push_str(" AND i.issue_level=?"); }
    if q.status.is_some() { sql.push_str(" AND i.status=?"); count_sql.push_str(" AND i.status=?"); }
    if q.repo_id.is_some() { sql.push_str(" AND t.repo_id=?"); count_sql.push_str(" AND t.repo_id=?"); }
    sql.push_str(" ORDER BY i.created_at DESC LIMIT ? OFFSET ?");
    let mut values = vec![];
    if let Some(x) = &q.level { values.push(x.clone()); }
    if let Some(x) = &q.status { values.push(x.clone()); }
    if let Some(x) = &q.repo_id { values.push(x.clone()); }
    let mut query = sqlx::query_as::<_, ReviewIssue>(&sql);
    let mut count_query = sqlx::query(&count_sql);
    for v in values {
        query = query.bind(v.clone());
        count_query = count_query.bind(v);
    }
    let page = q.page.unwrap_or(1).max(1);
    let size = q.page_size.unwrap_or(20).clamp(1, 1000);
    let items = query.bind(size).bind((page - 1) * size).fetch_all(&state.db).await?;
    let total: i64 = count_query.fetch_one(&state.db).await?.try_get("total")?;
    Ok(Json(Page { items, total }))
}

pub async fn update_status(State(state): State<AppState>, Path(id): Path<String>, Json(body): Json<UpdateIssueStatus>) -> AppResult<Json<serde_json::Value>> {
    sqlx::query("UPDATE review_issue SET status=? WHERE id=?").bind(body.status).bind(id).execute(&state.db).await?;
    Ok(Json(json!({"ok": true})))
}
