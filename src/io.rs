use crate::model::{
    IntelCandidateEvidenceBundle, IntelCandidateHypothesisState, IntelCandidateScreeningEvent,
};

mod read;
#[cfg(test)]
mod tests;
mod validation;
mod write;

pub use read::{
    read_market_feature_deltas, read_market_regime_contexts, read_structured_packets,
    read_universe_snapshot,
};
pub use write::write_processing_results;

#[allow(dead_code)]
fn _assert_record_types(
    _: &IntelCandidateScreeningEvent,
    _: &IntelCandidateEvidenceBundle,
    _: &IntelCandidateHypothesisState,
) {
}
