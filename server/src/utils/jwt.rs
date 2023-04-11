use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{
    decode, encode, errors::Result, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseJWTClaims<T> {
    pub claim: T,
    exp: usize,
}

pub fn sign_user_jwt<T: Serialize>(claim: T, secret: String) -> String {
    let claims = BaseJWTClaims {
        claim,
        exp: 10000000000,
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
