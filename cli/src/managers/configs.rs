extern crate dotenv;

use dotenv::dotenv;
use std::env;

pub struct AppConfigs {
    pub api_endpoint: String,
    pub gh_auth_scope: String,
}

pub fn fetch_configs() -> AppConfigs {
    dotenv().ok();
    let mut api_endpoint = String::new();
    let mut gh_auth_scope = String::new();
    for (key, value) in env::vars() {
        if key == "API_ENDPOINT" {
            api_endpoint = value;
            continue;
        }
        if key == "GITHUB_AUTH_SCOPE" {
            gh_auth_scope = value;
            continue;
        }
    }
    if api_endpoint.is_empty() || gh_auth_scope.is_empty() {
        panic!("missing required environment variables")
    }
    AppConfigs {
        api_endpoint,
        gh_auth_scope,
    }
}
