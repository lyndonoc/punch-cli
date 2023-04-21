#[macro_use]
extern crate rocket;

pub mod api;
pub mod configs;
pub mod routes;
pub mod state;
pub mod utils;

use crate::configs::fetch_configs;
use crate::routes::auth::{client_id, login, verify};
use crate::state::AppDeps;

#[get("/")]
fn ping() -> &'static str {
    "pong"
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    rocket::build()
        .manage(AppDeps {
            configs: fetch_configs(),
        })
        .mount("/ping", routes![ping])
        .mount("/auth", routes![client_id, login, verify])
        .ignite()
        .await?;
    Ok(())
}
