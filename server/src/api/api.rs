use jsonwebtoken::{encode, errors::Error, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iat: usize,
    iss: String,
    exp: usize,
}

pub fn get_gh_app_jwt(app_id: String, private_key: Vec<u8>) -> Result<String, Error> {
    let jwt = EncodingKey::from_rsa_pem(&*private_key).expect("wowza");
    let iat = SystemTime::now().duration_since(UNIX_EPOCH).expect("power");
    let exp = iat.add(Duration::from_secs(600));
    let my_claims = Claims {
        iat: iat.as_secs() as usize,
        iss: app_id.clone(),
        exp: exp.as_secs() as usize,
    };
    let header = Header {
        alg: Algorithm::RS256,
        ..Default::default()
    };
    encode(&header, &my_claims, &jwt)
}
