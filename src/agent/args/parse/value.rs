use crate::error::{AppError, AppResult};

pub(super) fn next_value(values: &[String], index: usize, flag: &str) -> AppResult<String> {
    values
        .get(index)
        .cloned()
        .ok_or_else(|| AppError::config(format!("{flag} requires a value")))
}

pub(super) fn parse_positive_u64(value: &str) -> AppResult<u64> {
    let parsed = value
        .parse::<u64>()
        .map_err(|error| AppError::config(format!("invalid positive integer {value}: {error}")))?;
    if parsed == 0 {
        return Err(AppError::config("value must be positive"));
    }
    Ok(parsed)
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

pub(super) fn parse_non_negative_u32(value: &str) -> AppResult<u32> {
    value
        .parse::<u32>()
        .map_err(|error| AppError::config(format!("invalid non-negative integer {value}: {error}")))
}
