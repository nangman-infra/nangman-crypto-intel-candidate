use crate::error::{AppError, AppResult};
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_s3::operation::list_objects_v2::builders::ListObjectsV2FluentBuilder;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::Object;
use aws_types::region::Region;
use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectStoreConfig {
    pub endpoint: Option<String>,
    pub bucket: String,
    pub region: String,
    pub force_path_style: bool,
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
        let endpoint = config.endpoint.or_else(env_s3_endpoint);
        if let Some(endpoint) = endpoint {
            loader = loader.endpoint_url(endpoint.trim_end_matches('/'));
        }
        let sdk_config = loader.load().await;
        let force_path_style = config.force_path_style
            || env_bool("AWS_S3_FORCE_PATH_STYLE")
            || env_bool("AWS_USE_PATH_STYLE_ENDPOINT");
        let mut s3_builder = S3ConfigBuilder::from(&sdk_config).force_path_style(force_path_style);
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

    pub async fn list_keys(&self, prefix: &str, max_keys: usize) -> AppResult<Vec<String>> {
        self.list_keys_page(prefix, max_keys, None)
            .await
            .map(|page| page.keys)
    }

    pub async fn list_keys_page(
        &self,
        prefix: &str,
        max_keys: usize,
        start_after: Option<&str>,
    ) -> AppResult<ListKeysPage> {
        if max_keys == 0 {
            return Ok(ListKeysPage {
                keys: Vec::new(),
                next_start_after: None,
            });
        }
        let mut keys = Vec::new();
        let mut continuation_token = None;
        while keys.len() < max_keys {
            let output = self
                .list_objects_request(
                    prefix,
                    list_keys_remaining(keys.len(), max_keys),
                    continuation_token.as_deref(),
                    start_after,
                )
                .send()
                .await
                .map_err(|error| {
                    AppError::aws(format!(
                        "list_objects_v2 bucket={} prefix={} error={error}",
                        self.bucket, prefix
                    ))
                })?;
            append_object_keys(&mut keys, max_keys, output.contents());
            continuation_token = output.next_continuation_token().map(ToOwned::to_owned);
            if continuation_token.is_none() {
                break;
            }
        }
        Ok(ListKeysPage {
            next_start_after: next_list_start_after(&keys, continuation_token.as_deref()),
            keys,
        })
    }

    fn list_objects_request(
        &self,
        prefix: &str,
        remaining: i32,
        continuation_token: Option<&str>,
        start_after: Option<&str>,
    ) -> ListObjectsV2FluentBuilder {
        let request = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(prefix)
            .max_keys(remaining);
        apply_list_cursor(request, continuation_token, start_after)
    }

    pub async fn put_jsonl_record_idempotent<T: serde::Serialize>(
        &self,
        key: &str,
        record: &T,
    ) -> AppResult<Vec<u8>> {
        let mut bytes = serde_json::to_vec(record)?;
        bytes.push(b'\n');
        self.put_bytes_idempotent(key, bytes.clone(), "application/x-ndjson")
            .await?;
        Ok(bytes)
    }

    pub async fn put_jsonl_record_or_existing<T: serde::Serialize>(
        &self,
        key: &str,
        record: &T,
    ) -> AppResult<Vec<u8>> {
        match self.put_jsonl_record_idempotent(key, record).await {
            Ok(bytes) => Ok(bytes),
            Err(AppError::Validation(message)) if message.contains("idempotency conflict") => {
                self.get_bytes(key).await
            }
            Err(error) => Err(error),
        }
    }

    pub async fn put_bytes_idempotent(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &'static str,
    ) -> AppResult<()> {
        match self
            .put_bytes_guarded(key, bytes.clone(), content_type, true)
            .await
        {
            Ok(()) => Ok(()),
            Err(AppError::Validation(message)) if message.contains("object already exists") => {
                self.accept_existing_bytes_or_error(key, &bytes, AppError::Validation(message))
                    .await
            }
            Err(AppError::Aws(message)) => {
                self.accept_existing_bytes_or_error(key, &bytes, AppError::Aws(message))
                    .await
            }
            Err(error) => Err(error),
        }
    }

    async fn accept_existing_bytes_or_error(
        &self,
        key: &str,
        bytes: &[u8],
        original_error: AppError,
    ) -> AppResult<()> {
        match self.get_bytes(key).await {
            Ok(existing) if existing == bytes => Ok(()),
            Ok(_) => Err(AppError::validation(format!(
                "idempotency conflict bucket={} key={key}",
                self.bucket
            ))),
            Err(_) => Err(original_error),
        }
    }

    async fn put_bytes_guarded(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &'static str,
        if_absent: bool,
    ) -> AppResult<()> {
        let mut request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(ByteStream::from(bytes));
        if if_absent {
            request = request.if_none_match("*");
        }
        request.send().await.map_err(|error| {
            let message = error.to_string();
            if if_absent && is_precondition_failure(&message) {
                AppError::validation(format!(
                    "object already exists bucket={} key={key}",
                    self.bucket
                ))
            } else {
                AppError::aws(format!(
                    "put_object bucket={} key={} error={message}",
                    self.bucket, key
                ))
            }
        })?;
        Ok(())
    }
}

