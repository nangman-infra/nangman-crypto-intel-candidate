use super::helpers::push_component;
use crate::model::ScoreComponent;
use crate::policy::ScoringPolicy;
use crate::scoring::admission::AdmissionState;

pub(super) fn push_penalty_score_components(
    components: &mut Vec<ScoreComponent>,
    policy: &ScoringPolicy,
    admission: &AdmissionState,
) {
    if admission.social_only {
        push_component(
            components,
            "social_only_penalty",
            policy.weight("social_only_penalty"),
            "social-only source quality",
        );
    }
    if admission.stale_market_context {
        push_component(
            components,
            "stale_event_penalty",
            policy.weight("stale_event_penalty"),
            "stale market context",
        );
    }
}
