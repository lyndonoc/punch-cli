use chrono::{DateTime, Duration, Local, NaiveDateTime, Utc};
use hhmmss::Hhmmss;
use std::io::Write;
use tabwriter::TabWriter;

pub fn utc_ts_to_local_datetime(utc_ts: i64) -> String {
    let ts_ndt = NaiveDateTime::from_timestamp(i64::from(utc_ts), 0);
    let utc_dt: DateTime<Utc> = DateTime::from_utc(ts_ndt, Utc);
    let dt: DateTime<Local> = DateTime::from(utc_dt);
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn seconds_to_duration(seconds: i64) -> String {
    Duration::seconds(seconds).hhmmss()
}

pub fn write_tab_written_message(message: String) {
    let mut tw = TabWriter::new(vec![]);
    tw.write_all(message.as_bytes()).unwrap();
    tw.flush().unwrap();
    println!("{}", String::from_utf8(tw.into_inner().unwrap()).unwrap());
}
