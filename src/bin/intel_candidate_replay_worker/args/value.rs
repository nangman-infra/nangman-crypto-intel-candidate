use intel_candidate_app::error::{AppError, AppResult};

pub(super) fn next_value(values: &[String], index: usize, flag: &str) -> AppResult<String> {
    values
        .get(index)
        .cloned()
        .ok_or_else(|| AppError::config(format!("{flag} requires a value")))
}

pub(super) fn parse_positive_usize(value: &str) -> AppResult<usize> {
    let parsed = value
        .parse::<usize>()
        .map_err(|error| AppError::config(format!("invalid positive integer {value}: {error}")))?;
    if parsed == 0 {
        return Err(AppError::config("value must be positive"));
    }
    Ok(parsed)
}

pub(super) fn parse_non_negative_i64(value: &str) -> AppResult<i64> {
    let parsed = value
        .parse::<i64>()
        .map_err(|error| AppError::config(format!("invalid timestamp {value}: {error}")))?;
    if parsed < 0 {
        return Err(AppError::config("timestamp must be non-negative"));
    }
    Ok(parsed)
}
