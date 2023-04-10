use crate::keyring::SecretsManager;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct AuthSessionInfo {
    access_token: String,
    expires_in: u64,
    refresh_token: String,
    refresh_token_expires_in: u64,
    scope: String,
    token_type: String,
}

pub struct AuthManager {
    session_info: Option<AuthSessionInfo>,
}

pub fn new_auth_manager(secrets_manager: impl SecretsManager) -> AuthManager {
    let session_info = secrets_manager.retrieve_secrets::<AuthSessionInfo>();
    AuthManager { session_info }
}

impl AuthManager {
    pub fn is_logged_in(&self) -> bool {
        self.session_info.is_some()
    }
}
