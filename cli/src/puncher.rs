use std::cmp;

use diesel::prelude::*;
use diesel::sql_query;
use diesel::SqliteConnection;

use crate::api::api::{cancel_task, get_task, list_task, start_task};
use crate::database::{
    schema::tasks::{self, finished_at, name, started_at, table},
    task::{get_ts, get_unfinished_task, AggregatedTask, NewTask, Task},
};
use crate::managers::{auth::AuthManager, configs::AppConfigs, keyring::SecretsManager};

pub struct TaskListItem {
    pub name: String,
    pub duration: i64,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

pub type TaskList = Vec<TaskListItem>;

pub struct TaskStat {
    pub name: String,
    pub status: String,
    pub duration: i64,
}

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
                let new_task = NewTask {
                    name: task_name,
                    started_at: get_ts().unwrap().as_secs() as i64,
                };
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
                    return Err(String::from("no task in progress"));
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

    pub fn cancel(&self, task_name: String) -> Result<(), String> {
        match self.auth_manager.get_access_token() {
            Some(token) => {
                let api_resp = cancel_task(
                    format!("{}/punch/cancel", self.configs.api_endpoint),
                    token,
                    task_name,
                );
                return match api_resp {
                    Ok(_) => Ok(()),
                    Err(err) => Err(format!("{}", err)),
                };
            }
            None => {
                let started = get_unfinished_task(task_name.as_str(), self.db_conn);
                if started.len() == 0 {
                    return Err(String::from("no task in progress"));
                }
                return match diesel::delete(table.find(started[0].id)).execute(self.db_conn) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(format!("{}", err)),
                };
            }
        }
    }

    pub fn get(&self, task_name: String, since: i64, until: i64) -> Result<TaskStat, String> {
        match self.auth_manager.get_access_token() {
            Some(token) => {
                let api_resp = get_task(
                    format!(
                        "{}/punch/get/{}?since={}&until={}",
                        self.configs.api_endpoint,
                        task_name,
                        since.to_string(),
                        until.to_string(),
                    ),
                    token,
                );
                return match api_resp {
                    Ok(task_stat) => Ok(TaskStat {
                        name: task_stat.name,
                        status: task_stat.status,
                        duration: task_stat.duration,
                    }),
                    Err(err) => Err(format!("{}", err)),
                };
            }
            None => {
                return match tasks::table
                    .filter(name.eq(task_name.clone()))
                    .filter(finished_at.ge(since))
                    .or_filter(finished_at.is_null())
                    .filter(started_at.le(until))
                    .order(started_at.asc())
                    .load::<Task>(self.db_conn)
                {
                    Ok(tasks) => {
                        if tasks.len() == 0 {
                            return Err(format!("no task found for {}", task_name.to_owned()));
                        }
                        let sum: i64 = tasks
                            .iter()
                            .map(|task| match task.finished_at {
                                Some(fts) => {
                                    cmp::min(fts, until) - cmp::max(task.started_at, since)
                                }
                                None => until - cmp::max(task.started_at, since),
                            })
                            .fold(0, |a, b| a + b);
                        Ok(TaskStat {
                            name: task_name.clone(),
                            status: if tasks.iter().any(|task| task.finished_at.is_none()) {
                                "in progress".to_owned()
                            } else {
                                "complete".to_owned()
                            },
                            duration: sum,
                        })
                    }
                    Err(err) => Err(format!("{}", err)),
                }
            }
        }
    }

    pub fn list(&self) -> Result<Vec<TaskListItem>, String> {
        match self.auth_manager.get_access_token() {
            Some(token) => {
                let endpoint = format!("{}/punch/list", self.configs.api_endpoint,);
                let api_resp = list_task(&endpoint, &token);
                return match api_resp {
                    Ok(task_list) => Ok(task_list
                        .iter()
                        .map(|item| TaskListItem {
                            name: item.name.to_owned(),
                            started_at: item.started_at,
                            finished_at: item.finished_at,
                            duration: item.duration,
                        })
                        .collect()),
                    Err(err) => Err(format!("{}", err)),
                };
            }
            None => {
                let sqlite_op = sql_query(
                    "SELECT name, max(started_at) as started_at, case when count(*) - count(finished_at) > 0 then null else max(finished_at) end as finished_at, coalesce(sum(finished_at - started_at), 0) as duration FROM tasks GROUP BY name;",
                )
                    .load::<AggregatedTask>(self.db_conn);
                return match sqlite_op {
                    Ok(tasks) => Ok(tasks
                        .iter()
                        .map(|task| TaskListItem {
                            name: task.name.to_owned(),
                            started_at: task.started_at,
                            finished_at: task.finished_at,
                            duration: task.duration,
                        })
                        .collect()),
                    Err(err) => Err(format!("{}", err)),
                };
            }
        }
    }
}
