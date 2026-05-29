use super::CandidateWorker;
use super::content::{read_single_json_or_jsonl, validate_pointer_content_hash};
use super::market::repair_raw_event_id;
use crate::error::{AppError, AppResult};
use crate::model::{CandidateProcessingResult, StructuredIntelPacket};
use crate::nats::StructuredPointer;
use std::path::Path;

impl CandidateWorker {
    pub async fn process_pointer(
        &self,
        pointer: &StructuredPointer,
        created_at_ms: i64,
    ) -> AppResult<Option<CandidateProcessingResult>> {
        pointer.validate()?;
        if pointer.storage_ref.bucket != self.input_store.bucket() {
            return Err(AppError::validation(format!(
                "structured pointer bucket mismatch expected={} actual={}",
                self.input_store.bucket(),
                pointer.storage_ref.bucket
            )));
        }
        let packet_bytes = self.input_store.get_bytes(&pointer.storage_ref.key).await?;
        validate_pointer_content_hash(pointer, &packet_bytes)?;
        let packet: StructuredIntelPacket =
            read_single_json_or_jsonl(&packet_bytes, Path::new(&pointer.storage_ref.key))?;
        if self.is_stale_revision(&packet).await? {
            return Ok(None);
        }
        let result = self.score_packet(packet.clone(), created_at_ms).await?;
        self.write_and_publish_result(&result).await?;
        self.write_revision_index(&packet, &result, created_at_ms)
            .await?;
        Ok(Some(result))
    }

    pub async fn process_s3_key(
        &self,
        key: &str,
        created_at_ms: i64,
    ) -> AppResult<Option<CandidateProcessingResult>> {
        let packet_bytes = self.input_store.get_bytes(key).await?;
        let mut packet: StructuredIntelPacket =
            read_single_json_or_jsonl(&packet_bytes, Path::new(key))?;
        if packet.raw_event_id.trim().is_empty() {
            packet.raw_event_id = repair_raw_event_id(&packet, key);
        }
        if self.is_stale_revision(&packet).await? {
            return Ok(None);
        }
        let result = self.score_packet(packet.clone(), created_at_ms).await?;
        self.write_and_publish_result(&result).await?;
        self.write_revision_index(&packet, &result, created_at_ms)
            .await?;
        Ok(Some(result))
    }
}
