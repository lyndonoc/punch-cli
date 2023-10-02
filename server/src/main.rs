pub mod api;
pub mod models;
pub mod routes;
pub mod utils;

use std::env;

use actix_web::{dev::ServiceRequest, error::Error, web, App, HttpMessage, HttpServer, Responder};
use actix_web_httpauth::{
    extractors::{
        bearer::{self, BearerAuth},
        AuthenticationError,
    },
    middleware::HttpAuthentication,
};
use sqlx::postgres::PgPoolOptions;

use crate::api::gh::TokenPayload;
use crate::routes::{
    auth::{client_id, login, status, verify},
    punch::{cancel_task, finish_task, get_task, list_tasks, start_new_task},
};
use crate::utils::{configs::fetch_configs, jwt::verify_user_jwt, state::AppDeps};

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
        .await
    {
        Ok(pool) => pool,
        Err(err) => {
            panic!("{}", format!("failed to connect to the database: {}", err));
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
                    .route(
                        "/verify",
                        web::post().to(verify).wrap(bearer_middleware.clone()),
                    ),
            )
            .service(
                web::scope("/punch")
                    .route("/in", web::post().to(start_new_task))
                    .route("/out", web::post().to(finish_task))
                    .route("/cancel", web::post().to(cancel_task))
                    .route("/get/{task_name}", web::get().to(get_task))
                    .route("/list", web::get().to(list_tasks))
                    .route("/status", web::get().to(status))
                    .wrap(bearer_middleware.clone()),
            )
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}

async fn ping() -> impl Responder {
    "pong"
}
