use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct TaskModel {
    pub id: i64,
    pub name: String,
    pub user_github_id: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

