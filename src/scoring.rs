use crate::hash::{sha256_hex, stable_id};
use crate::model::{
    CANDIDATE_BUNDLE_SCHEMA_VERSION, CandidateClass, CandidateProcessingResult, ConfidenceBand,
    ContradictionFlag, DataQualitySummaryRef, EventType, EvidenceQualityReason,
    HYPOTHESIS_STATE_SCHEMA_VERSION, IntelCandidateEvidenceBundle, IntelCandidateHypothesisState,
    IntelCandidateScreeningEvent, MarketContextRef, MarketContextStatus, MarketFeatureDelta,
    MarketRegimeContext, PRODUCER_APP, SCREENING_EVENT_SCHEMA_VERSION,
    STRUCTURED_PACKET_SCHEMA_VERSION, ScoreBreakdown, ScoreComponent, SelectedMarketArtifactTrace,
    SourceIndependenceSummary, StructuredIntelPacket, SymbolUniverseSnapshot,
    ValidationRequirements,
};
use crate::policy::{ScoringPolicy, ValidationRequirementDefaults};
use crate::time::{hour_bucket_ms, path_segment, time_part};
use std::collections::BTreeSet;

mod admission;
mod bundle;
mod helpers;
mod hypothesis;
mod pipeline;
mod score;
#[cfg(test)]
mod tests;

use admission::*;
use bundle::*;
use helpers::*;
use hypothesis::*;
use score::*;

pub use helpers::{
    candidate_bundle_key, effective_packet_family_id, hypothesis_state_key, screening_event_key,
};
pub use pipeline::{MarketArtifactInputs, process_packet, process_packet_with_artifacts};
