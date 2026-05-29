use super::confidence::ConfidenceBand;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SymbolResolutionTrace {
    #[serde(default)]
    pub raw_mentions: Vec<String>,
    #[serde(default)]
    pub resolved_project: Option<String>,
    #[serde(default)]
    pub resolved_asset: Option<String>,
    #[serde(default)]
    pub canonical_symbol: Option<String>,
    #[serde(default)]
    pub venue_symbols: Vec<String>,
    #[serde(default)]
    pub mapping_confidence: ConfidenceBand,
    #[serde(default)]
    pub ambiguity_reason: Option<String>,
}
