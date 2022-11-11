use serde::{Deserialize, Serialize};
// swagger openapi
use utoipa::ToSchema;

#[derive(sqlx::FromRow, Deserialize, Serialize, ToSchema)]
pub struct Task {
    pub id: i64,
    #[schema(example = "Buy groceries")]
    pub task: String,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, ToSchema)]
pub struct NewTask {
    #[schema(example = "Buy groceries")]
    pub task: String,
}

#[derive(Deserialize, Serialize, sqlx::FromRow, ToSchema)]
pub struct UpdateTask {
    #[schema(example = "Buy many groceries")]
    pub task: String,
}
