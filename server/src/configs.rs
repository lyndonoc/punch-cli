extern crate dotenv;

use base64::{engine, Engine};
use dotenv::dotenv;
use std::env;

pub struct AppConfigs {
    pub github_app_id: u64,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_private_key: Vec<u8>,
}

pub fn fetch_configs() -> AppConfigs {
    dotenv().ok();
    let mut github_app_id = 0 as u64;
    let mut github_client_id = String::new();
    let mut github_client_secret = String::new();
    let mut github_private_key = Vec::<u8>::new();
    for (key, value) in env::vars() {
        if key == "GITHUB_APP_ID" {
            match value.parse::<u64>() {
                Ok(app_id) => {
                    github_app_id = app_id;
                    continue;
                }
                Err(_) => {
                    panic!("invalid value provided for GITHUB_APP_ID");
                }
            }
        }
        if key == "GITHUB_CLIENT_ID" {
            github_client_id = value;
            continue;
        }
        if key == "GITHUB_CLIENT_SECRET" {
            github_client_secret = value;
            continue;
        }
        if key == "GITHUB_PRIVATE_KEY" {
            match engine::general_purpose::STANDARD.decode_vec(value, &mut github_private_key) {
                Ok(_) => {
                    continue;
                }
                Err(_) => {
                    panic!("invalid value provided for GITHUB_PRIVATE_KEY")
                }
            };
        }
    }
    if github_client_id.is_empty() || github_client_secret.is_empty() {
        panic!("missing required environment variables")
    }
    AppConfigs {
        github_app_id,
        github_client_id,
        github_client_secret,
        github_private_key,
    }
}
