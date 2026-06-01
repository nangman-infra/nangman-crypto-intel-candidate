use crate::error::{AppError, AppResult};
use crate::model::CandidateProcessingResult;
use crate::nats::StructuredPointer;
use crate::scoring::screening_event_key;
use crate::telemetry;
use serde_json::json;

pub(super) fn log_message_received(
    worker_run_id: &str,
    pointer: &StructuredPointer,
) -> AppResult<()> {
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
    )
}

pub(super) fn log_candidate_result_published(
    worker_run_id: &str,
    pointer: &StructuredPointer,
    result: &CandidateProcessingResult,
) -> AppResult<()> {
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
    )
}

pub(super) fn log_published_message_acked(
    worker_run_id: &str,
    pointer: &StructuredPointer,
    result: &CandidateProcessingResult,
) -> AppResult<()> {
    telemetry::info(
        "message_acked",
        json!({
            "worker_run_id": worker_run_id,
            "packet_id": pointer.packet_id,
            "raw_event_id": pointer.raw_event_id,
            "screening_event_id": result.screening_event.screening_event_id,
            "ack": "yes",
        }),
    )
}

pub(super) fn log_stale_revision_skipped(
    worker_run_id: &str,
    pointer: &StructuredPointer,
) -> AppResult<()> {
    telemetry::info(
        "message_skipped_as_stale_revision",
        json!({
            "worker_run_id": worker_run_id,
            "packet_id": pointer.packet_id,
            "raw_event_id": pointer.raw_event_id,
            "ack": "pending",
        }),
    )
}

pub(super) fn log_stale_message_acked(
    worker_run_id: &str,
    pointer: &StructuredPointer,
) -> AppResult<()> {
    telemetry::info(
        "message_acked",
        json!({
            "worker_run_id": worker_run_id,
            "packet_id": pointer.packet_id,
            "raw_event_id": pointer.raw_event_id,
            "ack": "yes",
            "skipped": "stale_revision",
        }),
    )
}

pub(super) fn log_processing_failed(
    worker_run_id: &str,
    pointer: &StructuredPointer,
    error: &AppError,
) -> AppResult<()> {
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
    )
}
