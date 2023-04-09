use reqwest;
use rocket::serde::{json::Json, Deserialize as RocketDeserialize, Serialize as RocketSerialize};
use rocket::State;
use serde::{Deserialize, Serialize};

use crate::state::AppDeps;

#[derive(Debug, Serialize, Deserialize)]
struct GHLoginPayload {
    device_code: String,
    expires_in: u64,
    interval: u32,
    user_code: String,
    verification_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginPayload {
    client_id: String,
    scope: String,
}

#[derive(RocketDeserialize, RocketSerialize)]
#[serde(crate = "rocket::serde")]
pub struct LoginResponse {
    device_code: String,
    expires_in: u64,
    interval: u32,
    user_code: String,
    verification_uri: String,
}

#[post("/login")]
pub async fn login(app_deps: &State<AppDeps>) -> Json<LoginResponse> {
    let payload = LoginPayload {
        client_id: app_deps.configs.github_client_id.clone(),
        scope: String::from("read:user"),
    };
    let res = reqwest::Client::new()
        .post("https://github.com/login/device/code")
        .json(&payload)
        .send()
        .await
        .expect("internal server error");
    if res.status().as_u16() != 200 {
        panic!("internal server error");
    }
    let body = res.text().await.expect("internal server error");
    let decoded = serde_urlencoded::from_bytes::<GHLoginPayload>(body.as_bytes())
        .expect("internal server error");
    Json(LoginResponse {
        device_code: decoded.device_code,
        expires_in: decoded.expires_in,
        interval: decoded.interval,
        user_code: decoded.user_code,
        verification_uri: decoded.verification_uri,
    })
}
