#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NatsConfig {
    pub url: String,
    pub input_stream: String,
    pub input_subject: String,
    pub input_consumer: String,
    pub input_deliver_policy: String,
    pub output_stream: String,
    pub bundle_subject: String,
    pub screening_subject: String,
    pub hypothesis_state_subject: String,
    pub health_subject: String,
    pub ensure_output_stream: bool,
    pub output_stream_max_age_secs: u64,
    pub output_stream_duplicate_window_secs: u64,
    pub ack_wait_secs: u64,
    pub max_deliver: i64,
    pub batch_size: usize,
}
