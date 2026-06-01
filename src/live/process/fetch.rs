use super::super::logging::log_error;
use crate::error::AppResult;
use crate::nats::StructuredIntelMessage;
use std::time::Duration;

pub async fn handle_fetch_result(
    message_result: AppResult<Option<StructuredIntelMessage>>,
    worker_run_id: &str,
) -> AppResult<Option<StructuredIntelMessage>> {
    match message_result {
        Ok(message) => Ok(message),
        Err(error) => {
            log_error("fetch_failed", Some(worker_run_id), &error)?;
            tokio::time::sleep(Duration::from_secs(1)).await;
            Ok(None)
        }
    }
}
