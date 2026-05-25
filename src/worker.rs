use crate::error::{AppError, AppResult};
use crate::hash::{sha256_hex, stable_id};
use crate::model::{
    CANDIDATE_POINTER_SCHEMA_VERSION, CANDIDATE_REVISION_INDEX_SCHEMA_VERSION,
    CandidateProcessingResult, CandidateRevisionIndex, MarketFeatureDelta,
    MarketFeatureDeltaSummary, MarketRegimeContext, StructuredIntelPacket, SymbolUniverseSnapshot,
};
use crate::nats::{
    CandidateArtifactPointer, CandidatePublisher, NatsConfig, S3ObjectPointer, StructuredPointer,
};
use crate::policy::{DEFAULT_POLICY_PATH, ScoringPolicy, load_policy};
use crate::scoring::{
    MarketArtifactInputs, effective_packet_family_id, process_packet_with_artifacts,
    screening_event_key,
};
use crate::storage::{ListKeysPage, ObjectStore, ObjectStoreConfig};
use crate::time::path_segment;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const DEFAULT_AWS_REGION: &str = "ap-northeast-2";
pub const DEFAULT_INPUT_BUCKET: &str = "nangman-crypto-dev-intel-structuring-l1-<account-suffix>";
pub const DEFAULT_OUTPUT_BUCKET: &str = "nangman-crypto-dev-intel-candidate-<account-suffix>";
pub const DEFAULT_MARKET_L1_BUCKET: &str = "nangman-crypto-dev-market-ingest-l1-<account-suffix>";
pub const DEFAULT_INPUT_STREAM: &str = "STRUCTURED_INTEL";
pub const DEFAULT_INPUT_SUBJECT: &str = "structured_intel_packet.created";
pub const DEFAULT_INPUT_CONSUMER: &str = "intel-candidate";
pub const DEFAULT_OUTPUT_STREAM: &str = "INTEL_CANDIDATE";
pub const DEFAULT_BUNDLE_SUBJECT: &str = "intel_candidate_evidence_bundle.created";
pub const DEFAULT_SCREENING_SUBJECT: &str = "intel_candidate_screening_event.created";
pub const DEFAULT_HYPOTHESIS_STATE_SUBJECT: &str = "intel_candidate_hypothesis_state.created";
pub const DEFAULT_HEALTH_SUBJECT: &str = "intel_candidate_health_event.created";
const REVISION_INDEX_MAX_KEYS: usize = 256;

fn revision_index_prefix(packet_family_id: &str, scoring_policy_version: &str) -> String {
    format!(
        "candidate-revision-index/schema={}/packet_family_id={}/scoring_policy={}/",
        CANDIDATE_REVISION_INDEX_SCHEMA_VERSION,
        path_segment(packet_family_id),
        path_segment(scoring_policy_version)
    )
}

fn revision_index_key(
    packet_family_id: &str,
    scoring_policy_version: &str,
    revision: u32,
) -> String {
    format!(
        "{}revision={:010}.json",
        revision_index_prefix(packet_family_id, scoring_policy_version),
        revision
    )
}

