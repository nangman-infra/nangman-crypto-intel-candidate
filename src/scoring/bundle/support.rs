use crate::model::{StructuredIntelPacket, ValidationRequirements};
use crate::policy::ValidationRequirementDefaults;
use crate::scoring::admission::dedupe_strings;

pub(super) fn evidence_refs(packet: &StructuredIntelPacket) -> Vec<String> {
    let mut refs = Vec::new();
    refs.extend(packet.text_evidence.iter().map(|evidence| {
        format!(
            "text_evidence:{}:{}",
            evidence.source_event_id, evidence.source_id
        )
    }));
    refs.extend(packet.metric_evidence.iter().map(|evidence| {
        format!(
            "metric_evidence:{}:{}",
            evidence.source_event_id, evidence.metric_name
        )
    }));
    dedupe_strings(refs)
}

pub(super) fn validation_requirements(
    defaults: &ValidationRequirementDefaults,
) -> ValidationRequirements {
    ValidationRequirements {
        required_adapters: defaults.required_adapters.clone(),
        optional_adapters: defaults.optional_adapters.clone(),
        min_unseen_windows: defaults.min_unseen_windows,
        include_fee: defaults.include_fee,
        include_slippage: defaults.include_slippage,
        include_latency_assumption: defaults.include_latency_assumption,
        include_liquidity_filter: defaults.include_liquidity_filter,
        required_train_validation_split: defaults.required_train_validation_split,
        max_adapter_runtime_minutes: defaults.max_adapter_runtime_minutes,
    }
}
