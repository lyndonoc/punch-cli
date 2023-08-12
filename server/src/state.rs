use crate::configs::AppConfigs;
use sqlx::postgres::PgPool;

pub struct AppDeps {
    pub configs: AppConfigs,
    pub db_pool: PgPool,
}
