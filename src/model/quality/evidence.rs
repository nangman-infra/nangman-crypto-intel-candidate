use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TextEvidence {
    pub evidence_text: String,
    pub source_event_id: String,
    pub source_id: String,
    #[serde(default)]
    pub published_at_ms: Option<i64>,
    #[serde(default)]
    pub evidence_kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MetricEvidence {
    pub metric_name: String,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub value: Option<f64>,
    #[serde(default)]
    pub previous_value: Option<f64>,
    #[serde(default)]
    pub delta_pct: Option<f64>,
    #[serde(default)]
    pub window_ms: Option<i64>,
    pub observed_at_ms: i64,
    pub source_event_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TimeRelevanceWindow {
    pub start_ms: i64,
    pub end_ms: i64,
    pub relevance_decay_hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionFlag {
    TimeMismatch,
    SymbolAmbiguity,
    SourceClaimConflict,
    RumorVsOfficial,
    TitleBodyMismatch,
    EvidenceWeak,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceQualityReason {
    BaselineMissing,
    SingleNumericSnapshot,
    SingleSourceOnly,
    TitleOnly,
    SymbolAmbiguous,
    MarketContextMissing,
    DuplicateOrSyndicatedSource,
    ApprovedUniverseMissing,
}

impl EvidenceQualityReason {
    pub fn as_policy_key(&self) -> &'static str {
        match self {
            Self::BaselineMissing => "baseline_missing",
            Self::SingleNumericSnapshot => "single_numeric_snapshot",
            Self::SingleSourceOnly => "single_source_only",
            Self::TitleOnly => "title_only",
            Self::SymbolAmbiguous => "symbol_ambiguous",
            Self::MarketContextMissing => "market_context_missing",
            Self::DuplicateOrSyndicatedSource => "duplicate_or_syndicated_source",
            Self::ApprovedUniverseMissing => "approved_universe_missing",
        }
    }
}