fn parse_revision_from_key(key: &str) -> Option<u32> {
    key.strip_suffix(".json")?
        .rsplit_once("revision=")?
        .1
        .parse()
        .ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerArgs {
    pub nats: NatsConfig,
    pub input_store: ObjectStoreConfig,
    pub output_store: ObjectStoreConfig,
    pub market_store: ObjectStoreConfig,
    pub policy_file: PathBuf,
    pub max_messages: Option<usize>,
    pub exit_on_idle: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayInputKeyPage {
    pub keys: Vec<String>,
    pub next_start_after: Option<String>,
}

pub struct CandidateWorker {
    input_store: ObjectStore,
    output_store: ObjectStore,
    market_store: ObjectStore,
    policy: ScoringPolicy,
    publisher: CandidatePublisher,
}

impl WorkerArgs {
    pub fn parse(mut values: impl Iterator<Item = String>) -> AppResult<Option<Self>> {
        let mut args = Self::default();
        while let Some(arg) = values.next() {
            match arg.as_str() {
                "-h" | "--help" => return Ok(None),
                "--nats-url" => {
                    args.nats.url = next_string(&mut values, "--nats-url requires a URL")?;
                }
                "--input-stream" => {
                    args.nats.input_stream =
                        next_string(&mut values, "--input-stream requires a stream name")?;
                }
                "--input-subject" => {
                    args.nats.input_subject =
                        next_string(&mut values, "--input-subject requires a subject")?;
                }
                "--input-consumer" => {
                    args.nats.input_consumer =
                        next_string(&mut values, "--input-consumer requires a durable name")?;
                }
                "--output-stream" => {
                    args.nats.output_stream =
                        next_string(&mut values, "--output-stream requires a stream name")?;
                }
                "--bundle-subject" => {
                    args.nats.bundle_subject =
                        next_string(&mut values, "--bundle-subject requires a subject")?;
                }
                "--screening-subject" => {
                    args.nats.screening_subject =
                        next_string(&mut values, "--screening-subject requires a subject")?;
                }
                "--hypothesis-state-subject" => {
                    args.nats.hypothesis_state_subject =
                        next_string(&mut values, "--hypothesis-state-subject requires a subject")?;
                }
                "--input-s3-bucket" => {
                    args.input_store.bucket =
                        next_string(&mut values, "--input-s3-bucket requires a bucket")?;
                }
                "--output-s3-bucket" => {
                    args.output_store.bucket =
                        next_string(&mut values, "--output-s3-bucket requires a bucket")?;
                }
                "--market-l1-s3-bucket" => {
                    args.market_store.bucket =
                        next_string(&mut values, "--market-l1-s3-bucket requires a bucket")?;
                }
                "--aws-region" => {
                    let region = next_string(&mut values, "--aws-region requires a region")?;
                    args.input_store.region = region.clone();
                    args.output_store.region = region.clone();
                    args.market_store.region = region;
                }
                "--input-s3-region" => {
                    args.input_store.region =
                        next_string(&mut values, "--input-s3-region requires a region")?;
                }
                "--output-s3-region" => {
                    args.output_store.region =
                        next_string(&mut values, "--output-s3-region requires a region")?;
                }
                "--market-l1-s3-region" => {
                    args.market_store.region =
                        next_string(&mut values, "--market-l1-s3-region requires a region")?;
                }
                "--aws-profile" => {
                    let profile = Some(next_string(&mut values, "--aws-profile requires a name")?);
                    args.input_store.profile = profile.clone();
                    args.output_store.profile = profile.clone();
                    args.market_store.profile = profile;
                }
                "--policy-file" => {
                    args.policy_file = absolute_path_arg(
                        values.next(),
                        "--policy-file requires an absolute path",
                    )?;
                }
                "--ack-wait-secs" => {
                    args.nats.ack_wait_secs = positive_u64_arg(values.next(), "--ack-wait-secs")?;
                }
                "--max-deliver" => {
                    args.nats.max_deliver = positive_i64_arg(values.next(), "--max-deliver")?;
                }
                "--batch-size" => {
                    args.nats.batch_size = positive_usize_arg(values.next(), "--batch-size")?;
                }
                "--max-messages" => {
                    args.max_messages = Some(positive_usize_arg(values.next(), "--max-messages")?);
                }
                "--exit-on-idle" => {
                    args.exit_on_idle = true;
                }
                "--no-ensure-output-stream" => {
                    args.nats.ensure_output_stream = false;
                }
                other => {
                    return Err(AppError::config(format!(
                        "unknown worker argument: {other}\n\n{}",
                        worker_help()
                    )));
                }
            }
        }
        if args.nats.url.trim().is_empty() {
            args.nats.url = std::env::var("NATS_URL").unwrap_or_default();
        }
        if args.nats.url.trim().is_empty() {
            return Err(AppError::config("--nats-url or NATS_URL is required"));
        }
        validate_bucket_arg(&args.input_store.bucket, "--input-s3-bucket")?;
        validate_bucket_arg(&args.output_store.bucket, "--output-s3-bucket")?;
        validate_bucket_arg(&args.market_store.bucket, "--market-l1-s3-bucket")?;
        Ok(Some(args))
    }
}

impl Default for WorkerArgs {
    fn default() -> Self {
        let input_store = ObjectStoreConfig {
            endpoint: None,
            bucket: DEFAULT_INPUT_BUCKET.to_owned(),
            region: DEFAULT_AWS_REGION.to_owned(),
            force_path_style: false,
            profile: None,
            access_key_id: None,
            secret_access_key: None,
        };
        let output_store = ObjectStoreConfig {
            bucket: DEFAULT_OUTPUT_BUCKET.to_owned(),
            ..input_store.clone()
        };
        let market_store = ObjectStoreConfig {
            bucket: DEFAULT_MARKET_L1_BUCKET.to_owned(),
            ..input_store.clone()
        };
        Self {
            nats: NatsConfig {
                url: String::new(),
                input_stream: DEFAULT_INPUT_STREAM.to_owned(),
                input_subject: DEFAULT_INPUT_SUBJECT.to_owned(),
                input_consumer: DEFAULT_INPUT_CONSUMER.to_owned(),
                input_deliver_policy: "new".to_owned(),
                output_stream: DEFAULT_OUTPUT_STREAM.to_owned(),
                bundle_subject: DEFAULT_BUNDLE_SUBJECT.to_owned(),
                screening_subject: DEFAULT_SCREENING_SUBJECT.to_owned(),
                hypothesis_state_subject: DEFAULT_HYPOTHESIS_STATE_SUBJECT.to_owned(),
                health_subject: DEFAULT_HEALTH_SUBJECT.to_owned(),
                ensure_output_stream: true,
                output_stream_max_age_secs: 336 * 60 * 60,
                output_stream_duplicate_window_secs: 24 * 60 * 60,
                ack_wait_secs: 300,
                max_deliver: 20,
                batch_size: 1,
            },
            input_store,
            output_store,
            market_store,
            policy_file: PathBuf::from(DEFAULT_POLICY_PATH),
            max_messages: None,
            exit_on_idle: false,
        }
    }
}

impl CandidateWorker {
    pub async fn connect(args: &WorkerArgs) -> AppResult<Self> {
        Ok(Self {
            input_store: ObjectStore::connect(args.input_store.clone()).await?,
            output_store: ObjectStore::connect(args.output_store.clone()).await?,
            market_store: ObjectStore::connect(args.market_store.clone()).await?,
            policy: load_policy(&args.policy_file)?,
            publisher: CandidatePublisher::connect(&args.nats).await?,
        })
    }

    pub async fn process_pointer(
        &self,
        pointer: &StructuredPointer,
        created_at_ms: i64,
    ) -> AppResult<Option<CandidateProcessingResult>> {
        pointer.validate()?;
        if pointer.storage_ref.bucket != self.input_store.bucket() {
            return Err(AppError::validation(format!(
                "structured pointer bucket mismatch expected={} actual={}",
                self.input_store.bucket(),
                pointer.storage_ref.bucket
            )));
        }
        let packet_bytes = self.input_store.get_bytes(&pointer.storage_ref.key).await?;
        validate_pointer_content_hash(pointer, &packet_bytes)?;
        let packet: StructuredIntelPacket =
            read_single_json_or_jsonl(&packet_bytes, Path::new(&pointer.storage_ref.key))?;
        if self.is_stale_revision(&packet).await? {
            return Ok(None);
        }
        let result = self.score_packet(packet.clone(), created_at_ms).await?;
        self.write_and_publish_result(&result).await?;
        self.write_revision_index(&packet, &result, created_at_ms)
            .await?;
        Ok(Some(result))
    }

    pub async fn score_s3_key(
        &self,
        key: &str,
        created_at_ms: i64,
    ) -> AppResult<CandidateProcessingResult> {
        let packet_bytes = self.input_store.get_bytes(key).await?;
        let packet: StructuredIntelPacket =
            read_single_json_or_jsonl(&packet_bytes, Path::new(key))?;
        self.score_packet(packet, created_at_ms).await
    }

    async fn score_packet(
        &self,
        packet: StructuredIntelPacket,
        created_at_ms: i64,
    ) -> AppResult<CandidateProcessingResult> {
        let universe = match packet
            .market_context_ref
            .as_ref()
            .and_then(|reference| reference.symbol_universe_snapshot_key.as_deref())
        {
            Some(key) if !key.trim().is_empty() => self
                .market_store
                .get_json::<SymbolUniverseSnapshot>(key)
                .await
                .ok(),
            _ => None,
        };
        let market_feature_deltas = self.read_market_feature_deltas(&packet).await?;
        let market_regime_contexts = match packet
            .market_context_ref
            .as_ref()
            .and_then(|reference| reference.market_regime_context_key.as_deref())
        {
            Some(key) if !key.trim().is_empty() => self
                .market_store
                .get_json::<Vec<MarketRegimeContext>>(key)
                .await
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let result = process_packet_with_artifacts(
            packet.clone(),
            &self.policy,
            MarketArtifactInputs {
                universe: universe.as_ref(),
                market_feature_deltas: &market_feature_deltas,
                market_regime_contexts: &market_regime_contexts,
            },
            created_at_ms,
        );
        Ok(result)
    }

    pub async fn process_s3_key(
        &self,
        key: &str,
        created_at_ms: i64,
    ) -> AppResult<Option<CandidateProcessingResult>> {
        let packet_bytes = self.input_store.get_bytes(key).await?;
        let mut packet: StructuredIntelPacket =
            read_single_json_or_jsonl(&packet_bytes, Path::new(key))?;
        if packet.raw_event_id.trim().is_empty() {
            packet.raw_event_id = repair_raw_event_id(&packet, key);
        }
        if self.is_stale_revision(&packet).await? {
            return Ok(None);
        }
        let result = self.score_packet(packet.clone(), created_at_ms).await?;
        self.write_and_publish_result(&result).await?;
        self.write_revision_index(&packet, &result, created_at_ms)
            .await?;
        Ok(Some(result))
    }

    pub async fn write_replay_artifacts(
        &self,
        result: &CandidateProcessingResult,
    ) -> AppResult<()> {
        let screening_key = screening_event_key(
            result.screening_event.created_at_ms,
            &result.screening_event.screening_event_id,
        );
        self.output_store
            .put_jsonl_record_or_existing(&screening_key, &result.screening_event)
            .await?;
        if let Some(bundle) = &result.evidence_bundle {
            self.output_store
                .put_jsonl_record_or_existing(&bundle.bundle_key, bundle)
                .await?;
        }
        if let Some(state) = &result.hypothesis_state {
            self.output_store
                .put_jsonl_record_or_existing(&state.state_key, state)
                .await?;
        }
        Ok(())
    }

    pub async fn list_replay_input_keys(
        &self,
        prefix: &str,
        max_keys: usize,
    ) -> AppResult<Vec<String>> {
        self.list_replay_input_key_page(prefix, max_keys, None)
            .await
            .map(|page| page.keys)
    }

    pub async fn list_replay_input_key_page(
        &self,
        prefix: &str,
        max_keys: usize,
        start_after: Option<&str>,
    ) -> AppResult<ReplayInputKeyPage> {
        let ListKeysPage {
            mut keys,
            next_start_after,
        } = self
            .input_store
            .list_keys_page(prefix, max_keys, start_after)
            .await?;
        keys.retain(|key| {
            let normalized = key.to_ascii_lowercase();
            normalized.ends_with(".json") || normalized.ends_with(".jsonl")
        });
        keys.sort();
        Ok(ReplayInputKeyPage {
            keys,
            next_start_after,
        })
    }

    async fn write_and_publish_result(&self, result: &CandidateProcessingResult) -> AppResult<()> {
        let screening_key = screening_event_key(
            result.screening_event.created_at_ms,
            &result.screening_event.screening_event_id,
        );
        let screening_bytes = self
            .output_store
            .put_jsonl_record_or_existing(&screening_key, &result.screening_event)
            .await?;
        let screening_pointer = CandidateArtifactPointer {
            schema_version: CANDIDATE_POINTER_SCHEMA_VERSION.to_owned(),
            artifact_family: "intel_candidate_screening_event".to_owned(),
            candidate_id: result.screening_event.candidate_id.clone(),
            screening_event_id: result.screening_event.screening_event_id.clone(),
            candidate_class: result
                .screening_event
                .candidate_class
                .as_policy_key()
                .to_owned(),
            storage_ref: S3ObjectPointer {
                bucket: self.output_store.bucket().to_owned(),
                key: screening_key,
                content_sha256: sha256_prefixed(&screening_bytes),
                schema_version: result.screening_event.schema_version.clone(),
            },
            created_at_ms: result.screening_event.created_at_ms,
        };
        self.publisher
            .publish_screening_pointer(&screening_pointer)
            .await?;

        if let Some(bundle) = &result.evidence_bundle {
            let bundle_bytes = self
                .output_store
                .put_jsonl_record_or_existing(&bundle.bundle_key, bundle)
                .await?;
            let bundle_pointer = CandidateArtifactPointer {
                schema_version: CANDIDATE_POINTER_SCHEMA_VERSION.to_owned(),
                artifact_family: "intel_candidate_evidence_bundle".to_owned(),
                candidate_id: Some(bundle.candidate_id.clone()),
                screening_event_id: result.screening_event.screening_event_id.clone(),
                candidate_class: bundle.candidate_class.as_policy_key().to_owned(),
                storage_ref: S3ObjectPointer {
                    bucket: self.output_store.bucket().to_owned(),
                    key: bundle.bundle_key.clone(),
                    content_sha256: sha256_prefixed(&bundle_bytes),
                    schema_version: bundle.schema_version.clone(),
                },
                created_at_ms: bundle.created_at_ms,
            };
            self.publisher
                .publish_bundle_pointer(&bundle_pointer)
                .await?;
        }
        if let Some(state) = &result.hypothesis_state {
            let state_bytes = self
                .output_store
                .put_jsonl_record_or_existing(&state.state_key, state)
                .await?;
            let state_pointer = CandidateArtifactPointer {
                schema_version: CANDIDATE_POINTER_SCHEMA_VERSION.to_owned(),
                artifact_family: "intel_candidate_hypothesis_state".to_owned(),
                candidate_id: Some(state.hypothesis_id.clone()),
                screening_event_id: result.screening_event.screening_event_id.clone(),
                candidate_class: state.current_state.as_policy_key().to_owned(),
                storage_ref: S3ObjectPointer {
                    bucket: self.output_store.bucket().to_owned(),
                    key: state.state_key.clone(),
                    content_sha256: sha256_prefixed(&state_bytes),
                    schema_version: state.schema_version.clone(),
                },
                created_at_ms: state.updated_at_ms,
            };
            self.publisher
                .publish_hypothesis_state_pointer(&state_pointer)
                .await?;
        }
        self.publisher.flush().await
    }

    async fn read_market_feature_deltas(
        &self,
        packet: &StructuredIntelPacket,
    ) -> AppResult<Vec<MarketFeatureDelta>> {
        let reference = packet.market_context_ref.as_ref();
        let Some(reference) = reference else {
            return Ok(Vec::new());
        };
        let detail_key = reference
            .market_feature_delta_key
            .as_deref()
            .filter(|key| !key.trim().is_empty());
        if let Some(key) = reference
            .market_feature_delta_summary_key
            .as_deref()
            .filter(|key| !key.trim().is_empty())
        {
            let summary = self
                .market_store
                .get_json::<MarketFeatureDeltaSummary>(key)
                .await
                .ok();
            let Some(summary) = summary else {
                return Ok(Vec::new());
            };
            let summary_deltas = expand_market_feature_delta_summary(summary);
            if market_feature_deltas_satisfy_packet(packet, &summary_deltas) {
                return Ok(summary_deltas);
            }
            if let Some(detail_key) = detail_key {
                let detail_deltas = self
                    .market_store
                    .get_json::<Vec<MarketFeatureDelta>>(detail_key)
                    .await
                    .unwrap_or_default();
                if !detail_deltas.is_empty() {
                    return Ok(detail_deltas);
                }
            }
            return Ok(summary_deltas);
        }
        if let Some(key) = detail_key {
            return Ok(self
                .market_store
                .get_json::<Vec<MarketFeatureDelta>>(key)
                .await
                .unwrap_or_default());
        }
        Ok(Vec::new())
    }

    async fn is_stale_revision(&self, packet: &StructuredIntelPacket) -> AppResult<bool> {
        let Some(index) = self
            .latest_revision_index(
                effective_packet_family_id(packet),
                &self.policy.policy_version,
            )
            .await?
        else {
            return Ok(false);
        };
        Ok(packet.revision < index.latest_packet_revision
            || (packet.revision == index.latest_packet_revision
                && packet.packet_id == index.latest_packet_id))
    }

    async fn write_revision_index(
        &self,
        packet: &StructuredIntelPacket,
        result: &CandidateProcessingResult,
        updated_at_ms: i64,
    ) -> AppResult<()> {
        let index = CandidateRevisionIndex {
            schema_version: CANDIDATE_REVISION_INDEX_SCHEMA_VERSION.to_owned(),
            packet_family_id: effective_packet_family_id(packet).to_owned(),
            scoring_policy_version: self.policy.policy_version.clone(),
            latest_packet_revision: packet.revision,
            latest_packet_id: packet.packet_id.clone(),
            latest_screening_event_id: result.screening_event.screening_event_id.clone(),
            latest_candidate_id: result.screening_event.candidate_id.clone(),
            updated_at_ms,
        };
        self.output_store
            .put_bytes_idempotent(
                &revision_index_key(
                    effective_packet_family_id(packet),
                    &self.policy.policy_version,
                    packet.revision,
                ),
                serde_json::to_vec_pretty(&index)?,
                "application/json",
            )
            .await
    }

    async fn latest_revision_index(
        &self,
        packet_family_id: &str,
        scoring_policy_version: &str,
    ) -> AppResult<Option<CandidateRevisionIndex>> {
        let mut latest: Option<(u32, String)> = None;
        for key in self
            .output_store
            .list_keys(
                &revision_index_prefix(packet_family_id, scoring_policy_version),
                REVISION_INDEX_MAX_KEYS,
            )
            .await?
        {
            let Some(revision) = parse_revision_from_key(&key) else {
                continue;
            };
            let replace = latest
                .as_ref()
                .is_none_or(|(current_revision, _)| revision > *current_revision);
            if replace {
                latest = Some((revision, key));
            }
        }
        let Some((_, key)) = latest else {
            return Ok(None);
        };
        self.output_store.get_json(&key).await.map(Some)
    }
}

fn expand_market_feature_delta_summary(
    summary: MarketFeatureDeltaSummary,
) -> Vec<MarketFeatureDelta> {
    let mut deltas = Vec::new();
    for row in summary.rows {
        for metric in row.metrics {
            deltas.push(MarketFeatureDelta {
                schema_version: summary.schema_version.clone(),
                feature_delta_id: stable_id(
                    "market_delta_summary_metric",
                    &[
                        &summary.feature_delta_summary_id,
                        &row.venue,
                        &row.symbol_native,
                        &row.symbol_canonical,
                        &row.market_type,
                        &metric.metric_name,
                        &metric.window_start_ms.to_string(),
                        &metric.window_end_ms.to_string(),
                    ],
                ),
                l1_run_id: summary.l1_run_id.clone(),
                metric_name: metric.metric_name,
                venue: row.venue.clone(),
                symbol_native: row.symbol_native.clone(),
                symbol_canonical: row.symbol_canonical.clone(),
                market_type: row.market_type.clone(),
                value_now: metric.value_now,
                value_15m_ago: metric.value_15m_ago,
                value_1h_ago: metric.value_1h_ago,
                change_pct_15m: metric.change_pct_15m,
                change_pct_1h: metric.change_pct_1h,
                price_change_same_window: metric.price_change_same_window,
                volume_change_same_window: metric.volume_change_same_window,
                oi_price_divergence: metric.oi_price_divergence,
                window_start_ms: metric.window_start_ms,
                window_end_ms: metric.window_end_ms,
                known_as_of_ms: row.known_as_of_ms,
                quality_status: if metric.quality_status.trim().is_empty() {
                    row.quality_status.clone()
                } else {
                    metric.quality_status
                },
                missing_reasons: row.missing_reasons.clone(),
            });
        }
    }
    deltas
}

fn market_feature_deltas_satisfy_packet(
    packet: &StructuredIntelPacket,
    deltas: &[MarketFeatureDelta],
) -> bool {
    let Some(decision_available_at_ms) = packet.decision_available_at_ms else {
        return false;
    };
    packet.normalized_symbols.iter().any(|symbol| {
        let candidates = canonical_symbol_candidates(symbol);
        deltas.iter().any(|delta| {
            candidates.contains(&delta.symbol_canonical.to_ascii_uppercase())
                && delta.window_end_ms <= decision_available_at_ms
                && delta.known_as_of_ms <= decision_available_at_ms
                && is_usable_market_artifact_quality(&delta.quality_status)
                && (delta.change_pct_1h.is_some() || delta.change_pct_15m.is_some())
        })
    })
}

fn repair_raw_event_id(packet: &StructuredIntelPacket, key: &str) -> String {
    path_value(key, "raw_event_id")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| packet.packet_id.clone())
}

fn path_value(key: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    key.split('/')
        .find_map(|segment| segment.strip_prefix(&prefix))
        .map(ToOwned::to_owned)
}

fn canonical_symbol_candidates(symbol: &str) -> BTreeSet<String> {
    let upper = symbol.trim().to_ascii_uppercase();
    let mut values = BTreeSet::from([upper.clone()]);
    for suffix in ["USDT", "USDC", "USD", "BUSD", "BTC", "ETH"] {
        if upper.len() > suffix.len() && upper.ends_with(suffix) {
            values.insert(upper.trim_end_matches(suffix).to_owned());
        }
    }
    values
}

fn is_usable_market_artifact_quality(status: &str) -> bool {
    matches!(
        status.trim().to_ascii_lowercase().as_str(),
        "complete" | "partial" | "available"
    )
}

pub fn worker_help() -> &'static str {
    r#"intel-candidate-worker
Usage:
  intel-candidate-worker \
    --nats-url nats://REPLACE_WITH_S2S_NATS_HOST:4222 \
    --input-s3-bucket nangman-crypto-dev-intel-structuring-l1-<account-suffix> \
    --output-s3-bucket nangman-crypto-dev-intel-candidate-<account-suffix> \
    --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
    --policy-file /opt/nangman-crypto/intel-candidate/policies/scoring-policy.v1.json

The worker reads structured_intel_packet.created pointers, writes screening events
and research-eligible candidate evidence bundles idempotently. Non-research
candidates with enough lineage are preserved as hypothesis_state records so
the recomposition loop can rerun them after market, policy, or code changes.
The worker publishes output pointers and only then acknowledges the input
message."#
}

fn read_single_json_or_jsonl<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    display_path: &Path,
) -> AppResult<T> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| AppError::Json(format!("{}: {error}", display_path.display())))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(format!(
            "{} is empty",
            display_path.display()
        )));
    }
    if trimmed.starts_with('{')
        && let Ok(value) = serde_json::from_str(trimmed)
    {
        return Ok(value);
    }
    let first_line = trimmed
        .lines()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| {
            AppError::validation(format!("{} has no JSONL records", display_path.display()))
        })?;
    Ok(serde_json::from_str(first_line)?)
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256_hex(bytes))
}

