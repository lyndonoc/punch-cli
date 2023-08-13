use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use reqwest;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenVerificationPayload {
    pub access_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenPayload {
    pub user: GitHubUser,
    pub token: String,
}

pub async fn fetch_gh_user(
    client_id: String,
    client_secret: String,
    token: String,
) -> TokenPayload {
    let mut base64 = String::new();
    _ = general_purpose::STANDARD
        .encode_string(format!("{}:{}", client_id, client_secret), &mut base64);
    let payload = TokenVerificationPayload {
        access_token: token.clone(),
    };
    reqwest::Client::new()
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
        .json::<TokenPayload>()
        .await
        .expect("failed to parse response from GitHub")
}
