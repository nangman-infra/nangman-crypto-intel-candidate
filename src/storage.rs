use crate::error::{AppError, AppResult};
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_s3::operation::list_objects_v2::builders::ListObjectsV2FluentBuilder;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::Object;
use aws_types::region::Region;

mod config;
mod list;
#[cfg(test)]
mod tests;
mod write;

use config::validate_config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectStoreConfig {
    pub bucket: String,
    pub region: String,
    pub profile: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
}

#[derive(Clone)]
pub struct ObjectStore {
    client: Client,
    bucket: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListKeysPage {
    pub keys: Vec<String>,
    pub next_start_after: Option<String>,
}

impl ObjectStore {
    pub async fn connect(config: ObjectStoreConfig) -> AppResult<Self> {
        validate_config(&config)?;
        let mut loader =
            aws_config::defaults(BehaviorVersion::latest()).region(Region::new(config.region));
        if let Some(profile) = config.profile {
            loader = loader.profile_name(profile);
        }
        let sdk_config = loader.load().await;
        let mut s3_builder = S3ConfigBuilder::from(&sdk_config);
        if let (Some(access_key_id), Some(secret_access_key)) =
            (config.access_key_id, config.secret_access_key)
        {
            s3_builder = s3_builder.credentials_provider(Credentials::new(
                access_key_id,
                secret_access_key,
                None,
                None,
                "intel-candidate-app-explicit-object-store",
            ));
        }
        let store = Self {
            client: Client::from_conf(s3_builder.build()),
            bucket: config.bucket,
        };
        store.head_bucket().await?;
        Ok(store)
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub async fn head_bucket(&self) -> AppResult<()> {
        self.client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .map_err(|error| AppError::aws(format!("head_bucket {}: {error}", self.bucket)))?;
        Ok(())
    }

    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> AppResult<T> {
        let bytes = self.get_bytes(key).await?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub async fn get_bytes(&self, key: &str) -> AppResult<Vec<u8>> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|error| {
                AppError::aws(format!(
                    "get_object bucket={} key={} error={error}",
                    self.bucket, key
                ))
            })?;
        Ok(output
            .body
            .collect()
            .await
            .map_err(|error| AppError::aws(format!("collect body key={key}: {error}")))?
            .into_bytes()
            .to_vec())
    }
}
