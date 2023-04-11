use keyring::Entry;
use serde::de::DeserializeOwned;
use serde_json::{from_value, Value};

pub trait SecretsManager {
    fn new() -> Self;
    fn retrieve_secrets<T: DeserializeOwned>(&self) -> Option<T>;
    fn save_secrets(&self, pwd: &str);
}

pub struct KeyRingManager {
    storage: Entry,
}

impl SecretsManager for KeyRingManager {
    fn new() -> Self {
        let storage = Entry::new("punch-cli", "session_info").expect("failed to read keyring");
        KeyRingManager { storage }
    }
    fn retrieve_secrets<T: DeserializeOwned>(&self) -> Option<T> {
        let pwd = self.storage.get_password().expect("");
        let json_value = Value::String(pwd);
        from_value::<Option<T>>(json_value).unwrap_or_else(|_| None::<T>)
    }

    fn save_secrets(&self, pwd: &str) {
        self.storage
            .set_password(&pwd)
            .expect("failed to save token to keyring");
    }
}
