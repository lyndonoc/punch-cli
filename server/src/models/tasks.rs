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

impl TaskModel {
    pub fn to_json(&self) -> impl Serialize {
        return serde_json::json!({
            "id": self.id.to_owned(),
            "name": self.name.to_owned(),
            "started_at": self.started_at.to_owned(),
            "finished_at": self.finished_at.to_owned(),
        })
    }
}
