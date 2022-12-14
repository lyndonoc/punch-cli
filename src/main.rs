#![allow(unused_imports)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

extern crate chrono;

use ansi_term::Colour::{Cyan, Green, Purple, Red, Yellow};

use chrono::{DateTime, Local, NaiveDateTime, Timelike, Utc};

use clap::{arg, Command};

use dateparser;

use db::schema::tasks::{finished_at, name, started_at};
use db::{
    create_connection, model::*, schema::*, seconds_to_duration, utc_ts_to_local_datetime,
    write_tab_written_message,
};

use diesel::dsl::max;
use diesel::prelude::*;
use diesel::sql_query;
use diesel_migrations::embed_migrations;

use std::cmp;
use std::convert::TryInto;
use std::io::Write;
use std::ptr::write;
use std::time::{SystemTime, UNIX_EPOCH};

use tabwriter::TabWriter;

embed_migrations!("./migrations");

fn main() -> Result<(), std::io::Error> {
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

    let conn = create_connection().unwrap();

    embedded_migrations::run(&conn).unwrap();
    embedded_migrations::run_with_output(&conn, &mut std::io::stdout()).unwrap();

    match matches.subcommand() {
        Some(("in", sub_matches)) => {
            let task_name = sub_matches.value_of("NAME").unwrap();
            let unfinished = get_unfinished_task(task_name, &conn);
            if unfinished.len() > 0 {
                println!(
                    "{} task {} already started_at {}",
                    Red.paint("ERROR:"),
                    Cyan.paint(task_name),
                    Red.paint(utc_ts_to_local_datetime(unfinished[0].started_at)),
                );
                std::process::exit(1);
            }
            let new_task = new_task(task_name);
            diesel::insert_into(tasks::table)
                .values(&new_task)
                .execute(&conn)
                .unwrap();
            write_tab_written_message(format!(
                "{}\n{}\t{}",
                Cyan.paint("name\tstarted at"),
                task_name,
                Green.paint(utc_ts_to_local_datetime(new_task.started_at)),
            ));
        }
        Some(("out", sub_matches)) => {
            let task_name = sub_matches.value_of("NAME").unwrap();
            let existing = get_unfinished_task(task_name, &conn);
            if existing.len() == 0 {
                println!(
                    "{} no task in progress for {}",
                    Red.paint("ERROR:"),
                    Cyan.paint(task_name),
                );
                std::process::exit(1);
            }
            let finished_ts = get_ts().unwrap().as_secs() as i64;
            diesel::update(tasks::table.find(existing[0].id))
                .set(finished_at.eq(finished_ts))
                .execute(&conn)
                .unwrap();
            write_tab_written_message(format!(
                "{}\n{}\t{}\t{}",
                Cyan.paint("name\tfinished at\ttime spent"),
                task_name,
                Green.paint(utc_ts_to_local_datetime(finished_ts)),
                Yellow.paint(seconds_to_duration(finished_ts - existing[0].started_at)),
            ));
        }
        Some(("cancel", sub_matches)) => {
            let task_name = sub_matches.value_of("NAME").unwrap();
            let started = get_unfinished_task(task_name, &conn);
            if started.len() == 0 {
                println!(
                    "{} no task in progress for {}",
                    Red.paint("ERROR:"),
                    Cyan.paint(task_name),
                );
                std::process::exit(1);
            }
            diesel::delete(tasks::table.find(started[0].id))
                .execute(&conn)
                .unwrap();
            println!("cancelled {}", Cyan.paint(task_name));
        }
        Some(("list", _)) => {
            let tasks: Vec<AggregatedTask> = sql_query(
                "SELECT name, max(started_at) as started_at, case when count(*) - count(finished_at) > 0 then null else max(finished_at) end as finished_at, sum(finished_at - started_at) as duration FROM tasks GROUP BY name;",
            )
            .load(&conn)
            .unwrap();
            write_tab_written_message(
                tasks
                    .iter()
                    .map(|task| {
                        let duration =
                            Purple.paint(seconds_to_duration(task.duration.unwrap_or_default()));
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
                        Cyan.paint("name\tstatus\tcurrent total\ttotal (minus current total)\n")
                            .to_string(),
                        |a, b| a + &b + "\n",
                    ),
            );
        }
        Some(("get", sub_matches)) => {
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
            let tasks = tasks::table
                .filter(name.eq(task_name))
                .filter(finished_at.ge(since))
                .or_filter(finished_at.is_null())
                .filter(started_at.le(until))
                .order(started_at.asc())
                .load::<Task>(&conn)
                .unwrap();
            if tasks.len() == 0 {
                println!(
                    "{} no task found for {}",
                    Red.paint("ERROR:"),
                    Cyan.paint(task_name),
                );
                std::process::exit(1);
            }
            let sum: i64 = tasks
                .iter()
                .map(|task| match task.finished_at {
                    Some(fts) => cmp::min(fts, until) - cmp::max(task.started_at, since),
                    None => until - cmp::max(task.started_at, since),
                })
                .fold(0, |a, b| a + b);
            write_tab_written_message(format!(
                "{}\n{}\t({})\t{}\t{}\t{}",
                Cyan.paint("name\tstatus\ttime spent\tfrom\tto"),
                task_name,
                if tasks.iter().any(|task| task.finished_at.is_none()) {
                    Red.paint("in progress")
                } else {
                    Green.paint("complete")
                },
                Yellow.paint(seconds_to_duration(sum)),
                Purple.paint(utc_ts_to_local_datetime(if tasks.len() > 0 {
                    cmp::max(since, tasks[0].started_at)
                } else {
                    since
                })),
                Purple.paint(utc_ts_to_local_datetime(if tasks.len() > 0 {
                    match tasks[tasks.len() - 1].finished_at {
                        Some(fa) => cmp::min(until, fa),
                        None => until,
                    }
                } else {
                    until
                }),)
            ));
        }
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
    Ok(())
}
