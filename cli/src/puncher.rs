use diesel::prelude::*;
use diesel::SqliteConnection;

use crate::model::get_ts;
use crate::model::Task;
use crate::schema::tasks::finished_at;
use crate::{
    api::api::start_task,
    auth::AuthManager,
    configs::AppConfigs,
    keyring::SecretsManager,
    model::{get_unfinished_task, new_task},
    schema::tasks::table,
};

pub struct Puncher<'a, T: SecretsManager> {
    auth_manager: &'a AuthManager<'a, T>,
    configs: &'a AppConfigs,
    db_conn: &'a SqliteConnection,
}

impl<'a, T> Puncher<'a, T>
where
    T: SecretsManager,
{
    pub fn new(
        auth_manager: &'a AuthManager<T>,
        configs: &'a AppConfigs,
        db_conn: &'a SqliteConnection,
    ) -> Puncher<'a, T> {
        Puncher {
            auth_manager,
            configs,
            db_conn,
        }
    }
    pub fn punch_in(&self, task_name: String) -> Result<i64, String> {
        match self.auth_manager.get_access_token() {
            Some(token) => {
                let api_resp = start_task(
                    format!("{}/punch/in", self.configs.api_endpoint),
                    token,
                    task_name,
                );
                return match api_resp {
                    Ok(task) => Ok(task.started_at),
                    Err(err) => Err(err.into()),
                };
            }
            None => {
                let unfinished = get_unfinished_task(task_name.as_str(), self.db_conn);
                if unfinished.len() > 0 {
                    return Err(String::from("the task is already in progress"));
                }
                let new_task = new_task(task_name.as_str());
                return match diesel::insert_into(table)
                    .values(&new_task)
                    .execute(self.db_conn)
                {
                    Ok(_) => Ok(new_task.started_at),
                    Err(err) => Err(format!("{}", err)),
                };
            }
        }
    }

    pub fn punch_out(&self, task_name: String) -> Result<Task, String> {
        match self.auth_manager.get_access_token() {
            Some(token) => {
                let api_resp = start_task(
                    format!("{}/punch/out", self.configs.api_endpoint),
                    token,
                    task_name,
                );
                return match api_resp {
                    Ok(task) => Ok(Task {
                        id: task.id as i32,
                        name: task.name,
                        started_at: task.started_at,
                        finished_at: task.finished_at,
                    }),
                    Err(err) => Err(err.into()),
                };
            }
            None => {
                let mut existing = get_unfinished_task(task_name.as_str(), self.db_conn);
                if existing.len() == 0 {
                    return Err(String::from("{} no task in progress for {}"));
                }
                let finished_ts = match get_ts() {
                    Ok(ts) => ts.as_secs() as i64,
                    Err(err) => return Err(err.to_string()),
                };
                return match diesel::update(table.find(existing[0].id))
                    .set(finished_at.eq(finished_ts))
                    .execute(self.db_conn)
                {
                    Ok(_) => {
                        let mut old_task = existing.remove(0);
                        old_task.finished_at = Some(finished_ts);
                        Ok(old_task)
                    }
                    Err(err) => Err(format!("{}", err)),
                };
            }
        }
    }
}
