use keyring::{Entry, Error, Result};

pub trait SecretsManager {
    fn remove_secret(&self);
    fn retrieve_secrets(&self) -> Result<String>;
    fn save_secrets(&self, pwd: &str);
}

pub struct KeyRingManager {
    storage: Entry,
}

impl SecretsManager for KeyRingManager {
    fn remove_secret(&self) {
        match self.storage.delete_password() {
            Err(err) => match err {
                Error::NoEntry => (),
                _ => {
                    panic!("{:?}", err);
                }
            },
            _ => (),
        };
    }

    fn retrieve_secrets(&self) -> Result<String> {
        self.storage.get_password()
    }

    fn save_secrets(&self, pwd: &str) {
        self.storage
            .set_password(&pwd)
            .expect("failed to save token to keyring");
    }
}

pub fn new_key_ring_manager() -> impl SecretsManager {
    let storage = Entry::new("punch-cli", "session_info").expect("failed to read keyring");
    KeyRingManager { storage }
}
