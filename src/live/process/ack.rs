use super::super::logging::log_error;
use crate::error::AppResult;
use crate::nats::StructuredIntelMessage;

pub(super) async fn ack_message(
    worker_run_id: &str,
    message: StructuredIntelMessage,
) -> AppResult<()> {
    if let Err(error) = message.ack().await {
        log_error("ack_failed", Some(worker_run_id), &error)?;
        return Err(error);
    }
    Ok(())
}
