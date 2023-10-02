use std::env;

#[derive(Clone)]
pub struct AppConfigs {
    pub database_url: String,
    pub github_app_name: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub jwt_secret: String,
}

pub fn fetch_configs() -> AppConfigs {
    let mut database_url = String::new();
    let mut github_app_name = String::new();
    let mut github_client_id = String::new();
    let mut github_client_secret = String::new();
    let mut jwt_secret = String::new();
    for (key, value) in env::vars() {
        if key == "DATABASE_URL" {
            database_url = value;
            continue;
        }
        if key == "GITHUB_APP_NAME" {
            github_app_name = value;
            continue;
        }
        if key == "GITHUB_CLIENT_ID" {
            github_client_id = value;
            continue;
        }
        if key == "GITHUB_CLIENT_SECRET" {
            github_client_secret = value;
            continue;
        }
        if key == "JWT_SECRET" {
            jwt_secret = value;
            continue;
        }
    }
    if database_url.is_empty()
        || github_app_name.is_empty()
        || github_client_id.is_empty()
        || github_client_secret.is_empty()
        || jwt_secret.is_empty()
    {
        panic!("missing required environment variables")
    }
    AppConfigs {
        database_url,
        github_app_name,
        github_client_id,
        github_client_secret,
        jwt_secret,
    }
}
