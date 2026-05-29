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
use crate::policy::{ScoringPolicy, load_policy};
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
use content::*;
use market::*;
use revision::*;

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
