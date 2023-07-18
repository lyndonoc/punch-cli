use actix_web::{web, HttpResponse, HttpResponseBuilder};

use crate::api::gh::{fetch_gh_user, TokenPayload, TokenVerificationPayload};
use crate::state::AppDeps;
use crate::utils::jwt::sign_user_jwt;

pub async fn client_id(app_deps: web::Data<AppDeps>) -> String {
    app_deps.configs.github_client_id.clone()
}

pub async fn login(
    app_deps: web::Data<AppDeps>,
    token_payload: web::Json<TokenVerificationPayload>,
) -> String {
    let gh_user = fetch_gh_user(
        app_deps.configs.github_client_id.clone(),
        app_deps.configs.github_client_secret.clone(),
        token_payload.access_token.clone(),
    )
    .await;
    sign_user_jwt::<TokenPayload>(gh_user, &app_deps.configs.jwt_secret)
}

pub async fn verify() -> HttpResponseBuilder {
    HttpResponse::NoContent()
}
