use crate::model::{
    CANDIDATE_BUNDLE_SCHEMA_VERSION, HYPOTHESIS_STATE_SCHEMA_VERSION,
    SCREENING_EVENT_SCHEMA_VERSION,
};
use crate::time::{path_segment, time_part};

pub fn candidate_bundle_key(
    created_at_ms: i64,
    candidate_id: &str,
    priority_partition: &str,
) -> String {
    let part = time_part(created_at_ms);
    format!(
        "candidate-evidence-bundle/priority={}/schema={}/dt={}/hour={:02}/candidate_id={}/part-000001.jsonl",
        path_segment(priority_partition),
        CANDIDATE_BUNDLE_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(candidate_id)
    )
}

pub fn screening_event_key(created_at_ms: i64, screening_event_id: &str) -> String {
    let part = time_part(created_at_ms);
    format!(
        "candidate-screening/schema={}/dt={}/hour={:02}/screening_event_id={}/part-000001.jsonl",
        SCREENING_EVENT_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(screening_event_id)
    )
}

pub fn hypothesis_state_key(created_at_ms: i64, hypothesis_id: &str) -> String {
    let part = time_part(created_at_ms);
    format!(
        "hypothesis-state/schema={}/dt={}/hour={:02}/hypothesis_id={}/state.json",
        HYPOTHESIS_STATE_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(hypothesis_id)
    )
}