fn list_keys_remaining(current_len: usize, max_keys: usize) -> i32 {
    max_keys.saturating_sub(current_len).min(i32::MAX as usize) as i32
}

fn apply_list_cursor(
    request: ListObjectsV2FluentBuilder,
    continuation_token: Option<&str>,
    start_after: Option<&str>,
) -> ListObjectsV2FluentBuilder {
    match continuation_token {
        Some(token) => request.continuation_token(token),
        None => apply_start_after(request, start_after),
    }
}

fn apply_start_after(
    request: ListObjectsV2FluentBuilder,
    start_after: Option<&str>,
) -> ListObjectsV2FluentBuilder {
    match start_after.filter(|value| !value.trim().is_empty()) {
        Some(value) => request.start_after(value),
        None => request,
    }
}

fn append_object_keys(keys: &mut Vec<String>, max_keys: usize, objects: &[Object]) {
    for object in objects {
        let Some(key) = object.key() else {
            continue;
        };
        keys.push(key.to_owned());
        if keys.len() >= max_keys {
            break;
        }
    }
}

fn next_list_start_after(keys: &[String], continuation_token: Option<&str>) -> Option<String> {
    continuation_token.and_then(|_| keys.last().cloned())
}

fn env_s3_endpoint() -> Option<String> {
    env::var("AWS_ENDPOINT_URL_S3")
        .ok()
        .or_else(|| env::var("AWS_ENDPOINT_URL").ok())
        .map(|value| value.trim().trim_end_matches('/').to_owned())
        .filter(|value| !value.is_empty())
}

fn env_bool(name: &str) -> bool {
    env::var(name)
        .ok()
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn validate_config(config: &ObjectStoreConfig) -> AppResult<()> {
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
    if let Some(endpoint) = &config.endpoint
        && !endpoint.starts_with("http://")
        && !endpoint.starts_with("https://")
    {
        return Err(AppError::config("object store endpoint must be http(s)"));
    }
    if config.access_key_id.is_some() != config.secret_access_key.is_some() {
        return Err(AppError::config(
            "object store explicit credentials require both access_key_id and secret_access_key",
        ));
    }
    Ok(())
}

fn is_precondition_failure(message: &str) -> bool {
    message.contains("PreconditionFailed")
        || message.contains("precondition")
        || message.contains("412")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_endpoint() {
        let config = ObjectStoreConfig {
            endpoint: Some("ftp://example.com".to_owned()),
            bucket: "bucket".to_owned(),
            region: "ap-northeast-2".to_owned(),
            force_path_style: false,
            profile: None,
            access_key_id: None,
            secret_access_key: None,
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn rejects_public_doc_bucket_placeholder() {
        let config = ObjectStoreConfig {
            endpoint: None,
            bucket: "nangman-crypto-dev-intel-candidate-<account-suffix>".to_owned(),
            region: "ap-northeast-2".to_owned(),
            force_path_style: false,
            profile: None,
            access_key_id: None,
            secret_access_key: None,
        };
        let err = validate_config(&config).unwrap_err();
        assert!(err.to_string().contains("public-doc placeholder"));
    }
}
