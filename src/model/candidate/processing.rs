use serde::{Deserialize, Serialize};

use super::bundle::IntelCandidateEvidenceBundle;
use super::screening::IntelCandidateScreeningEvent;
use super::state::IntelCandidateHypothesisState;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CandidateProcessingResult {
    pub screening_event: IntelCandidateScreeningEvent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_bundle: Option<IntelCandidateEvidenceBundle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hypothesis_state: Option<IntelCandidateHypothesisState>,
}
