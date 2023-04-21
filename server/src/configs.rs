use std::env;

pub struct AppConfigs {
    pub github_app_name: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub jwt_secret: String,
}

pub fn fetch_configs() -> AppConfigs {
    let mut github_app_name = String::new();
    let mut github_client_id = String::new();
    let mut github_client_secret = String::new();
    let mut jwt_secret = String::new();
    for (key, value) in env::vars() {
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
    if github_app_name.is_empty()
        || github_client_id.is_empty()
        || github_client_secret.is_empty()
        || jwt_secret.is_empty()
    {
        panic!("missing required environment variables")
    }
    AppConfigs {
        github_app_name,
        github_client_id,
        github_client_secret,
        jwt_secret,
    }
}
