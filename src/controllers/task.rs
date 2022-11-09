use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use axum::{Extension, Json};
use serde_json::json;
use sqlx::SqlitePool;

use crate::models::task;

pub async fn all_tasks(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    let sql = "SELECT id, task FROM task ".to_string();

    let task = sqlx::query_as::<_, task::Task>(&sql)
        .fetch_all(&pool)
        .await
        .unwrap();

    (StatusCode::OK, Json(task))
}

pub async fn new_task(
    Json(task): Json<task::NewTask>,
    Extension(pool): Extension<SqlitePool>,
) -> impl IntoResponse {
    let sql = "INSERT INTO task (task) values ($1)";

    let _ = sqlx::query(&sql)
        .bind(&task.task)
        .execute(&pool)
        .await
        .unwrap();

    (StatusCode::OK, Json(task))
}

pub async fn task(
    Path(id): Path<i32>,
    Extension(pool): Extension<SqlitePool>,
) -> impl IntoResponse {
    let sql = "SELECT * FROM task where id=$1".to_string();

    let task: task::Task = sqlx::query_as(&sql)
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();

    (StatusCode::OK, Json(task))
}

pub async fn update_task(
    Path(id): Path<i32>,
    Json(task): Json<task::UpdateTask>,
    Extension(pool): Extension<SqlitePool>,
) -> impl IntoResponse {
    sqlx::query("UPDATE task SET task=$1 WHERE id=$2")
        .bind(&task.task)
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    (StatusCode::OK, Json(task))
}

pub async fn delete_task(
    Path(id): Path<i32>,
    Extension(pool): Extension<SqlitePool>,
) -> impl IntoResponse {
    sqlx::query("DELETE FROM task WHERE id=$1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    (StatusCode::OK, Json(json!({"msg": "Task Deleted"})))
}
