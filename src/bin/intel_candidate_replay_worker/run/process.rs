use intel_candidate_app::error::AppResult;
use intel_candidate_app::model::CandidateProcessingResult;
use intel_candidate_app::storage::ObjectStore;
use intel_candidate_app::worker::CandidateWorker;

use super::super::args::ReplayArgs;
use super::super::keys::{checksum_json, replay_result_key, sha256_hex};
use super::super::model::{RESULT_SCHEMA_VERSION, ReplayResultRecord};

pub(super) async fn process_replay_key(
    args: &ReplayArgs,
    worker: &CandidateWorker,
    output_store: &ObjectStore,
    created_at_ms: i64,
    replay_run_id: &str,
    key: &str,
) -> AppResult<(CandidateProcessingResult, String)> {
    let result = worker.score_s3_key(key, created_at_ms).await?;
    if args.write_artifacts {
        worker.write_replay_artifacts(&result).await?;
    }
    let result_key = replay_result_key(&args.result_prefix, created_at_ms, replay_run_id, key);
    let mut result_record = ReplayResultRecord {
        schema_version: RESULT_SCHEMA_VERSION.to_owned(),
        replay_run_id: replay_run_id.to_owned(),
        created_at_ms,
        input_bucket: args.worker.input_store.bucket.clone(),
        input_key: key.to_owned(),
        input_key_sha256: sha256_hex(key.as_bytes()),
        result: result.clone(),
        checksum: String::new(),
    };
    result_record.checksum = checksum_json(&result_record)?;
    output_store
        .put_bytes_idempotent(
            &result_key,
            serde_json::to_vec_pretty(&result_record)?,
            "application/json",
        )
        .await?;
    Ok((result, result_key))
}
