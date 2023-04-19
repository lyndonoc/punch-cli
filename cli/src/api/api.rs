use reqwest;
use reqwest::{Result, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchAccessTokenPayload {
    access_token: String,
}

pub fn fetch_access_token(api_endpoint: String, access_token: &String) -> String {
    let res = reqwest::blocking::Client::new()
        .post(api_endpoint)
        .json(&FetchAccessTokenPayload {
            access_token: access_token.clone(),
        })
        .send()
        .expect("failed to fetch API access token");
    res.text()
        .expect("failed to parse API access token response")
}

pub fn verify_access_token(api_endpoint: String, access_token: &str) -> Result<bool> {
    let res = reqwest::blocking::Client::new()
        .post(api_endpoint)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .expect("request to verify the token failed");
    Ok(res.status() == StatusCode::NO_CONTENT)
}
