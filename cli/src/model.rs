use crate::schema::tasks;
use crate::schema::tasks::{finished_at, name};

use diesel::prelude::*;
use diesel::sql_types::{BigInt, Nullable};
use diesel::sqlite::SqliteConnection;

use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

#[derive(Insertable, Queryable, Debug)]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

#[derive(Insertable, Debug)]
#[table_name = "tasks"]
pub struct NewTask<'a> {
    pub name: &'a str,
    pub started_at: i64,
}

#[derive(QueryableByName, Debug)]
#[table_name = "tasks"]
pub struct AggregatedTask {
    pub name: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    #[sql_type = "Nullable<BigInt>"]
    pub duration: Option<i64>,
}

pub fn new_task(task_name: &str) -> NewTask {
    NewTask {
        name: task_name,
        started_at: get_ts().unwrap().as_secs() as i64,
    }
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
