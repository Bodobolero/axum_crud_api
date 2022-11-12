use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use axum::{Extension, Json};
use serde_json::json;
use sqlx::SqlitePool;

use crate::models::task;

/// List all Tasks
///
/// List all Tasks in database
#[utoipa::path(
        get,
        path = "/tasks",
        responses(
            (status = 200, description = "List all tasks successfully", body = [Task])
        )
    )]
pub async fn all_tasks(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    let sql = "SELECT id, task FROM task ".to_string();

    let task = sqlx::query_as::<_, task::Task>(&sql)
        .fetch_all(&pool)
        .await
        .unwrap();

    (StatusCode::OK, Json(task))
}

/// Create new Task
///
/// Tries to create a new Task in the database
#[utoipa::path(
        post,
        path = "/task",
        request_body = NewTask,
        responses(
            (status = 200, description = "Task created successfully", body = Task),
        )
    )]
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

/// Get task by id
///
/// Return task by given id. Return only status 200 on success or 404 if Todo is not found.
#[utoipa::path(
        get,
        path = "/task/{id}",
        responses(
            (status = 200, description = "Task returned successfully"),
            (status = 404, description = "Task not found")
        ),
        params(
            ("id" = i64, Path, description = "Task database id")
        )
    )]
pub async fn task(
    Path(id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
) -> impl IntoResponse {
    let sql = "SELECT * FROM task where id=$1".to_string();

    let result: Result<task::Task, sqlx::Error> =
        sqlx::query_as(&sql).bind(id).fetch_one(&pool).await;
    if let Ok(task) = result {
        return (StatusCode::OK, Json(task));
    }

    tracing::error!(
        "could not find task with id: {:?} error: {:?}",
        id,
        result.err()
    );
    (
        StatusCode::NOT_FOUND,
        Json(task::Task {
            id,
            task: "".to_string(),
        }),
    )
}

/// Update Task with new description by id
///
/// Update Task with id
#[utoipa::path(
        put,
        path = "/task/{id}",
        request_body = UpdateTask,
        responses(
            (status = 200, description = "Task updated successfully"),
            (status = 404, description = "Task was not found"),
        ),
        params(
            ("id" = i64, Path, description = "Todo database id")
        ),
        security(
            (), // <-- make optional authentication
            ("api_key" = [])
        )
    )]
pub async fn update_task(
    Path(id): Path<i64>,
    Json(task): Json<task::UpdateTask>,
    Extension(pool): Extension<SqlitePool>,
) -> impl IntoResponse {
    match sqlx::query("UPDATE task SET task=$1 WHERE id=$2")
        .bind(&task.task)
        .bind(id)
        .execute(&pool)
        .await
    {
        Err(e) => {
            tracing::error!("could not find task with id: {:?} error: {:?}", id, e);
            (StatusCode::NOT_FOUND, Json(task))
        }
        Ok(_) => (StatusCode::OK, Json(task)),
    }
}

/// Delete Task by id
///
/// Delete Task from database by id. Returns either 200 success of 404 with TodoError if Todo is not found.
#[utoipa::path(
        delete,
        path = "/task/{id}",
        responses(
            (status = 200, description = "Task was deleted"),
            (status = 404, description = "Task was not found"),
              ),
        params(
            ("id" = i64, Path, description = "Task database id")
        ),
    )]
pub async fn delete_task(
    Path(id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
) -> impl IntoResponse {
    sqlx::query("DELETE FROM task WHERE id=$1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    (StatusCode::OK, Json(json!({"msg": "Task Deleted"})))
}
