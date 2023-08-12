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
use sqlx::postgres::PgPoolOptions;

pub mod api;
pub mod models;
pub mod configs;
pub mod routes;
pub mod state;
pub mod utils;

use crate::configs::fetch_configs;
use crate::routes::auth::{client_id, login, verify};
use crate::routes::punch::start_new_task;
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
    let configs = fetch_configs();
    let db_pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&configs.database_url)
        .await {
            Ok(pool) => { pool }
            Err(_) => {
                std::process::exit(1);
            }
        };
    HttpServer::new(move || {
        let bearer_middleware = HttpAuthentication::bearer(auth_validator);

        App::new()
            .app_data(web::Data::new(AppDeps {
                configs: configs.clone(),
                db_pool: db_pool.clone(),
            }))
            .route("/ping", web::get().to(ping))
            .service(
                web::scope("/auth")
                    .route("/client_id", web::get().to(client_id))
                    .route("/login", web::post().to(login))
                    .route("/verify", web::post().to(verify).wrap(bearer_middleware.clone())),
            )
            .service(
                web::scope("/punch")
                    .route("/in", web::post().to(start_new_task).wrap(bearer_middleware.clone())),
            )
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}

async fn ping() -> impl Responder {
    "pong"
}
