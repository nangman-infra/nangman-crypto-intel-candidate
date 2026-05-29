use super::super::*;

pub(in crate::scoring) struct BundleBuildContext<'a> {
    pub(in crate::scoring) candidate_id: &'a str,
    pub(in crate::scoring) packet: &'a StructuredIntelPacket,
    pub(in crate::scoring) policy: &'a ScoringPolicy,
    pub(in crate::scoring) research_inputs: ResearchBundleInputs<'a>,
    pub(in crate::scoring) created_at_ms: i64,
    pub(in crate::scoring) candidate_class: CandidateClass,
    pub(in crate::scoring) score_breakdown: ScoreBreakdown,
    pub(in crate::scoring) reasons: Vec<String>,
    pub(in crate::scoring) selected_market_artifacts: Vec<SelectedMarketArtifactTrace>,
    pub(in crate::scoring) idempotency_key: String,
}

pub(in crate::scoring) struct ResearchBundleInputs<'a> {
    pub(super) universe: &'a SymbolUniverseSnapshot,
    pub(super) decision_available_at_ms: i64,
    pub(super) fetched_at_ms: i64,
    pub(super) structured_at_ms: i64,
    pub(super) source_independence: SourceIndependenceSummary,
}
