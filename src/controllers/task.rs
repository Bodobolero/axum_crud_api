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
            (status = 200, description = "List all tasks successfully", body = [Task]),
            (status = 500, description = "Internal server error when retrieving list of all tasks", body = [Task])
        )
    )]
pub async fn all_tasks(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    let sql = "SELECT id, task FROM task ".to_string();

    let result: Result<Vec<task::Task>, sqlx::Error> =
        sqlx::query_as::<_, task::Task>(&sql).fetch_all(&pool).await;

    match result {
        Ok(tasks) => (StatusCode::OK, Json(tasks)),
        Err(err) => {
            tracing::error!("error retrieving tasks: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Vec::<task::Task>::new()),
            )
        }
    }
}

/// Create new Task
///
/// Tries to create a new Task in the database
#[utoipa::path(
        post,
        path = "/tasks",
        request_body = NewTask,
        responses(
            (status = 201, description = "Task created successfully", body = Task),
            (status = 500, description = "Task could not be created", body = Task),
        )
    )]
pub async fn new_task(
    Json(task): Json<task::NewTask>,
    Extension(pool): Extension<SqlitePool>,
) -> impl IntoResponse {
    // we use "RETURNING" - non-standard SQL syntax (which is supported by sqlite and postgres) to return the new ID created by the database
    // to our caller
    let sql = "INSERT INTO task (task) values ($1) RETURNING *";

    let result: Result<task::Task, sqlx::Error> =
        sqlx::query_as(&sql).bind(&task.task).fetch_one(&pool).await;

    match result {
        Ok(taskwithid) => (
            StatusCode::CREATED,
            [("Location", format!("/tasks/{:?}", taskwithid.id))],
            Json(taskwithid),
        ),
        Err(err) => {
            tracing::error!("could not create task. error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("Location", "/tasks/{id}".to_string())],
                Json(task::Task {
                    id: 0,
                    task: task.task,
                }),
            )
        }
    }
}

/// Get task by id
///
/// Return task by given id. Return only status 200 on success or 404 if Todo is not found.
#[utoipa::path(
        get,
        path = "/tasks/{id}",
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

    match result {
        Ok(task) => (StatusCode::OK, Json(task)),
        Err(err) => {
            tracing::error!("could not find task with id: {:?} error: {:?}", id, err);
            (
                StatusCode::NOT_FOUND,
                Json(task::Task {
                    id,
                    task: "".to_string(),
                }),
            )
        }
    }
}

/// Update Task with new description by id
///
/// Update Task with id
#[utoipa::path(
        put,
        path = "/tasks/{id}",
        request_body = UpdateTask,
        responses(
            (status = 200, description = "Task updated successfully"),
            (status = 404, description = "Task was not found"),
        ),
        params(
            ("id" = i64, Path, description = "Task database id")
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
        Ok(queryresult) => {
            return match queryresult.rows_affected() {
                1 => (StatusCode::OK, Json(task)),
                _ => (StatusCode::NOT_FOUND, Json(task)),
            };
        }
        Err(e) => {
            tracing::error!("could not find task with id: {:?} error: {:?}", id, e);
            (StatusCode::NOT_FOUND, Json(task))
        }
    }
}

/// Delete Task by id
///
/// Delete Task from database by id. Returns either 200 success of 404 with TodoError if Todo is not found.
#[utoipa::path(
        delete,
        path = "/tasks/{id}",
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
    match sqlx::query("DELETE FROM task WHERE id=$1")
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(queryresult) => {
            return match queryresult.rows_affected() {
                1 => (StatusCode::OK, Json(json!({"msg": "Task Deleted"}))),
                _ => {
                    (
                        StatusCode::NOT_FOUND,
                        Json(json!({"msg": "task not found"})),
                    )
                }
            };
        }
        Err(e) => {
            tracing::error!("could not find task with id: {:?} error: {:?}", id, e);
            (
                StatusCode::NOT_FOUND,
                Json(json!({"msg": "task not found"})),
            )
        }
    }
}
