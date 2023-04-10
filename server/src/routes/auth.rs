use rocket::State;

use crate::state::AppDeps;

#[get("/client_id")]
pub fn client_id(app_deps: &State<AppDeps>) -> String {
    app_deps.configs.github_client_id.clone()
}
