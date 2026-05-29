use crate::error::{AppError, AppResult};
use crate::model::{
    CANDIDATE_BUNDLE_SCHEMA_VERSION, CANDIDATE_POINTER_SCHEMA_VERSION,
    HYPOTHESIS_STATE_SCHEMA_VERSION, SCREENING_EVENT_SCHEMA_VERSION,
    STRUCTURED_PACKET_SCHEMA_VERSION, STRUCTURED_POINTER_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct StructuredPointer {
    pub schema_version: String,
    pub packet_id: String,
    pub raw_event_id: String,
    pub terminal_decision: serde_json::Value,
    pub storage_ref: S3ObjectPointer,
    pub manifest_key: String,
    pub created_at_ms: i64,
}

impl StructuredPointer {
    pub fn validate(&self) -> AppResult<()> {
        if self.schema_version != STRUCTURED_POINTER_SCHEMA_VERSION {
            return Err(AppError::validation(format!(
                "structured pointer schema mismatch expected={} actual={}",
                STRUCTURED_POINTER_SCHEMA_VERSION, self.schema_version
            )));
        }
        if self.packet_id.trim().is_empty() {
            return Err(AppError::validation(
                "structured pointer packet_id is required",
            ));
        }
        if self.raw_event_id.trim().is_empty() {
            return Err(AppError::validation(
                "structured pointer raw_event_id is required",
            ));
        }
        if self.manifest_key.trim().is_empty() {
            return Err(AppError::validation(
                "structured pointer manifest_key is required",
            ));
        }
        self.storage_ref
            .validate(STRUCTURED_PACKET_SCHEMA_VERSION)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct S3ObjectPointer {
    pub bucket: String,
    pub key: String,
    pub content_sha256: String,
    pub schema_version: String,
}

impl S3ObjectPointer {
    fn validate(&self, expected_schema_version: &str) -> AppResult<()> {
        if self.bucket.trim().is_empty() {
            return Err(AppError::validation("pointer storage bucket is required"));
        }
        if self.key.trim().is_empty() {
            return Err(AppError::validation("pointer storage key is required"));
        }
        if !self.content_sha256.starts_with("sha256:") {
            return Err(AppError::validation(
                "pointer storage content_sha256 must be sha256-prefixed",
            ));
        }
        if self.schema_version != expected_schema_version {
            return Err(AppError::validation(format!(
                "pointer storage schema mismatch expected={} actual={}",
                expected_schema_version, self.schema_version
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CandidateArtifactPointer {
    pub schema_version: String,
    pub artifact_family: String,
    pub candidate_id: Option<String>,
    pub screening_event_id: String,
    pub candidate_class: String,
    pub storage_ref: S3ObjectPointer,
    pub created_at_ms: i64,
}

impl CandidateArtifactPointer {
    pub(super) fn validate(&self) -> AppResult<()> {
        if self.schema_version != CANDIDATE_POINTER_SCHEMA_VERSION {
            return Err(AppError::validation(format!(
                "candidate pointer schema mismatch expected={} actual={}",
                CANDIDATE_POINTER_SCHEMA_VERSION, self.schema_version
            )));
        }
        if self.screening_event_id.trim().is_empty() {
            return Err(AppError::validation(
                "candidate pointer screening_event_id is required",
            ));
        }
        if self.candidate_class.trim().is_empty() {
            return Err(AppError::validation(
                "candidate pointer candidate_class is required",
            ));
        }
        let expected_storage_schema = match self.artifact_family.as_str() {
            "intel_candidate_screening_event" => SCREENING_EVENT_SCHEMA_VERSION,
            "intel_candidate_evidence_bundle" => CANDIDATE_BUNDLE_SCHEMA_VERSION,
            "intel_candidate_hypothesis_state" => HYPOTHESIS_STATE_SCHEMA_VERSION,
            other => {
                return Err(AppError::validation(format!(
                    "unsupported candidate artifact family: {other}"
                )));
            }
        };
        self.storage_ref.validate(expected_storage_schema)?;
        Ok(())
    }
}
