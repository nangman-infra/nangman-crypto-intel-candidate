use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateClass {
    StrongCandidate,
    ResearchCandidate,
    WeakCandidate,
    ObserveOnly,
    Reject,
    Quarantine,
}

impl CandidateClass {
    pub fn is_research_eligible(&self) -> bool {
        matches!(self, Self::StrongCandidate | Self::ResearchCandidate)
    }

    pub fn as_policy_key(&self) -> &'static str {
        match self {
            Self::StrongCandidate => "strong_candidate",
            Self::ResearchCandidate => "research_candidate",
            Self::WeakCandidate => "weak_candidate",
            Self::ObserveOnly => "observe_only",
            Self::Reject => "reject",
            Self::Quarantine => "quarantine",
        }
    }
}
