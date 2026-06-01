use super::{ObjectStore, validate_object_key};
use crate::error::{AppError, AppResult};
use aws_sdk_s3::primitives::ByteStream;

impl ObjectStore {
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
        validate_object_key(key, "S3 object key")?;
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

fn is_precondition_failure(message: &str) -> bool {
    message.contains("PreconditionFailed")
        || message.contains("precondition")
        || message.contains("412")
}
