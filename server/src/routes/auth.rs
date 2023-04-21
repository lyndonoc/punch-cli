use crate::api::gh::{fetch_gh_user, TokenPayload, TokenVerificationPayload};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;

use crate::state::AppDeps;
use crate::utils::jwt::sign_user_jwt;

#[get("/client_id")]
pub fn client_id(app_deps: &State<AppDeps>) -> String {
    app_deps.configs.github_client_id.clone()
}

#[post("/login", data = "<token_payload>")]
pub async fn login(
    app_deps: &State<AppDeps>,
    token_payload: Json<TokenVerificationPayload>,
) -> String {
    let gh_user = fetch_gh_user(
        app_deps.configs.github_client_id.clone(),
        app_deps.configs.github_client_secret.clone(),
        token_payload.access_token.clone(),
    )
    .await;
    sign_user_jwt::<TokenPayload>(gh_user, &app_deps.configs.jwt_secret)
}

#[post("/verify")]
pub fn verify(_user: TokenPayload) -> Status {
    Status::NoContent
}
