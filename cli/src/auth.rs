use crate::keyring::{KeyRingManager, SecretsManager};

use crate::api::api::{fetch_access_token, verify_access_token};
use crate::api::github::{fetch_gh_client_id, fetch_gh_login_info, prompt_and_fetch_gh_tokens};
use crate::configs::AppConfigs;
use crate::utils::SimpleError;

pub struct AuthManager<T: SecretsManager> {
    configs: AppConfigs,
    pub is_logged_in: bool,
    keyring_manager: T,
    token: Option<String>,
}

impl<T> AuthManager<T>
where
    T: SecretsManager,
{
    pub fn new(configs: AppConfigs, keyring_manager: T) -> AuthManager<T> {
        AuthManager {
            configs,
            is_logged_in: false,
            keyring_manager,
            token: None,
        }
    }

    pub fn initialize(&mut self) {
        let mut token = self
            .keyring_manager
            .retrieve_secrets()
            .unwrap_or_else(|_| String::from(""));
        if token == "" {
            token = self.login()
        }
        match self.verify_login(&token) {
            Ok(_) => {
                self.token = Some(token);
                self.is_logged_in = true;
            }
            Err(err) => {
                panic!("{}", err);
            }
        };
    }

    pub fn login(&self) -> String {
        let client_id_info =
            fetch_gh_client_id(&self.configs.api_endpoint, &self.configs.gh_auth_scope);
        let login_info = fetch_gh_login_info(&client_id_info);
        let user = prompt_and_fetch_gh_tokens(&client_id_info, &login_info);
        fetch_access_token(
            String::from(format!("{}/auth/login", &self.configs.api_endpoint)),
            &user.access_token,
        )
    }

    fn verify_login(&self, access_token: &str) -> Result<(), SimpleError> {
        return match verify_access_token(
            format!("{}/auth/verify", &self.configs.api_endpoint),
            access_token,
        ) {
            Ok(is_verified) => {
                if !is_verified {
                    self.keyring_manager.remove_secret();
                    return Err(SimpleError {
                        message: String::from("the token is not valid"),
                    });
                }
                Ok(())
            }
            Err(err) => {
                self.keyring_manager.remove_secret();
                Err(SimpleError {
                    message: String::from(format!("failed to verify the token {}", err)),
                })
            }
        };
    }
}
