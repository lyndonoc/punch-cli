use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

use diesel::prelude::*;
use diesel::sql_types::{BigInt, Nullable};
use diesel::sqlite::SqliteConnection;

use super::schema::tasks::{self, finished_at, name};

#[derive(Insertable, Queryable)]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

#[derive(Insertable)]
#[table_name = "tasks"]
pub struct NewTask {
    pub name: String,
    pub started_at: i64,
}

#[derive(QueryableByName)]
#[table_name = "tasks"]
pub struct AggregatedTask {
    pub name: String,
    pub started_at: i64,
    #[sql_type = "Nullable<BigInt>"]
    pub finished_at: Option<i64>,
    #[sql_type = "BigInt"]
    pub duration: i64,
}

pub fn get_unfinished_task(task_name: &str, conn: &SqliteConnection) -> Vec<Task> {
    tasks::table
        .filter(name.eq(task_name))
        .filter(finished_at.is_null())
        .limit(1)
        .load::<Task>(conn)
        .unwrap()
}

pub fn get_ts() -> Result<Duration, SystemTimeError> {
    SystemTime::now().duration_since(UNIX_EPOCH)
}
