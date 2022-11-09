use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct Task {
    pub id: i64,
    pub task: String,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct NewTask {
    pub task: String,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct UpdateTask {
    pub task: String,
}
