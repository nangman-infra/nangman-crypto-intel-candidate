use super::super::logging::log_error;
use super::ack::ack_message;
use super::events::{
    log_candidate_result_published, log_message_received, log_processing_failed,
    log_published_message_acked, log_stale_message_acked, log_stale_revision_skipped,
};
use crate::error::AppResult;
use crate::nats::StructuredIntelMessage;
use crate::time::now_ms;
use crate::worker::CandidateWorker;

pub async fn process_and_ack_message(
    worker_run_id: &str,
    worker: &CandidateWorker,
    message: StructuredIntelMessage,
) -> AppResult<()> {
    let pointer = match message.pointer() {
        Ok(pointer) => pointer,
        Err(error) => {
            log_error("pointer_parse_failed", Some(worker_run_id), &error)?;
            return Ok(());
        }
    };
    log_message_received(worker_run_id, &pointer)?;

    match worker.process_pointer(&pointer, now_ms()).await {
        Ok(Some(result)) => {
            log_candidate_result_published(worker_run_id, &pointer, &result)?;
            ack_message(worker_run_id, message).await?;
            log_published_message_acked(worker_run_id, &pointer, &result)?;
        }
        Ok(None) => {
            log_stale_revision_skipped(worker_run_id, &pointer)?;
            ack_message(worker_run_id, message).await?;
            log_stale_message_acked(worker_run_id, &pointer)?;
        }
        Err(error) => {
            log_processing_failed(worker_run_id, &pointer, &error)?;
        }
    }
    Ok(())
}
