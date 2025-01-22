use chrono::TimeZone;
use chrono::{DateTime, Datelike, Timelike, Utc};
use sqlx::types::time::{Date, PrimitiveDateTime, Time};
use time::Month;

pub fn chrono_to_primitive_datetime(datetime: DateTime<Utc>) -> PrimitiveDateTime {
    let naive_datetime = datetime.naive_utc();
    let month = Month::try_from(naive_datetime.month() as u8).expect("Invalid month value");
    let date = Date::from_calendar_date(naive_datetime.year(), month, naive_datetime.day() as u8)
        .expect("Invalid date");
    let time = Time::from_hms_micro(
        naive_datetime.hour() as u8,
        naive_datetime.minute() as u8,
        naive_datetime.second() as u8,
        naive_datetime.and_utc().timestamp_subsec_micros(),
    )
    .expect("Invalid time");

    PrimitiveDateTime::new(date, time)
}
pub fn convert_timestamp_to_string(timestamp: u64) -> String {
    let datetime = Utc.timestamp_opt(timestamp as i64, 0).unwrap();
    datetime.to_rfc3339()
}
