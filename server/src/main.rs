use crate::api::gh::TokenPayload;
use actix_web::{dev::ServiceRequest, error::Error, web, App, HttpMessage, HttpServer, Responder};
use actix_web_httpauth::{
    extractors::{
        bearer::{self, BearerAuth},
        AuthenticationError,
    },
    middleware::HttpAuthentication,
};
use std::env;

pub mod api;
pub mod configs;
pub mod routes;
pub mod state;
pub mod utils;

use crate::configs::fetch_configs;
use crate::routes::auth::{client_id, login, verify};
use crate::state::AppDeps;
use crate::utils::jwt::verify_user_jwt;

async fn auth_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();
    let secret = env::var("JWT_SECRET").expect("internal server error");
    match verify_user_jwt::<TokenPayload>(token, secret) {
        Ok(decoded) => {
            req.extensions_mut().insert(decoded.claims.claim);
            Ok(req)
        }
        Err(_) => {
            let config = req
                .app_data::<bearer::Config>()
                .cloned()
                .unwrap_or_default()
                .scope("");
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let bearer_middleware = HttpAuthentication::bearer(auth_validator);

        App::new()
            .app_data(web::Data::new(AppDeps {
                configs: fetch_configs(),
            }))
            .route("/ping", web::get().to(ping))
            .service(
                web::scope("/auth")
                    .route("/client_id", web::get().to(client_id))
                    .route("/login", web::post().to(login))
                    .route("/verify", web::post().to(verify).wrap(bearer_middleware)),
            )
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}

async fn ping() -> impl Responder {
    "pong"
}
