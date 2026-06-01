use super::ObjectStoreConfig;
use super::validate_bucket_name;
use crate::error::{AppError, AppResult};

pub(super) fn validate_config(config: &ObjectStoreConfig) -> AppResult<()> {
    validate_bucket_name(&config.bucket, "object store bucket")?;
    if config.region.trim().is_empty() {
        return Err(AppError::config("object store region is required"));
    }
    if config.access_key_id.is_some() != config.secret_access_key.is_some() {
        return Err(AppError::config(
            "object store explicit credentials require both access_key_id and secret_access_key",
        ));
    }
    Ok(())
}
