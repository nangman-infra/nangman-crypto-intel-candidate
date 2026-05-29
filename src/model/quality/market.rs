use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MarketContextStatus {
    Available,
    AvailableSymbolContext,
    AvailableGeneralContext,
    NearestAvailable,
    SymbolContextOnly,
    StaleButUsable,
    Pending,
    Unavailable,
    #[default]
    Unknown,
}

impl MarketContextStatus {
    pub fn as_policy_key(&self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::AvailableSymbolContext => "available_symbol_context",
            Self::AvailableGeneralContext => "available_general_context",
            Self::NearestAvailable => "nearest_available",
            Self::SymbolContextOnly => "symbol_context_only",
            Self::StaleButUsable => "stale_but_usable",
            Self::Pending => "pending",
            Self::Unavailable => "unavailable",
            Self::Unknown => "unknown",
        }
    }
}
