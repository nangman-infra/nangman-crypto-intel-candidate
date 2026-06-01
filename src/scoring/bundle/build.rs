use super::context::BundleBuildContext;
use super::identity::bundle_identity;
use super::summaries::{confidence_summary, data_quality_summary};
use super::support::{evidence_refs, validation_requirements};
use crate::hash::{sha256_hex, stable_id};
use crate::model::{CANDIDATE_BUNDLE_SCHEMA_VERSION, IntelCandidateEvidenceBundle, PRODUCER_APP};
use crate::scoring::helpers::{
    approved_universe_symbols, effective_packet_family_id, parent_artifact_ids,
};

pub(in crate::scoring) fn build_evidence_bundle(
    context: BundleBuildContext<'_>,
) -> Result<IntelCandidateEvidenceBundle, serde_json::Error> {
    let candidate_id = context.candidate_id;
    let packet = context.packet;
    let policy = context.policy;
    let research_inputs = context.research_inputs;
    let universe = research_inputs.universe;
    let created_at_ms = context.created_at_ms;
    let packet_decision_available_at_ms = research_inputs.decision_available_at_ms;
    let event_time_ms = packet_decision_available_at_ms;
    let decision_available_at_ms = packet_decision_available_at_ms.max(created_at_ms);
    let fetched_at_ms = research_inputs.fetched_at_ms;
    let structured_at_ms = research_inputs.structured_at_ms;
    let candidate_score = context.score_breakdown.final_score;
    let identity = bundle_identity(packet, policy, candidate_id, created_at_ms);
    let source_independence = research_inputs.source_independence;
    let parent_artifact_ids = parent_artifact_ids(packet);
    let evidence_refs = evidence_refs(packet);

    let mut bundle = IntelCandidateEvidenceBundle {
        candidate_id: candidate_id.to_owned(),
        candidate_lifecycle_key: identity.candidate_lifecycle_key,
        bundle_key: identity.bundle_key.clone(),
        producer_app: PRODUCER_APP.to_owned(),
        producer_run_id: stable_id("cand_run", &[candidate_id, &created_at_ms.to_string()]),
        created_at_ms,
        event_time_ms,
        published_at_ms: packet.published_at_ms,
        fetched_at_ms,
        structured_at_ms,
        candidate_created_at_ms: created_at_ms,
        decision_available_at_ms,
        forbidden_lookahead_boundary_ms: decision_available_at_ms,
        schema_version: CANDIDATE_BUNDLE_SCHEMA_VERSION.to_owned(),
        scoring_policy_version: policy.policy_version.clone(),
        normalized_symbols: packet.normalized_symbols.clone(),
        input_packet_family_id: effective_packet_family_id(packet).to_owned(),
        input_packet_revision: packet.revision,
        supersedes_packet_id: packet.supersedes_packet_id.clone(),
        symbol_universe_snapshot_id: universe.symbol_universe_snapshot_id.clone(),
        universe_as_of_ms: universe.universe_as_of_ms,
        approved_universe_symbol: approved_universe_symbols(packet, universe),
        event_types: vec![packet.event_type.as_policy_key().to_owned()],
        hypothesis_type: identity.hypothesis_type,
        allowed_horizons: identity.allowed_horizons,
        source_story_cluster_ids: vec![packet.cluster_id.clone()],
        source_structured_packet_ids: vec![packet.packet_id.clone()],
        source_context_flag_packet_ids: Vec::new(),
        evidence_refs,
        text_evidence: packet.text_evidence.clone(),
        metric_evidence: packet.metric_evidence.clone(),
        market_context_ref: packet.market_context_ref.clone(),
        data_quality_summary: data_quality_summary(packet),
        selected_market_artifacts: context.selected_market_artifacts,
        candidate_class: context.candidate_class,
        candidate_score,
        score_breakdown: context.score_breakdown,
        research_priority: identity.research_priority,
        research_eligible: true,
        validation_requirements: validation_requirements(&policy.validation_requirement_defaults),
        source_independence,
        symbol_resolution_trace: packet.symbol_resolution_trace.clone(),
        confidence_summary: confidence_summary(packet),
        contradiction_summary: packet.contradiction_flags.clone(),
        observe_or_reject_reasons: context.reasons,
        parent_artifact_ids,
        storage_uri: identity.bundle_key,
        checksum: String::new(),
        idempotency_key: context.idempotency_key,
    };
    let checksum_payload = serde_json::to_vec(&bundle)?;
    bundle.checksum = sha256_hex(checksum_payload);
    Ok(bundle)
}
