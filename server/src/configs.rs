extern crate dotenv;

use dotenv::dotenv;
use std::env;

pub struct AppConfigs {
    pub github_client_id: String,
    pub github_client_secret: String,
}

pub fn fetch_configs() -> AppConfigs {
    dotenv().ok();
    let mut github_client_id = String::new();
    let mut github_client_secret = String::new();
    for (key, value) in env::vars() {
        if key == "GITHUB_CLIENT_ID" {
            github_client_id = value;
            continue;
        }
        if key == "GITHUB_CLIENT_SECRET" {
            github_client_secret = value;
            continue;
        }
    }
    if github_client_id.is_empty() || github_client_secret.is_empty() {
        panic!("missing required environment variables")
    }
    AppConfigs {
        github_client_id,
        github_client_secret,
    }
}
