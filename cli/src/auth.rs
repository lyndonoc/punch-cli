use keyring::{Entry, Result as KeyringResult};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_value};

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

pub fn new_auth_manager() -> AuthManager {
    let password = retrieve_from_keyring()
        .unwrap_or_default();
    let json_value = Value::String(password);
    let parsed = from_value::<Option<AuthSessionInfo>>(json_value)
        .unwrap_or_else(|_| { None::<AuthSessionInfo> });
    AuthManager{
        session_info: parsed,
    }
}

impl AuthManager {
    pub fn is_logged_in(&self) -> bool {
        self.session_info.is_some()
    }

    pub fn login(&self) {

    }
}

fn retrieve_from_keyring() -> KeyringResult<String> {
    let entry = Entry::new("punch-cli", "session_info")?;
    entry.get_password()
}
