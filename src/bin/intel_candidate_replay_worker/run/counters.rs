use intel_candidate_app::model::CandidateProcessingResult;

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct ReplayCounters {
    pub(super) keys_processed: usize,
    pub(super) result_records_created: usize,
    pub(super) evidence_bundles_created: usize,
    pub(super) hypothesis_states_created: usize,
    pub(super) screening_events_created: usize,
}

impl ReplayCounters {
    pub(super) fn record_success(&mut self, result: &CandidateProcessingResult) {
        self.keys_processed += 1;
        self.result_records_created += 1;
        self.screening_events_created += 1;
        if result.evidence_bundle.is_some() {
            self.evidence_bundles_created += 1;
        }
        if result.hypothesis_state.is_some() {
            self.hypothesis_states_created += 1;
        }
    }
}
