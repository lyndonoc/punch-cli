use std::time::SystemTime;
use actix_web::{HttpResponse, Responder, web};

use::serde::Deserialize;

use crate::api::gh::TokenPayload;
use crate::state::AppDeps;
use crate::models::tasks::TaskModel;
use crate::utils::errors::PunchTaskError;

#[derive(Deserialize)]
pub struct BaseTaskInfo {
    name: String,
}

pub async fn start_new_task(
    app_deps: web::Data<AppDeps>,
    token: web::ReqData<TokenPayload>,
    task_info: web::Json<BaseTaskInfo>,
) -> impl Responder {
    let task_name = task_info.name.to_lowercase();
    let dupe_count = match sqlx::query!(
            "SELECT COUNT(*) FROM tasks WHERE name = $1 AND user_github_id = $2 AND finished_at IS NULL;", 
            task_name,
            token.user.id.to_string(),
        )
        .fetch_one(&app_deps.db_pool)
        .await {
            Ok(count) => count.count.unwrap(),
            Err(_) => return Err(PunchTaskError::InternalError),
        };
    if dupe_count > 0 {
        return Err(PunchTaskError::TaskAlreadyInProgress);
    }
    let new_task_op = sqlx::query_as!(
            TaskModel,
            r#"
            INSERT INTO tasks (name, user_github_id, started_at)
            VALUES ($1, $2, $3)
            RETURNING *;
            "#,
            task_name,
            token.user.id.to_string(),
            match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(n) => n.as_secs() as i64,
                Err(_) => return Err(PunchTaskError::InternalError),
            },
        )
        .fetch_one(&app_deps.db_pool)
        .await;
    match new_task_op {
        Ok(new_task) => {
            return Ok(HttpResponse::Ok().json(new_task.to_json()));
        }
        Err(_) => {
            return Err(PunchTaskError::InternalError);
        }
    }
}

pub async fn finish_task(
    app_deps: web::Data<AppDeps>,
    token: web::ReqData<TokenPayload>,
    task_info: web::Json<BaseTaskInfo>,
) -> impl Responder {
    let task_name = task_info.name.to_lowercase();
    let finished_at = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() as i64,
        Err(_) => return Err(PunchTaskError::InternalError),
    };
    let update_op = sqlx::query_as!(
            TaskModel,
            r#"
            UPDATE tasks
            SET finished_at = $1
            WHERE name = $2 AND user_github_id = $3 AND finished_at IS NULL
            RETURNING *;
            "#,
            finished_at,
            task_name,
            token.user.id.to_string(),
        )
        .fetch_one(&app_deps.db_pool)
        .await;
    match update_op {
        Ok(updated_task) => {
            return Ok(HttpResponse::Ok().json(updated_task.to_json()));
        }
        Err(err) => {
            let api_err = match err {
                sqlx::Error::RowNotFound => PunchTaskError::InProgressTaskNotFound,
                _ => PunchTaskError::InternalError,
            };
            return Err(api_err);
        }
    }
}
