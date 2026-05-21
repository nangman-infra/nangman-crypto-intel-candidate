use chrono::{DateTime, Datelike, Timelike, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimePart {
    pub event_date: String,
    pub hour: u32,
}

pub fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn hour_bucket_ms(value: i64) -> i64 {
    value - value.rem_euclid(60 * 60 * 1000)
}

pub fn time_part(timestamp_ms: i64) -> TimePart {
    let datetime =
        DateTime::<Utc>::from_timestamp_millis(timestamp_ms).unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    TimePart {
        event_date: format!(
            "{:04}-{:02}-{:02}",
            datetime.year(),
            datetime.month(),
            datetime.day()
        ),
        hour: datetime.hour(),
    }
}

pub fn path_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_utc_time_part() {
        let part = time_part(0);
        assert_eq!(part.event_date, "1970-01-01");
        assert_eq!(part.hour, 0);
    }
}
