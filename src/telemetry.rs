use crate::error::AppResult;
use crate::model::PRODUCER_APP;
use crate::time::now_ms;
use serde::Serialize;
use serde_json::{Map, Value};

pub const WORKER_LOG_SCHEMA_VERSION: &str = "intel_candidate_worker_log_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

pub fn info(event: &str, fields: impl Serialize) -> AppResult<()> {
    println!("{}", log_line(LogLevel::Info, event, fields)?);
    Ok(())
}

pub fn warn(event: &str, fields: impl Serialize) -> AppResult<()> {
    println!("{}", log_line(LogLevel::Warn, event, fields)?);
    Ok(())
}

pub fn error(event: &str, fields: impl Serialize) -> AppResult<()> {
    eprintln!("{}", log_line(LogLevel::Error, event, fields)?);
    Ok(())
}

pub fn log_line(level: LogLevel, event: &str, fields: impl Serialize) -> AppResult<String> {
    let mut record = Map::new();
    record.insert(
        "schema_version".to_owned(),
        Value::String(WORKER_LOG_SCHEMA_VERSION.to_owned()),
    );
    record.insert(
        "producer_app".to_owned(),
        Value::String(PRODUCER_APP.to_owned()),
    );
    record.insert("timestamp_ms".to_owned(), Value::from(now_ms()));
    record.insert("level".to_owned(), Value::String(level.as_str().to_owned()));
    record.insert("event".to_owned(), Value::String(event.to_owned()));

    match serde_json::to_value(fields)? {
        Value::Object(fields) => {
            for (key, value) in fields {
                if !record.contains_key(&key) {
                    record.insert(key, value);
                }
            }
        }
        value => {
            record.insert("fields".to_owned(), value);
        }
    }

    Ok(Value::Object(record).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn log_line_contains_stable_schema_fields() {
        let line = log_line(
            LogLevel::Info,
            "message_acked",
            json!({
                "worker_run_id": "run-1",
                "packet_id": "packet-1",
                "ack": "yes"
            }),
        )
        .expect("log line should serialize");
        let value: Value = serde_json::from_str(&line).expect("log line should be valid json");
        assert_eq!(
            value["schema_version"],
            Value::String(WORKER_LOG_SCHEMA_VERSION.to_owned())
        );
        assert_eq!(
            value["producer_app"],
            Value::String(PRODUCER_APP.to_owned())
        );
        assert_eq!(value["level"], Value::String("info".to_owned()));
        assert_eq!(value["event"], Value::String("message_acked".to_owned()));
        assert_eq!(value["worker_run_id"], Value::String("run-1".to_owned()));
        assert_eq!(value["ack"], Value::String("yes".to_owned()));
        assert!(value["timestamp_ms"].is_i64());
    }
}
