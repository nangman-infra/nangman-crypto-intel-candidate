use super::helpers::push_component;
use crate::model::{ScoreComponent, StructuredIntelPacket, SymbolUniverseSnapshot};
use crate::policy::ScoringPolicy;
use crate::scoring::admission::{AdmissionState, effective_market_context_status};

pub(super) fn push_market_score_components(
    components: &mut Vec<ScoreComponent>,
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    universe: Option<&SymbolUniverseSnapshot>,
    admission: &AdmissionState,
) {
    let market_status = effective_market_context_status(packet);
    push_component(
        components,
        "market_context",
        policy.weight(&format!("market_context_{}", market_status.as_policy_key())),
        "market context status",
    );
    if universe.is_some() && admission.approved_universe_symbol {
        push_component(
            components,
            "approved_universe_symbol",
            policy.weight("approved_universe_symbol"),
            "point-in-time universe approved every symbol",
        );
    }
}