fn validate_pointer_content_hash(pointer: &StructuredPointer, bytes: &[u8]) -> AppResult<()> {
    let actual = sha256_prefixed(bytes);
    if actual != pointer.storage_ref.content_sha256 {
        return Err(AppError::validation(format!(
            "structured packet content hash mismatch packet_id={} expected={} actual={}",
            pointer.packet_id, pointer.storage_ref.content_sha256, actual
        )));
    }
    Ok(())
}

fn next_string(
    values: &mut impl Iterator<Item = String>,
    message: &'static str,
) -> AppResult<String> {
    let value = values.next().ok_or_else(|| AppError::config(message))?;
    if value.trim().is_empty() {
        return Err(AppError::config(message));
    }
    Ok(value)
}

fn validate_bucket_arg(value: &str, name: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::config(format!("{name} requires a bucket")));
    }
    if value.contains('<') || value.contains('>') {
        return Err(AppError::config(format!(
            "{name} must be a real bucket name, not a public-doc placeholder"
        )));
    }
    Ok(())
}

fn absolute_path_arg(value: Option<String>, message: &str) -> AppResult<PathBuf> {
    let value = value.ok_or_else(|| AppError::config(message))?;
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(AppError::config(format!(
            "{message}; got {}",
            path.display()
        )));
    }
    Ok(path)
}

