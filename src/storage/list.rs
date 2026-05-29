use super::*;

impl ObjectStore {
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
