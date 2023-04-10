use keyring::Entry;
use serde::de::DeserializeOwned;
use serde_json::{from_value, Value};

pub trait SecretsManager {
    fn retrieve_secrets<T: DeserializeOwned>(&self) -> Option<T>;
    fn save_secrets();
}

pub struct KeyRingManager {
    storage: Entry,
}

impl SecretsManager for KeyRingManager {
    fn retrieve_secrets<T: DeserializeOwned>(&self) -> Option<T> {
        let pwd = self.storage.get_password().expect("");
        let json_value = Value::String(pwd);
        from_value::<Option<T>>(json_value).unwrap_or_else(|_| None::<T>)
    }

    fn save_secrets() {
        todo!()
    }
}

pub fn new_keyring_manager() -> KeyRingManager {
    let storage = Entry::new("punch-cli", "session_info").expect("failed to read keyring");
    KeyRingManager { storage }
}
