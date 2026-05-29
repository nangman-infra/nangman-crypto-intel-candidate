use super::ObjectStoreConfig;
use crate::error::{AppError, AppResult};

pub(super) fn validate_config(config: &ObjectStoreConfig) -> AppResult<()> {
    if config.bucket.trim().is_empty() {
        return Err(AppError::config("object store bucket is required"));
    }
    if config.bucket.contains('<') || config.bucket.contains('>') {
        return Err(AppError::config(
            "object store bucket must be a real bucket name, not a public-doc placeholder",
        ));
    }
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
