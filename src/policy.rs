use crate::error::AppResult;
use std::path::Path;

mod loading;
mod model;
mod validation;

pub const DEFAULT_POLICY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/policies/scoring-policy.v1.json"
);

pub fn load_policy(path: &Path) -> AppResult<ScoringPolicy> {
    loading::load_policy(path)
}

#[cfg(test)]
mod tests;

pub use model::{
    AdmissionRequirements, HardGates, MarketContextPendingPolicy, MarketContextStatusPolicy,
    ScoringPolicy, Thresholds, ValidationRequirementDefaults,
};
