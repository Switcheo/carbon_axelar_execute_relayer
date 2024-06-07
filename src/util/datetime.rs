use chrono::{DateTime, Duration, Utc};
use pbjson_types::Timestamp;

pub fn timestamp_to_datetime(timestamp: &Timestamp) -> DateTime<Utc> {
    let seconds = timestamp.seconds;
    let nanos = timestamp.nanos;

    let naive_datetime = chrono::NaiveDateTime::from_timestamp_opt(seconds, nanos as u32).unwrap();
    DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc)
}

pub fn time_difference_str(duration: Duration) -> String {
    let seconds = duration.num_seconds().abs();
    let minutes = duration.num_minutes().abs();
    let hours = duration.num_hours().abs();
    let days = duration.num_days().abs();

    if days > 0 {
        return format!("~ {} days", days)
    } else if hours > 0 {
        return format!("~ {} hours", hours)
    } else if minutes > 0 {
        return format!("~ {} minutes", minutes)
    } else {
        return format!("~ {} seconds", seconds)
    }
}