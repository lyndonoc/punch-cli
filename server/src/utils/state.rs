use sqlx::postgres::PgPool;

use crate::utils::configs::AppConfigs;

pub struct AppDeps {
    pub configs: AppConfigs,
    pub db_pool: PgPool,
}
