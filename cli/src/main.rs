#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

pub mod api;
pub mod auth;
pub mod configs;
pub mod db;
pub mod keyring;
pub mod model;
pub mod puncher;
pub mod schema;
pub mod utils;

use crate::db::create_connection;
use crate::model::*;
use crate::utils::{seconds_to_duration, utc_ts_to_local_datetime, write_tab_written_message};

use crate::auth::AuthManager;
use crate::configs::fetch_configs;
use crate::keyring::new_key_ring_manager;
use crate::puncher::Puncher;
use ansi_term::Colour::{Cyan, Green, Purple, Red, Yellow};
use chrono::{DateTime, Utc};
use clap::{arg, Command};
use dateparser;
use diesel_migrations::embed_migrations;
use std::time::{SystemTime, UNIX_EPOCH};

embed_migrations!("./migrations");

fn main() -> Result<(), std::io::Error> {
    let conn = create_connection().unwrap();
    let cf = fetch_configs();
    let sm = new_key_ring_manager();
    let mut am = AuthManager::new(&cf, sm);
    am.initialize();
    let puncher = Puncher::new(&am, &cf, &conn);

    let matches = Command::new("Punch CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("in")
                .about("start a new task")
                .arg(arg!([NAME])),
        )
        .subcommand(
            Command::new("out")
                .about("finish a task that is in progress")
                .arg(arg!([NAME])),
        )
        .subcommand(
            Command::new("cancel")
                .about("cancel a task")
                .arg(arg!([NAME])),
        )
        .subcommand(
            Command::new("get")
                .about("prints how much time you spent for a given task")
                .arg(arg!([NAME]))
                .arg(arg!(--since[SINCE_TS]))
                .arg(arg!(--until[UNTIL_TS])),
        )
        .subcommand(Command::new("list").about("list all tasks and their status"))
        .get_matches();

    embedded_migrations::run(&conn).unwrap();
    embedded_migrations::run_with_output(&conn, &mut std::io::stdout()).unwrap();

    match matches.subcommand() {
        Some(("in", sub_matches)) => {
            let task_name = sub_matches.value_of("NAME").unwrap();
            match puncher.punch_in(task_name.to_owned()) {
                Ok(timestamp) => {
                    write_tab_written_message(format!(
                        "{}\n{}\t{}",
                        Cyan.paint("name\tstarted at"),
                        task_name,
                        Green.paint(utc_ts_to_local_datetime(timestamp)),
                    ));
                }
                Err(err) => {
                    println!("{} {}", Red.paint("ERROR:"), Cyan.paint(err),);
                    std::process::exit(1);
                }
            };
        }
        Some(("out", sub_matches)) => {
            let task_name = sub_matches.value_of("NAME").unwrap();
            match puncher.punch_out(task_name.to_owned()) {
                Ok(task) => {
                    // TODO: this needs better error handling
                    let ts = task.finished_at.unwrap();
                    write_tab_written_message(format!(
                        "{}\n{}\t{}\t{}",
                        Cyan.paint("name\tfinished at\ttime spent"),
                        task_name,
                        Green.paint(utc_ts_to_local_datetime(ts)),
                        Yellow.paint(seconds_to_duration(ts - task.started_at)),
                    ));
                }
                Err(err) => {
                    println!("{} {}", Red.paint("ERROR:"), Cyan.paint(err));
                    std::process::exit(1);
                }
            };
        }
        Some(("cancel", sub_matches)) => {
            let task_name = sub_matches.value_of("NAME").unwrap();
            match puncher.cancel(task_name.to_string()) {
                Ok(_) => {
                    println!("Cancelled {}", Cyan.paint(task_name));
                }
                Err(err) => {
                    println!("{} {}", Red.paint("ERROR:"), Cyan.paint(err));
                    std::process::exit(1);
                }
            };
        }
        Some(("list", _)) => match puncher.list() {
            Ok(tasks) => {
                write_tab_written_message(
                    tasks
                        .iter()
                        .map(|task| {
                            let duration = Purple.paint(seconds_to_duration(task.duration));
                            return match task.finished_at {
                                Some(_) => {
                                    format!(
                                        "{}\t({})\t{}\t{}",
                                        task.name,
                                        Green.paint("complete"),
                                        String::new(),
                                        duration,
                                    )
                                }
                                None => {
                                    let now_dt = get_ts().unwrap();
                                    let now_ts = now_dt.as_secs() as i64;
                                    format!(
                                        "{}\t({})\t{}\t{}",
                                        task.name,
                                        Red.paint("in progress"),
                                        Yellow.paint(seconds_to_duration(now_ts - task.started_at)),
                                        duration,
                                    )
                                }
                            };
                        })
                        .fold(
                            Cyan.paint(
                                "name\tstatus\tcurrent total\ttotal (minus current total)\n",
                            )
                            .to_string(),
                            |a, b| a + &b + "\n",
                        ),
                );
            }
            Err(err) => {
                println!("{} {}", Red.paint("ERROR:"), Cyan.paint(err));
                std::process::exit(1);
            }
        },
        Some(("get", sub_matches)) => {
            if !sub_matches.is_present("NAME") {
                panic!("you must provide the task name")
            }
            let task_name = sub_matches.value_of("NAME").unwrap();
            let epoch_dt: DateTime<Utc> = UNIX_EPOCH.into();
            let now_dt: DateTime<Utc> = SystemTime::now().into();
            let since_dt: DateTime<Utc> = if sub_matches.is_present("since") {
                let since_arg = sub_matches
                    .value_of("since")
                    .unwrap_or("1970-01-01 00:00 UTC");
                match dateparser::parse(since_arg) {
                    Ok(dt) => dt,
                    Err(_) => panic!("failed to parse 'since' time value"),
                }
            } else {
                epoch_dt
            };
            let until_dt: DateTime<Utc> = if sub_matches.is_present("until") {
                let until_default = now_dt.to_string();
                let until_arg = sub_matches.value_of("since").unwrap_or(&until_default);
                match dateparser::parse(until_arg) {
                    Ok(dt) => dt,
                    Err(_) => panic!("failed to parse 'until' time value"),
                }
            } else {
                now_dt
            };
            let since = since_dt.signed_duration_since(epoch_dt).num_seconds();
            let until = until_dt.signed_duration_since(epoch_dt).num_seconds();
            match puncher.get(task_name.to_owned(), since, until) {
                Ok(stat) => write_tab_written_message(format!(
                    "{}\n{}\t({})\t{}",
                    Cyan.paint("name\tstatus\ttime spent"),
                    stat.name,
                    if stat.status == "in progress" {
                        Red.paint("in progress")
                    } else {
                        Green.paint("complete")
                    },
                    Yellow.paint(seconds_to_duration(stat.duration)),
                )),
                Err(err) => {
                    println!("{} {}", Red.paint("ERROR:"), Cyan.paint(err));
                    std::process::exit(1);
                }
            }
        }
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
    Ok(())
}
