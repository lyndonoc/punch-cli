use reqwest;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchAccessTokenPayload {
    access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchAccessTokenResponse {
    access_token: String,
}

pub fn fetch_access_token(api_endpoint: String, access_token: String) -> String {
    let res = reqwest::blocking::Client::new()
        .post(api_endpoint.clone())
        .json(&FetchAccessTokenPayload { access_token })
        .send()
        .expect("failed to fetch API access token");
    let token_res = res
        .text()
        .expect("failed to parse API access token response");
    token_res
}

pub fn verify_access_token(api_endpoint: String, access_token: String) -> bool {
    let res = reqwest::blocking::Client::new()
        .post(api_endpoint.clone())
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .expect("failed to fetch API access token");
    res.status() == StatusCode::NO_CONTENT
}
