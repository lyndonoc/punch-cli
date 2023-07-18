use jsonwebtoken::{
    decode, encode, errors::Result, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::time::{Duration, SystemTime};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BaseJWTClaims<T> {
    pub claim: T,
    iat: u64,
    exp: u64,
}

pub fn sign_user_jwt<T: Serialize>(claim: T, secret: &str) -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let claims = BaseJWTClaims {
        claim,
        iat: now.as_secs(),
        exp: now.add(Duration::from_secs(31556926)).as_secs(),
    };
    let header = Header {
        kid: Some("signing_key".to_owned()),
        alg: Algorithm::HS512,
        ..Default::default()
    };

    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("failed to sign JWT")
}

pub fn verify_user_jwt<T>(token: &str, secret: String) -> Result<TokenData<BaseJWTClaims<T>>>
where
    T: for<'a> Deserialize<'a>,
{
    decode::<BaseJWTClaims<T>>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS512),
    )
}
