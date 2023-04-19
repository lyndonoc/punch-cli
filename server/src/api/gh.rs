use base64::{engine::general_purpose, Engine};
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenVerificationPayload {
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenVerificationResponse {
    user: GitHubUser,
}

pub async fn fetch_gh_user(client_id: String, client_secret: String, token: String) -> GitHubUser {
    let mut base64 = String::new();
    _ = general_purpose::STANDARD
        .encode_string(format!("{}:{}", client_id, client_secret), &mut base64);
    let payload = TokenVerificationPayload {
        access_token: token.clone(),
    };
    let res = reqwest::Client::new()
        .post(format!(
            "https://api.github.com/applications/{}/token",
            client_id
        ))
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Basic {}", base64.clone()))
        .header("User-Agent", "punch-cli-dev")
        .json(&payload)
        .send()
        .await
        .expect("failed to fetch user")
        .json::<TokenVerificationResponse>()
        .await
        .expect("failed to parse response from GitHub");
    res.user
}

pub async fn verify_gh_token() {}
