use crate::model::{ScoreBreakdown, StructuredIntelPacket, SymbolUniverseSnapshot};
use crate::policy::ScoringPolicy;
use crate::scoring::admission::AdmissionState;

mod contradiction;
mod evidence_quality;
mod helpers;
mod market;
mod novelty;
mod numeric;
mod penalty;
mod quality;
mod source;

use contradiction::push_contradiction_score_components;
use evidence_quality::push_evidence_quality_score_components;
use market::push_market_score_components;
use novelty::push_novelty_score_component;
use penalty::push_penalty_score_components;
use quality::push_quality_score_components;
use source::push_source_score_components;

pub(in crate::scoring) fn calculate_score(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    universe: Option<&SymbolUniverseSnapshot>,
    admission: &AdmissionState,
) -> ScoreBreakdown {
    let mut components = Vec::new();
    push_source_score_components(&mut components, packet, policy);
    push_market_score_components(&mut components, packet, policy, universe, admission);
    push_quality_score_components(&mut components, packet, policy, admission);
    push_novelty_score_component(&mut components, packet, policy);
    push_contradiction_score_components(&mut components, packet, policy, admission);
    push_penalty_score_components(&mut components, policy, admission);
    push_evidence_quality_score_components(&mut components, packet, policy, admission);
    let final_score = components.iter().map(|component| component.value).sum();
    ScoreBreakdown {
        components,
        final_score,
    }
}
