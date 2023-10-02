use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

use chrono::{DateTime, Duration as ChronoDuration, Local, NaiveDateTime, Utc};
use hhmmss::Hhmmss;

pub fn utc_ts_to_local_datetime(utc_ts: i64) -> String {
    let ts_ndt = NaiveDateTime::from_timestamp_opt(i64::from(utc_ts), 0).unwrap();
    let utc_dt: DateTime<Utc> = DateTime::from_utc(ts_ndt, Utc);
    let dt: DateTime<Local> = DateTime::from(utc_dt);
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn seconds_to_duration(seconds: i64) -> String {
    ChronoDuration::seconds(seconds).hhmmss()
}

pub fn get_now_ts() -> Result<Duration, SystemTimeError> {
    SystemTime::now().duration_since(UNIX_EPOCH)
}
