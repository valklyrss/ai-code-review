use crate::{api::AppState, error::AppResult, model::{commit::ReviewCommit, issue::ReviewIssue, task::{ReviewFile, ReviewTask}}, util::time::now};
use axum::{extract::{Path, Query, State}, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
pub struct TaskQuery {
    page: Option<i64>,
    page_size: Option<i64>,
    repo_id: Option<String>,
    branch: Option<String>,
    status: Option<String>,
    result: Option<String>,
    risk_level: Option<String>,
}

#[derive(Serialize)]
pub struct Page<T> { items: Vec<T>, total: i64 }

pub async fn list_tasks(State(state): State<AppState>, Query(q): Query<TaskQuery>) -> AppResult<Json<Page<ReviewTask>>> {
    let mut sql = "SELECT * FROM review_task WHERE 1=1".to_string();
    let mut count_sql = "SELECT COUNT(*) as total FROM review_task WHERE 1=1".to_string();
    append_filters(&mut sql, &q);
    append_filters(&mut count_sql, &q);
    sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
    let page = q.page.unwrap_or(1).max(1);
    let size = q.page_size.unwrap_or(20).clamp(1, 100);
    let mut query = sqlx::query_as::<_, ReviewTask>(&sql);
    let mut count_query = sqlx::query(&count_sql);
    for v in filter_values(&q) {
        query = query.bind(v.clone());
        count_query = count_query.bind(v);
    }
    let items = query.bind(size).bind((page - 1) * size).fetch_all(&state.db).await?;
    let total: i64 = count_query.fetch_one(&state.db).await?.try_get("total")?;
    Ok(Json(Page { items, total }))
}

pub async fn get_task(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<serde_json::Value>> {
    let task = sqlx::query_as::<_, ReviewTask>("SELECT * FROM review_task WHERE id=?").bind(&id).fetch_one(&state.db).await?;
    let commits = sqlx::query_as::<_, ReviewCommit>("SELECT * FROM review_commit WHERE task_id=?").bind(&id).fetch_all(&state.db).await?;
    let files = sqlx::query_as::<_, ReviewFile>("SELECT * FROM review_file WHERE task_id=?").bind(&id).fetch_all(&state.db).await?;
    let issues = sqlx::query_as::<_, ReviewIssue>("SELECT * FROM review_issue WHERE task_id=?").bind(&id).fetch_all(&state.db).await?;
    Ok(Json(json!({"task": task, "commits": commits, "files": files, "issues": issues})))
}

pub async fn retry_task(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<serde_json::Value>> {
    sqlx::query("UPDATE review_task SET status='WAITING',result=NULL,risk_level=NULL,error_msg=NULL,started_at=NULL,finished_at=NULL,created_at=? WHERE id=? AND status='FAILED'")
        .bind(now()).bind(&id).execute(&state.db).await?;
    Ok(Json(json!({"ok": true})))
}

fn append_filters(sql: &mut String, q: &TaskQuery) {
    if q.repo_id.is_some() { sql.push_str(" AND repo_id=?"); }
    if q.branch.is_some() { sql.push_str(" AND branch_name=?"); }
    if q.status.is_some() { sql.push_str(" AND status=?"); }
    if q.result.is_some() { sql.push_str(" AND result=?"); }
    if q.risk_level.is_some() { sql.push_str(" AND risk_level=?"); }
}

fn filter_values(q: &TaskQuery) -> Vec<String> {
    let mut v = vec![];
    if let Some(x) = &q.repo_id { v.push(x.clone()); }
    if let Some(x) = &q.branch { v.push(x.clone()); }
    if let Some(x) = &q.status { v.push(x.clone()); }
    if let Some(x) = &q.result { v.push(x.clone()); }
    if let Some(x) = &q.risk_level { v.push(x.clone()); }
    v
}

use sqlx::Row;

