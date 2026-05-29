use super::*;

impl CandidateWorker {
    pub(super) async fn write_and_publish_result(
        &self,
        result: &CandidateProcessingResult,
    ) -> AppResult<()> {
        let screening_key = screening_event_key(
            result.screening_event.created_at_ms,
            &result.screening_event.screening_event_id,
        );
        let screening_bytes = self
            .output_store
            .put_jsonl_record_or_existing(&screening_key, &result.screening_event)
            .await?;
        let screening_pointer = CandidateArtifactPointer {
            schema_version: CANDIDATE_POINTER_SCHEMA_VERSION.to_owned(),
            artifact_family: "intel_candidate_screening_event".to_owned(),
            candidate_id: result.screening_event.candidate_id.clone(),
            screening_event_id: result.screening_event.screening_event_id.clone(),
            candidate_class: result
                .screening_event
                .candidate_class
                .as_policy_key()
                .to_owned(),
            storage_ref: S3ObjectPointer {
                bucket: self.output_store.bucket().to_owned(),
                key: screening_key,
                content_sha256: sha256_prefixed(&screening_bytes),
                schema_version: result.screening_event.schema_version.clone(),
            },
            created_at_ms: result.screening_event.created_at_ms,
        };
        self.publisher
            .publish_screening_pointer(&screening_pointer)
            .await?;

        if let Some(bundle) = &result.evidence_bundle {
            let bundle_bytes = self
                .output_store
                .put_jsonl_record_or_existing(&bundle.bundle_key, bundle)
                .await?;
            let bundle_pointer = CandidateArtifactPointer {
                schema_version: CANDIDATE_POINTER_SCHEMA_VERSION.to_owned(),
                artifact_family: "intel_candidate_evidence_bundle".to_owned(),
                candidate_id: Some(bundle.candidate_id.clone()),
                screening_event_id: result.screening_event.screening_event_id.clone(),
                candidate_class: bundle.candidate_class.as_policy_key().to_owned(),
                storage_ref: S3ObjectPointer {
                    bucket: self.output_store.bucket().to_owned(),
                    key: bundle.bundle_key.clone(),
                    content_sha256: sha256_prefixed(&bundle_bytes),
                    schema_version: bundle.schema_version.clone(),
                },
                created_at_ms: bundle.created_at_ms,
            };
            self.publisher
                .publish_bundle_pointer(&bundle_pointer)
                .await?;
        }
        if let Some(state) = &result.hypothesis_state {
            let state_bytes = self
                .output_store
                .put_jsonl_record_or_existing(&state.state_key, state)
                .await?;
            let state_pointer = CandidateArtifactPointer {
                schema_version: CANDIDATE_POINTER_SCHEMA_VERSION.to_owned(),
                artifact_family: "intel_candidate_hypothesis_state".to_owned(),
                candidate_id: Some(state.hypothesis_id.clone()),
                screening_event_id: result.screening_event.screening_event_id.clone(),
                candidate_class: state.current_state.as_policy_key().to_owned(),
                storage_ref: S3ObjectPointer {
                    bucket: self.output_store.bucket().to_owned(),
                    key: state.state_key.clone(),
                    content_sha256: sha256_prefixed(&state_bytes),
                    schema_version: state.schema_version.clone(),
                },
                created_at_ms: state.updated_at_ms,
            };
            self.publisher
                .publish_hypothesis_state_pointer(&state_pointer)
                .await?;
        }
        self.publisher.flush().await
    }
}
