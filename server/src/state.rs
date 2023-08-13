use sqlx::postgres::PgPool;

use crate::configs::AppConfigs;

pub struct AppDeps {
    pub configs: AppConfigs,
    pub db_pool: PgPool,
}