fn positive_u64_arg(value: Option<String>, name: &str) -> AppResult<u64> {
    let raw = value.ok_or_else(|| AppError::config(format!("{name} requires a number")))?;
    let parsed = raw
        .parse::<u64>()
        .map_err(|_| AppError::config(format!("{name} must be a positive integer")))?;
    if parsed == 0 {
        return Err(AppError::config(format!(
            "{name} must be greater than zero"
        )));
    }
    Ok(parsed)
}

fn positive_i64_arg(value: Option<String>, name: &str) -> AppResult<i64> {
    let raw = value.ok_or_else(|| AppError::config(format!("{name} requires a number")))?;
    let parsed = raw
        .parse::<i64>()
        .map_err(|_| AppError::config(format!("{name} must be a positive integer")))?;
    if parsed <= 0 {
        return Err(AppError::config(format!(
            "{name} must be greater than zero"
        )));
    }
    Ok(parsed)
}

fn positive_usize_arg(value: Option<String>, name: &str) -> AppResult<usize> {
    let raw = value.ok_or_else(|| AppError::config(format!("{name} requires a number")))?;
    let parsed = raw
        .parse::<usize>()
        .map_err(|_| AppError::config(format!("{name} must be a positive integer")))?;
    if parsed == 0 {
        return Err(AppError::config(format!(
            "{name} must be greater than zero"
        )));
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{STRUCTURED_PACKET_SCHEMA_VERSION, STRUCTURED_POINTER_SCHEMA_VERSION};
    use serde_json::json;

    #[test]
    fn worker_defaults_follow_candidate_contract_names() {
        let args = WorkerArgs::default();
        assert_eq!(args.nats.input_stream, "STRUCTURED_INTEL");
        assert_eq!(args.nats.output_stream, "INTEL_CANDIDATE");
        assert_eq!(
            args.nats.hypothesis_state_subject,
            "intel_candidate_hypothesis_state.created"
        );
        assert_eq!(args.output_store.bucket, DEFAULT_OUTPUT_BUCKET);
    }

    #[test]
    fn parses_required_nats_url() {
        let args = WorkerArgs::parse(
            [
                "--nats-url",
                "nats://127.0.0.1:4222",
                "--input-s3-bucket",
                "test-structured-l1",
                "--output-s3-bucket",
                "test-candidate",
                "--market-l1-s3-bucket",
                "test-market-l1",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap()
        .unwrap();
        assert_eq!(args.nats.url, "nats://127.0.0.1:4222");
    }

    #[test]
    fn accepts_nats_url_from_environment() {
        unsafe {
            std::env::set_var("NATS_URL", "nats://127.0.0.1:4222");
        }
        let args = WorkerArgs::parse(
            [
                "--input-s3-bucket",
                "test-structured-l1",
                "--output-s3-bucket",
                "test-candidate",
                "--market-l1-s3-bucket",
                "test-market-l1",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap()
        .unwrap();
        assert_eq!(args.nats.url, "nats://127.0.0.1:4222");
        unsafe {
            std::env::remove_var("NATS_URL");
        }
    }

    #[test]
    fn rejects_public_doc_bucket_placeholder() {
        let err = WorkerArgs::parse(
            [
                "--nats-url",
                "nats://127.0.0.1:4222",
                "--input-s3-bucket",
                DEFAULT_INPUT_BUCKET,
                "--output-s3-bucket",
                "test-candidate",
                "--market-l1-s3-bucket",
                "test-market-l1",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap_err();

        assert!(err.to_string().contains("--input-s3-bucket"));
        assert!(err.to_string().contains("public-doc placeholder"));
    }

    #[test]
    fn reads_pretty_json_or_jsonl_first_record() {
        let json = br#"{"value":1}"#;
        let value: serde_json::Value = read_single_json_or_jsonl(json, Path::new("x")).unwrap();
        assert_eq!(value["value"], 1);

        let jsonl = br#"{"value":2}
{"value":3}
"#;
        let value: serde_json::Value = read_single_json_or_jsonl(jsonl, Path::new("x")).unwrap();
        assert_eq!(value["value"], 2);
    }

    #[test]
    fn revision_index_key_is_scoped_by_scoring_policy() {
        assert_eq!(
            revision_index_key("family/001", "intel_candidate_scoring_v2", 1),
            "candidate-revision-index/schema=intel_candidate_revision_index_v1/packet_family_id=family_001/scoring_policy=intel_candidate_scoring_v2/revision=0000000001.json"
        );
    }

    #[test]
    fn repair_s3_key_recovers_raw_event_id_from_partition() {
        let mut packet = packet_for_test();
        packet.raw_event_id.clear();
        let key = "structured-intel-packet/schema=structured_intel_packet_v1/dt=2026-05-22/hour=04/raw_event_id=intel_evt_abc/packet_id=intel_pkt_123/part-000001.jsonl";
        assert_eq!(repair_raw_event_id(&packet, key), "intel_evt_abc");
    }

    #[test]
    fn repair_s3_key_falls_back_to_packet_id_when_raw_event_id_is_missing() {
        let mut packet = packet_for_test();
        packet.raw_event_id.clear();
        assert_eq!(
            repair_raw_event_id(
                &packet,
                "structured-intel-packet/schema=structured_intel_packet_v1/x.jsonl"
            ),
            packet.packet_id
        );
    }

    #[test]
    fn validates_pointer_content_hash_before_scoring() {
        let pointer = StructuredPointer {
            schema_version: STRUCTURED_POINTER_SCHEMA_VERSION.to_owned(),
            packet_id: "packet_001".to_owned(),
            raw_event_id: "raw_001".to_owned(),
            terminal_decision: serde_json::Value::String("high_confidence_structured".to_owned()),
            storage_ref: S3ObjectPointer {
                bucket: DEFAULT_INPUT_BUCKET.to_owned(),
                key: "structured-intel-packet/schema=structured_intel_packet_v1/x.jsonl".to_owned(),
                content_sha256: sha256_prefixed(b"payload"),
                schema_version: STRUCTURED_PACKET_SCHEMA_VERSION.to_owned(),
            },
            manifest_key: "manifests/schema=intel_l1_manifest_v1/run.json".to_owned(),
            created_at_ms: 1,
        };
        assert!(validate_pointer_content_hash(&pointer, b"payload").is_ok());
        assert!(validate_pointer_content_hash(&pointer, b"tampered").is_err());
    }

    #[test]
    fn expands_market_feature_delta_summary_for_candidate_scoring() {
        let deltas = expand_market_feature_delta_summary(MarketFeatureDeltaSummary {
            schema_version: "market_feature_delta_summary_v1".to_owned(),
            feature_delta_summary_id: "summary_001".to_owned(),
            l1_run_id: "l1_001".to_owned(),
            detail_feature_delta_key: "market_feature_delta/run_id=l1_001/delta.json".to_owned(),
            window_start_ms: 1_000,
            window_end_ms: 2_000,
            known_as_of_ms: 2_100,
            detail_record_count: 900,
            summary_row_count: 1,
            rows: vec![crate::model::MarketFeatureDeltaSummaryRow {
                venue: "binance".to_owned(),
                symbol_native: "SUIUSDT".to_owned(),
                symbol_canonical: "SUI".to_owned(),
                market_type: "spot".to_owned(),
                window_start_ms: 1_000,
                window_end_ms: 2_000,
                known_as_of_ms: 2_100,
                quality_status: "complete".to_owned(),
                missing_reasons: Vec::new(),
                metrics: vec![crate::model::MarketFeatureDeltaSummaryMetric {
                    metric_name: "price".to_owned(),
                    value_now: 1.25,
                    value_15m_ago: Some(1.2),
                    value_1h_ago: Some(1.1),
                    change_pct_15m: Some(4.16),
                    change_pct_1h: Some(13.63),
                    price_change_same_window: Some(13.63),
                    volume_change_same_window: Some(5.0),
                    oi_price_divergence: None,
                    window_start_ms: 1_000,
                    window_end_ms: 2_000,
                    quality_status: "complete".to_owned(),
                }],
            }],
        });

        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].schema_version, "market_feature_delta_summary_v1");
        assert_eq!(deltas[0].l1_run_id, "l1_001");
        assert_eq!(deltas[0].symbol_canonical, "SUI");
        assert_eq!(deltas[0].metric_name, "price");
        assert_eq!(deltas[0].known_as_of_ms, 2_100);
        assert!(
            deltas[0]
                .feature_delta_id
                .starts_with("market_delta_summary_metric_")
        );
    }

    #[test]
    fn summary_delta_without_decision_safe_metric_requires_detail_fallback() {
        let packet: StructuredIntelPacket = serde_json::from_value(json!({
            "packet_id": "packet_001",
            "cluster_id": "cluster_001",
            "decision_available_at_ms": 2_000,
            "normalized_symbols": ["SUIUSDT"]
        }))
        .expect("minimal packet uses serde defaults");
        let future_delta = MarketFeatureDelta {
            schema_version: "market_feature_delta_summary_v1".to_owned(),
            feature_delta_id: "future_delta".to_owned(),
            l1_run_id: "l1_001".to_owned(),
            metric_name: "price".to_owned(),
            venue: "binance".to_owned(),
            symbol_native: "SUIUSDT".to_owned(),
            symbol_canonical: "SUI".to_owned(),
            market_type: "spot".to_owned(),
            value_now: 1.2,
            value_15m_ago: Some(1.1),
            value_1h_ago: Some(1.0),
            change_pct_15m: Some(9.0),
            change_pct_1h: Some(20.0),
            price_change_same_window: Some(20.0),
            volume_change_same_window: Some(5.0),
            oi_price_divergence: None,
            window_start_ms: 2_500,
            window_end_ms: 3_000,
            known_as_of_ms: 3_000,
            quality_status: "complete".to_owned(),
            missing_reasons: Vec::new(),
        };
        let valid_delta = MarketFeatureDelta {
            window_start_ms: 1_500,
            window_end_ms: 1_900,
            known_as_of_ms: 1_950,
            ..future_delta.clone()
        };

        assert!(!market_feature_deltas_satisfy_packet(
            &packet,
            &[future_delta]
        ));
        assert!(market_feature_deltas_satisfy_packet(
            &packet,
            &[valid_delta]
        ));
    }

    fn packet_for_test() -> StructuredIntelPacket {
        serde_json::from_value(json!({
            "packet_id": "packet_001",
            "packet_family_id": "family_001",
            "raw_event_id": "raw_001",
            "cluster_id": "cluster_001",
            "source_event_ids": ["raw_001"],
            "schema_version": STRUCTURED_PACKET_SCHEMA_VERSION
        }))
        .expect("valid packet")
    }
}
