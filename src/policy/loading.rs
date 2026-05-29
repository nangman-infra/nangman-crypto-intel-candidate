use super::model::ScoringPolicy;
use super::validation::validate_policy;
use crate::error::AppResult;
use std::path::Path;

pub(super) fn load_policy(path: &Path) -> AppResult<ScoringPolicy> {
    let bytes = std::fs::read(path)?;
    let policy = serde_json::from_slice(&bytes)?;
    validate_policy(&policy)?;
    Ok(policy)
}
