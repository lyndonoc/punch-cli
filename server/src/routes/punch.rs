use actix_web::{web, HttpResponse, Responder};
use std::time::SystemTime;

use serde::Deserialize;

use crate::api::gh::TokenPayload;
use crate::models::tasks::{
    tasks_to_task_report, TaskListModel, TaskListModelForResponse, TaskModel,
};
use crate::state::AppDeps;
use crate::utils::errors::PunchTaskError;
use bigdecimal::ToPrimitive;

#[derive(Deserialize)]
pub struct BaseTaskInfo {
    name: String,
}

#[derive(Deserialize)]
pub struct TimeFilterInfo {
    pub since: Option<i64>,
    pub until: Option<i64>,
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

pub async fn cancel_task(
    app_deps: web::Data<AppDeps>,
    token: web::ReqData<TokenPayload>,
    task_info: web::Json<BaseTaskInfo>,
) -> impl Responder {
    let task_name = task_info.name.to_lowercase();
    let delete_op = sqlx::query_as!(
        TaskModel,
        r#"
            DELETE FROM tasks
            WHERE name = $1 AND user_github_id = $2 AND finished_at IS NULL;
            "#,
        task_name,
        token.user.id.to_string(),
    )
    .execute(&app_deps.db_pool)
    .await;
    match delete_op {
        Ok(delete_result) => {
            if delete_result.rows_affected() < 1 {
                return Err(PunchTaskError::InProgressTaskNotFound);
            }
            return Ok(HttpResponse::NoContent());
        }
        Err(_) => {
            return Err(PunchTaskError::InternalError);
        }
    }
}

pub async fn get_task(
    app_deps: web::Data<AppDeps>,
    token: web::ReqData<TokenPayload>,
    name: web::Path<String>,
    ts_filter: web::Query<TimeFilterInfo>,
) -> impl Responder {
    let task_name = name.to_lowercase();
    let right_now = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() as i64,
        Err(_) => return Err(PunchTaskError::InternalError),
    };
    let since = match ts_filter.since {
        Some(since_ts) => since_ts,
        None => 0,
    };
    let until = match ts_filter.until {
        Some(until) => until,
        None => std::i64::MAX,
    };
    let get_task_op = sqlx::query_as!(
        TaskModel,
        r#"
            SELECT *
            FROM tasks
            WHERE 
            name = $1 AND 
            user_github_id = $2 AND
            started_at <= $3 AND
            (finished_at IS NULL OR finished_at >= $4)
            ORDER BY started_at ASC;
            "#,
        task_name,
        token.user.id.to_string(),
        until,
        since,
    )
    .fetch_all(&app_deps.db_pool)
    .await;
    match get_task_op {
        Ok(tasks) => {
            if tasks.len() < 1 {
                return Err(PunchTaskError::TaskNotFound);
            }
            let task_report = tasks_to_task_report(tasks, task_name, right_now);
            return Ok(HttpResponse::Ok().json(task_report));
        }
        Err(_) => {
            return Err(PunchTaskError::InternalError);
        }
    }
}

pub async fn list_tasks(
    app_deps: web::Data<AppDeps>,
    token: web::ReqData<TokenPayload>,
) -> impl Responder {
    let task_rows = sqlx::query_as::<_, TaskListModel>(
        r#"
            SELECT
                name,
                MAX(started_at) as started_at,
                CASE WHEN count(*) - count(finished_at) > 0 THEN NULL ELSE MAX(finished_at) END as finished_at,
                SUM(finished_at - started_at) as duration
            FROM
                tasks
            WHERE
                user_github_id = $1
            GROUP BY
                name;
            "#
    )
    .bind(token.user.id.to_string())
    .fetch_all(&app_deps.db_pool)
    .await;
    match task_rows {
        Ok(rows) => {
            let tasks: Vec<TaskListModelForResponse> = rows
                .iter()
                .map(|task_row| TaskListModelForResponse {
                    name: task_row.name.to_owned(),
                    duration: task_row.duration.to_i64().unwrap_or(0),
                    started_at: task_row.started_at,
                    finished_at: task_row.finished_at,
                })
                .collect();
            return Ok(HttpResponse::Ok().json(serde_json::json!(tasks)));
        }
        Err(_) => {
            return Err(PunchTaskError::InternalError);
        }
    }
}
