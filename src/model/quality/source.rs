use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SourceIndependenceSummary {
    pub source_event_count: usize,
    pub independent_source_count: usize,
    pub official_source_present: bool,
    #[serde(default)]
    pub duplicate_content_hashes: Vec<String>,
    #[serde(default)]
    pub syndicated_from: Option<String>,
    #[serde(default)]
    pub original_source_ids: Vec<String>,
}
