use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::time::Duration;
use time::OffsetDateTime;

use jsonwebtoken::{
    decode, encode, errors::Result, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};
use rocket::time;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BaseJWTClaims<T> {
    pub claim: T,
    iat: i64,
    exp: i64,
}

pub fn sign_user_jwt<T: Serialize>(claim: T, secret: &str) -> String {
    let iat = OffsetDateTime::now_utc();
    let exp = OffsetDateTime::now_utc().add(Duration::from_secs(31556926));

    let claims = BaseJWTClaims {
        claim,
        iat: iat.unix_timestamp(),
        exp: exp.unix_timestamp(),
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
