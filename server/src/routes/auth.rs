use actix_web::{web, HttpResponse, HttpResponseBuilder, Responder};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::api::gh::{fetch_gh_user, TokenPayload, TokenVerificationPayload};
use crate::utils::{errors::PunchTaskError, jwt::sign_user_jwt, state::AppDeps};

#[derive(Deserialize, FromRow, Serialize)]
pub struct TasksCount {
    pub count: i64,
}

pub async fn verify() -> HttpResponseBuilder {
    HttpResponse::NoContent()
}

pub async fn client_id(app_deps: web::Data<AppDeps>) -> String {
    app_deps.configs.github_client_id.clone()
}

pub async fn login(
    app_deps: web::Data<AppDeps>,
    token_payload: web::Json<TokenVerificationPayload>,
) -> impl Responder {
    let token = match fetch_gh_user(
        &app_deps.configs.github_client_id,
        &app_deps.configs.github_client_secret,
        &token_payload.access_token,
    )
    .await
    {
        Ok(gh_user) => sign_user_jwt::<TokenPayload>(gh_user, &app_deps.configs.jwt_secret),
        Err(err) => {
            return Err(PunchTaskError::ReportableInternalError {
                message: err.to_string(),
            });
        }
    };
    token.map_err(|err| PunchTaskError::ReportableInternalError {
        message: err.to_string(),
    })
}

pub async fn status(
    app_deps: web::Data<AppDeps>,
    token: web::ReqData<TokenPayload>,
) -> impl Responder {
    let count_op = sqlx::query_as::<_, TasksCount>(
        r#"
            SELECT
                COUNT(*) as in_progress_count
            FROM
                tasks
            WHERE
                user_github_id = $1 AND finished_at IS NULL
            GROUP BY
                name;
            "#,
    )
    .bind(token.user.id.to_string())
    .fetch_one(&app_deps.db_pool)
    .await;
    match count_op {
        Ok(count_result) => {
            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "tasks_in_progress": count_result.count,
            })));
        }
        Err(_) => {
            return Err(PunchTaskError::InternalError);
        }
    }
}
