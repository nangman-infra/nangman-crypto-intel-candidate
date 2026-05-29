use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Listing,
    ExchangeListing,
    Delisting,
    ExchangeDelisting,
    DepositWithdrawal,
    Incident,
    Partnership,
    ProjectNotice,
    TokenUnlock,
    Governance,
    FundingShift,
    MacroEvent,
    Regulatory,
    SocialBacklash,
    SocialHype,
    #[default]
    Other,
}

impl EventType {
    pub fn as_policy_key(&self) -> &'static str {
        match self {
            Self::Listing => "listing",
            Self::ExchangeListing => "exchange_listing",
            Self::Delisting => "delisting",
            Self::ExchangeDelisting => "exchange_delisting",
            Self::DepositWithdrawal => "deposit_withdrawal",
            Self::Incident => "incident",
            Self::Partnership => "partnership",
            Self::ProjectNotice => "project_notice",
            Self::TokenUnlock => "token_unlock",
            Self::Governance => "governance",
            Self::FundingShift => "funding_shift",
            Self::MacroEvent => "macro_event",
            Self::Regulatory => "regulatory",
            Self::SocialBacklash => "social_backlash",
            Self::SocialHype => "social_hype",
            Self::Other => "other",
        }
    }

    pub fn is_derivatives_like(&self) -> bool {
        matches!(self, Self::FundingShift)
    }
}
