use super::super::{
    DEFAULT_AWS_REGION, DEFAULT_BUNDLE_SUBJECT, DEFAULT_HEALTH_SUBJECT,
    DEFAULT_HYPOTHESIS_STATE_SUBJECT, DEFAULT_INPUT_BUCKET, DEFAULT_INPUT_CONSUMER,
    DEFAULT_INPUT_STREAM, DEFAULT_INPUT_SUBJECT, DEFAULT_MARKET_L1_BUCKET, DEFAULT_OUTPUT_BUCKET,
    DEFAULT_OUTPUT_STREAM, DEFAULT_SCREENING_SUBJECT, NatsConfig, ObjectStoreConfig,
};
use super::WorkerArgs;
use crate::policy::DEFAULT_POLICY_PATH;
use std::path::PathBuf;

pub(super) fn default_worker_args() -> WorkerArgs {
    let input_store = ObjectStoreConfig {
        bucket: DEFAULT_INPUT_BUCKET.to_owned(),
        region: DEFAULT_AWS_REGION.to_owned(),
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
    WorkerArgs {
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
