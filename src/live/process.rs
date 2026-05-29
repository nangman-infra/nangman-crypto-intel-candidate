use super::logging::log_error;
use crate::error::AppResult;
use crate::nats::StructuredIntelMessage;
use crate::scoring::screening_event_key;
use crate::telemetry;
use crate::time::now_ms;
use crate::worker::CandidateWorker;
use serde_json::json;

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
    telemetry::info(
        "message_received",
        json!({
            "worker_run_id": worker_run_id,
            "packet_id": pointer.packet_id,
            "raw_event_id": pointer.raw_event_id,
            "input_s3_bucket": pointer.storage_ref.bucket,
            "input_s3_key": pointer.storage_ref.key,
            "input_schema_version": pointer.storage_ref.schema_version,
            "manifest_key": pointer.manifest_key,
            "pointer_created_at_ms": pointer.created_at_ms,
        }),
    )?;
    match worker.process_pointer(&pointer, now_ms()).await {
        Ok(Some(result)) => {
            let screening_s3_key = screening_event_key(
                result.screening_event.created_at_ms,
                &result.screening_event.screening_event_id,
            );
            let evidence_bundle_s3_key = result
                .evidence_bundle
                .as_ref()
                .map(|bundle| bundle.bundle_key.as_str());
            telemetry::info(
                "candidate_result_published",
                json!({
                    "worker_run_id": worker_run_id,
                    "packet_id": pointer.packet_id,
                    "raw_event_id": pointer.raw_event_id,
                    "screening_event_id": result.screening_event.screening_event_id,
                    "candidate_id": result.screening_event.candidate_id,
                    "candidate_class": result.screening_event.candidate_class.as_policy_key(),
                    "candidate_score": result.screening_event.candidate_score,
                    "research_eligible": result.screening_event.research_eligible,
                    "quarantine": result.screening_event.quarantine,
                    "screening_s3_key": screening_s3_key,
                    "evidence_bundle_created": evidence_bundle_s3_key.is_some(),
                    "evidence_bundle_s3_key": evidence_bundle_s3_key,
                    "reason_count": result.screening_event.reasons.len(),
                    "ack": "pending",
                }),
            )?;
            if let Err(error) = message.ack().await {
                log_error("ack_failed", Some(worker_run_id), &error)?;
                return Err(error);
            }
            telemetry::info(
                "message_acked",
                json!({
                    "worker_run_id": worker_run_id,
                    "packet_id": pointer.packet_id,
                    "raw_event_id": pointer.raw_event_id,
                    "screening_event_id": result.screening_event.screening_event_id,
                    "ack": "yes",
                }),
            )?;
        }
        Ok(None) => {
            telemetry::info(
                "message_skipped_as_stale_revision",
                json!({
                    "worker_run_id": worker_run_id,
                    "packet_id": pointer.packet_id,
                    "raw_event_id": pointer.raw_event_id,
                    "ack": "pending",
                }),
            )?;
            if let Err(error) = message.ack().await {
                log_error("ack_failed", Some(worker_run_id), &error)?;
                return Err(error);
            }
            telemetry::info(
                "message_acked",
                json!({
                    "worker_run_id": worker_run_id,
                    "packet_id": pointer.packet_id,
                    "raw_event_id": pointer.raw_event_id,
                    "ack": "yes",
                    "skipped": "stale_revision",
                }),
            )?;
        }
        Err(error) => {
            telemetry::error(
                "processing_failed",
                json!({
                    "worker_run_id": worker_run_id,
                    "packet_id": pointer.packet_id,
                    "raw_event_id": pointer.raw_event_id,
                    "input_s3_bucket": pointer.storage_ref.bucket,
                    "input_s3_key": pointer.storage_ref.key,
                    "ack": "no",
                    "error": error.to_string(),
                }),
            )?;
        }
    }
    Ok(())
}

pub async fn handle_fetch_result(
    message_result: AppResult<Option<StructuredIntelMessage>>,
    worker_run_id: &str,
) -> AppResult<Option<StructuredIntelMessage>> {
    match message_result {
        Ok(message) => Ok(message),
        Err(error) => {
            log_error("fetch_failed", Some(worker_run_id), &error)?;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            Ok(None)
        }
    }
}
