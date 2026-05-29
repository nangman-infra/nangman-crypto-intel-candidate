use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RunSummary {
    pub processed_packets: usize,
    pub evidence_bundles_created: usize,
    pub hypothesis_states_created: usize,
    pub output_files: Vec<String>,
}
