use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceBand {
    Weak,
    Low,
    Moderate,
    Medium,
    Strong,
    High,
    #[default]
    Unknown,
}

impl ConfidenceBand {
    pub fn as_policy_key(&self) -> &'static str {
        match self {
            Self::Strong | Self::High => "strong",
            Self::Moderate | Self::Medium => "moderate",
            Self::Weak | Self::Low | Self::Unknown => "weak",
        }
    }

    pub fn is_research_allowed(&self) -> bool {
        matches!(
            self,
            Self::Moderate | Self::Medium | Self::Strong | Self::High
        )
    }

    pub fn is_strong(&self) -> bool {
        matches!(self, Self::Strong | Self::High)
    }
}
