use chrono::{TimeZone, Utc};

pub fn convert_timestamp_to_string(timestamp: u64) -> String {
    let datetime = Utc.timestamp_opt(timestamp as i64, 0).unwrap();
    datetime.to_rfc3339()
}
