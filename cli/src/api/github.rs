use crate::configs;

use chrono::{Duration, Utc};
use configs::AppConfigs;
use serde::{Deserialize, Serialize};
use std::{thread, time};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginPayload {
    pub client_id: String,
    scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub device_code: String,
    pub expires_in: u64,
    pub interval: u32,
    pub user_code: String,
    pub verification_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GHLoginPayload {
    pub device_code: String,
    pub expires_in: u64,
    pub interval: u32,
    pub user_code: String,
    pub verification_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenPayload {
    client_id: String,
    device_code: String,
    grant_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    token_type: String,
    scope: String,
}

pub fn fetch_gh_client_id(app_configs: &AppConfigs) -> LoginPayload {
    let res = reqwest::blocking::get(format!("{}/auth/client_id", app_configs.api_endpoint))
        .expect("failed to fetch client id from the server");
    let client_id = res
        .text()
        .expect("failed to parse client id from the response");
    LoginPayload {
        client_id,
        scope: app_configs.gh_auth_scope.clone(),
    }
}

pub fn fetch_gh_login_info(payload: &LoginPayload) -> LoginResponse {
    let res = reqwest::blocking::Client::new()
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .json(&payload)
        .send()
        .expect("failed to attempt to login to GitHub");
    match res.json::<GHLoginPayload>() {
        Ok(parsed) => LoginResponse {
            device_code: parsed.device_code,
            expires_in: parsed.expires_in,
            interval: parsed.interval,
            user_code: parsed.user_code,
            verification_uri: parsed.verification_uri,
        },
        Err(_) => {
            panic!("failed to attempt to login to GitHub");
        }
    }
}

pub fn prompt_and_fetch_gh_tokens(
    client_id_info: &LoginPayload,
    login_info: &LoginResponse,
) -> TokenResponse {
    println!(
        "please enter your one-time code: {}",
        login_info.user_code.clone()
    );
    _ = open::that(login_info.verification_uri.clone());
    let auth_payload = AccessTokenPayload {
        client_id: client_id_info.client_id.clone(),
        device_code: login_info.device_code.clone(),
        grant_type: String::from("urn:ietf:params:oauth:grant-type:device_code"),
    };
    let interval = time::Duration::from_secs_f32(login_info.interval as f32);
    let deadline = Utc::now() + Duration::seconds(login_info.expires_in as i64);
    loop {
        if Utc::now() > deadline {
            panic!("failed to fetch tokens from github API");
        }
        let res = reqwest::blocking::Client::new()
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .json(&auth_payload)
            .send()
            .expect("failed to fetch tokens from github API");
        match res.json::<TokenResponse>() {
            Ok(body) => {
                return body;
            }
            Err(_) => {
                thread::sleep(interval);
                continue;
            }
        };
    }
}
