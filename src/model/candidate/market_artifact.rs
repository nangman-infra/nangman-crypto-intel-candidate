use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DataQualitySummaryRef {
    pub market_data_quality_summary_key: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SelectedMarketArtifactTrace {
    pub artifact_type: String,
    pub artifact_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub l1_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol_canonical: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub known_as_of_ms: i64,
    pub quality_status: String,
}
