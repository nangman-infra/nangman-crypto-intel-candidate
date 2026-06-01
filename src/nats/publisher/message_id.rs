use crate::nats::pointer::CandidateArtifactPointer;

pub(super) fn bundle_message_id(pointer: &CandidateArtifactPointer) -> &str {
    pointer
        .candidate_id
        .as_deref()
        .unwrap_or(pointer.screening_event_id.as_str())
}

#[cfg(test)]
mod tests {
    use super::bundle_message_id;
    use crate::nats::pointer::{CandidateArtifactPointer, S3ObjectPointer};

    #[test]
    fn bundle_message_id_prefers_candidate_id() {
        let mut pointer = pointer();
        pointer.candidate_id = Some("candidate-1".to_owned());

        assert_eq!(bundle_message_id(&pointer), "candidate-1");
    }

    #[test]
    fn bundle_message_id_falls_back_to_screening_event_id() {
        let pointer = pointer();

        assert_eq!(bundle_message_id(&pointer), "screening-1");
    }

    fn pointer() -> CandidateArtifactPointer {
        CandidateArtifactPointer {
            schema_version: "intel_candidate_artifact_pointer_v1".to_owned(),
            artifact_family: "intel_candidate_evidence_bundle".to_owned(),
            candidate_id: None,
            screening_event_id: "screening-1".to_owned(),
            candidate_class: "approved".to_owned(),
            storage_ref: S3ObjectPointer {
                bucket: "bucket".to_owned(),
                key: "key".to_owned(),
                content_sha256: "sha256:abc".to_owned(),
                schema_version: "intel_candidate_evidence_bundle_v1".to_owned(),
            },
            created_at_ms: 1,
        }
    }
}
