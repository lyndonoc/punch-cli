use crate::api::gh::GitHubUser;
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use serde::Deserialize;

use crate::utils::jwt::verify_user_jwt;

#[derive(Debug, Deserialize)]
pub enum AuthError {
    BadRequest,
    InvalidToken,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GitHubUser {
    type Error = AuthError;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<GitHubUser, AuthError> {
        if let Some(auth_header) = req.headers().get_one("Authorization") {
            let tk_header = auth_header.to_string();
            if !tk_header.starts_with("Bearer ") {
                return Outcome::Failure((Status::BadRequest, AuthError::BadRequest));
            }
            let token = tk_header.trim_start_matches("Bearer ");
            return match verify_user_jwt::<GitHubUser>(token, String::from("power")) {
                Ok(decoded) => Outcome::Success(decoded.claims.claim),
                Err(_) => Outcome::Failure((Status::Unauthorized, AuthError::InvalidToken)),
            };
        }
        return Outcome::Failure((Status::BadRequest, AuthError::BadRequest));
    }
}
