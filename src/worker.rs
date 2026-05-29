use crate::error::AppResult;
use crate::nats::{CandidatePublisher, NatsConfig};
use crate::policy::{ScoringPolicy, load_policy};
use crate::storage::{ObjectStore, ObjectStoreConfig};

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
mod args;
mod args_support;
mod content;
mod market;
mod process;
mod publish;
mod replay;
mod revision;
mod revision_state;
mod score;

pub use args::WorkerArgs;
pub use args_support::worker_help;

#[cfg(test)]
use content::{read_single_json_or_jsonl, sha256_prefixed, validate_pointer_content_hash};
#[cfg(test)]
use market::{
    expand_market_feature_delta_summary, market_feature_deltas_satisfy_packet, repair_raw_event_id,
};
#[cfg(test)]
use revision::revision_index_key;

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
}

#[cfg(test)]
mod tests;
