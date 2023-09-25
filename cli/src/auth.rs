use crate::keyring::SecretsManager;

use crate::api::api::{fetch_access_token, verify_access_token};
use crate::api::github::{fetch_gh_client_id, fetch_gh_login_info, prompt_and_fetch_gh_tokens};
use crate::configs::AppConfigs;
use crate::utils::SimpleError;

pub struct AuthManager<'a, T: SecretsManager> {
    configs: &'a AppConfigs,
    keyring_manager: &'a T,
    token: Option<String>,
}

impl<'a, T> AuthManager<'a, T>
where
    T: SecretsManager,
{
    pub fn new(configs: &'a AppConfigs, keyring_manager: &'a T) -> AuthManager<'a, T> {
        match keyring_manager.retrieve_secrets() {
            Ok(secret) => {
                let endpoint = format!("{}/auth/verify", &configs.api_endpoint);
                match verify_access_token(&endpoint, &secret) {
                    Ok(is_verified) => AuthManager {
                        configs,
                        keyring_manager,
                        token: if is_verified { Some(secret) } else { None },
                    },
                    Err(_) => AuthManager {
                        configs,
                        keyring_manager,
                        token: None,
                    },
                }
            }
            Err(_) => AuthManager {
                configs,
                keyring_manager,
                token: None,
            },
        }
    }

    // TODO: change the return type to &str
    pub fn get_access_token(&self) -> Option<String> {
        self.token.clone()
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

    pub fn verify_login(&self, access_token: &str) -> Result<(), SimpleError> {
        let endpoint = format!("{}/auth/verify", &self.configs.api_endpoint);
        return match verify_access_token(&endpoint, access_token) {
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
