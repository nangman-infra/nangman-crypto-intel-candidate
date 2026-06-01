use super::*;
pub(super) use crate::model::{
    CandidateClass, ConfidenceBand, ContradictionFlag, EventType, EvidenceQualityReason,
    MetricEvidence,
};

#[path = "research_admission/derivatives.rs"]
mod derivatives;
#[path = "research_admission/hypothesis_state.rs"]
mod hypothesis_state;
#[path = "research_admission/market_artifacts.rs"]
mod market_artifacts;
#[path = "research_admission/temporal_cutoff.rs"]
mod temporal_cutoff;
#[path = "research_admission/universe_gate.rs"]
mod universe_gate;
