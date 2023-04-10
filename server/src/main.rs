#[macro_use]
extern crate rocket;

pub mod api;
pub mod configs;
pub mod routes;
pub mod state;

use crate::configs::fetch_configs;
use crate::routes::auth::client_id;
use crate::state::AppDeps;

#[get("/")]
fn ping() -> &'static str {
    "pong"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(AppDeps {
            configs: fetch_configs(),
        })
        .mount("/ping", routes![ping])
        .mount("/auth", routes![client_id])
}
