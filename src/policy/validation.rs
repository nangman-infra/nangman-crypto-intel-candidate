use super::model::ScoringPolicy;
use crate::error::{AppError, AppResult};

pub(super) const RESEARCH_COMPATIBLE_HORIZONS: &[&str] = &["15m", "1h", "4h", "24h", "72h"];

pub(super) fn validate_policy(policy: &ScoringPolicy) -> AppResult<()> {
    for (event_type, horizons) in &policy.event_type_to_allowed_horizons {
        if horizons.is_empty() {
            return Err(AppError::config(format!(
                "event_type_to_allowed_horizons.{event_type} must not be empty"
            )));
        }
        for horizon in horizons {
            if !RESEARCH_COMPATIBLE_HORIZONS.contains(&horizon.as_str()) {
                return Err(AppError::config(format!(
                    "event_type_to_allowed_horizons.{event_type} contains unsupported downstream research horizon {horizon}; allowed horizons are {}",
                    RESEARCH_COMPATIBLE_HORIZONS.join(",")
                )));
            }
        }
    }
    Ok(())
}
