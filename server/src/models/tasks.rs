use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
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
        });
    }
}

pub fn tasks_to_task_report(tasks: Vec<TaskModel>, name: String, right_now: i64) -> impl Serialize {
    let mut is_in_progress = false;
    let mut duration_sum: i64 = 0;
    for task in tasks {
        if task.finished_at.is_none() {
            is_in_progress = true;
            duration_sum = duration_sum + (right_now - task.started_at)
        } else {
            duration_sum = duration_sum + (task.finished_at.unwrap() - task.started_at)
        }
    }
    return serde_json::json!({
        "name": name.to_owned(),
        "status": if is_in_progress { "in progress" } else { "complete "},
        "duration": duration_sum,
    });
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskListModel {
    pub name: String,
    pub duration: Option<BigDecimal>,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}
