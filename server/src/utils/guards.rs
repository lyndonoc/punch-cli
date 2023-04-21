use std::env;

use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use serde::Deserialize;

use crate::api::gh::TokenPayload;
use crate::utils::jwt::verify_user_jwt;

#[derive(Debug, Deserialize)]
pub enum AuthError {
    BadRequest,
    InvalidToken,
    InternalServerError,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TokenPayload {
    type Error = AuthError;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<TokenPayload, AuthError> {
        if let Some(auth_header) = req.headers().get_one("Authorization") {
            let tk_header = auth_header.to_string();
            if !tk_header.starts_with("Bearer ") {
                return Outcome::Failure((Status::BadRequest, AuthError::BadRequest));
            }
            let token = tk_header.trim_start_matches("Bearer ");
            let mut jwt_secret = String::new();
            match env::var("JWT_SECRET") {
                Ok(secret) => {
                    jwt_secret = secret;
                }
                Err(_) => {}
            };
            if jwt_secret.is_empty() {
                return Outcome::Failure((Status::Unauthorized, AuthError::InternalServerError));
            }
            return match verify_user_jwt::<TokenPayload>(token, jwt_secret) {
                Ok(decoded) => Outcome::Success(decoded.claims.claim),
                Err(_) => Outcome::Failure((Status::Unauthorized, AuthError::InvalidToken)),
            };
        }
        return Outcome::Failure((Status::BadRequest, AuthError::BadRequest));
    }
}
