use crate::error::{AppError, AppResult};
use crate::model::{
    CANDIDATE_BUNDLE_SCHEMA_VERSION, CANDIDATE_POINTER_SCHEMA_VERSION,
    HYPOTHESIS_STATE_SCHEMA_VERSION, SCREENING_EVENT_SCHEMA_VERSION,
    STRUCTURED_PACKET_SCHEMA_VERSION, STRUCTURED_POINTER_SCHEMA_VERSION,
};
use async_nats::jetstream;
use async_nats::jetstream::consumer::PullConsumer;
use async_nats::jetstream::consumer::{AckPolicy, DeliverPolicy};
use async_nats::jetstream::stream;
use bytes::Bytes;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct StructuredPointer {
    pub schema_version: String,
    pub packet_id: String,
    pub raw_event_id: String,
    pub terminal_decision: serde_json::Value,
    pub storage_ref: S3ObjectPointer,
    pub manifest_key: String,
    pub created_at_ms: i64,
}

impl StructuredPointer {
    pub fn validate(&self) -> AppResult<()> {
        if self.schema_version != STRUCTURED_POINTER_SCHEMA_VERSION {
            return Err(AppError::validation(format!(
                "structured pointer schema mismatch expected={} actual={}",
                STRUCTURED_POINTER_SCHEMA_VERSION, self.schema_version
            )));
        }
        if self.packet_id.trim().is_empty() {
            return Err(AppError::validation(
                "structured pointer packet_id is required",
            ));
        }
        if self.raw_event_id.trim().is_empty() {
            return Err(AppError::validation(
                "structured pointer raw_event_id is required",
            ));
        }
        if self.manifest_key.trim().is_empty() {
            return Err(AppError::validation(
                "structured pointer manifest_key is required",
            ));
        }
        self.storage_ref
            .validate(STRUCTURED_PACKET_SCHEMA_VERSION)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct S3ObjectPointer {
    pub bucket: String,
    pub key: String,
    pub content_sha256: String,
    pub schema_version: String,
}

impl S3ObjectPointer {
    fn validate(&self, expected_schema_version: &str) -> AppResult<()> {
        if self.bucket.trim().is_empty() {
            return Err(AppError::validation("pointer storage bucket is required"));
        }
        if self.key.trim().is_empty() {
            return Err(AppError::validation("pointer storage key is required"));
        }
        if !self.content_sha256.starts_with("sha256:") {
            return Err(AppError::validation(
                "pointer storage content_sha256 must be sha256-prefixed",
            ));
        }
        if self.schema_version != expected_schema_version {
            return Err(AppError::validation(format!(
                "pointer storage schema mismatch expected={} actual={}",
                expected_schema_version, self.schema_version
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CandidateArtifactPointer {
    pub schema_version: String,
    pub artifact_family: String,
    pub candidate_id: Option<String>,
    pub screening_event_id: String,
    pub candidate_class: String,
    pub storage_ref: S3ObjectPointer,
    pub created_at_ms: i64,
}

impl CandidateArtifactPointer {
    fn validate(&self) -> AppResult<()> {
        if self.schema_version != CANDIDATE_POINTER_SCHEMA_VERSION {
            return Err(AppError::validation(format!(
                "candidate pointer schema mismatch expected={} actual={}",
                CANDIDATE_POINTER_SCHEMA_VERSION, self.schema_version
            )));
        }
        if self.screening_event_id.trim().is_empty() {
            return Err(AppError::validation(
                "candidate pointer screening_event_id is required",
            ));
        }
        if self.candidate_class.trim().is_empty() {
            return Err(AppError::validation(
                "candidate pointer candidate_class is required",
            ));
        }
        let expected_storage_schema = match self.artifact_family.as_str() {
            "intel_candidate_screening_event" => SCREENING_EVENT_SCHEMA_VERSION,
            "intel_candidate_evidence_bundle" => CANDIDATE_BUNDLE_SCHEMA_VERSION,
            "intel_candidate_hypothesis_state" => HYPOTHESIS_STATE_SCHEMA_VERSION,
            other => {
                return Err(AppError::validation(format!(
                    "unsupported candidate artifact family: {other}"
                )));
            }
        };
        self.storage_ref.validate(expected_storage_schema)?;
        Ok(())
    }
}

pub struct StructuredIntelConsumer {
    consumer: PullConsumer,
    batch_size: usize,
}

pub struct StructuredIntelMessage {
    inner: async_nats::jetstream::Message,
}

pub struct CandidatePublisher {
    client: async_nats::Client,
    jetstream: jetstream::Context,
    stream: String,
    bundle_subject: String,
    screening_subject: String,
    hypothesis_state_subject: String,
}

impl StructuredIntelConsumer {
    pub async fn connect(config: &NatsConfig) -> AppResult<Self> {
        let client = async_nats::connect(&config.url)
            .await
            .map_err(|error| AppError::nats(format!("connect {}: {error}", config.url)))?;
        let jetstream = jetstream::new(client);
        let stream = jetstream
            .get_stream(&config.input_stream)
            .await
            .map_err(|error| {
                AppError::nats(format!("get input stream {}: {error}", config.input_stream))
            })?;
        let consumer = stream
            .get_or_create_consumer(
                &config.input_consumer,
                jetstream::consumer::pull::Config {
                    durable_name: Some(config.input_consumer.clone()),
                    filter_subject: config.input_subject.clone(),
                    ack_policy: AckPolicy::Explicit,
                    ack_wait: Duration::from_secs(config.ack_wait_secs),
                    max_deliver: config.max_deliver,
                    max_ack_pending: config.batch_size as i64,
                    deliver_policy: deliver_policy(&config.input_deliver_policy)?,
                    ..Default::default()
                },
            )
            .await
            .map_err(|error| {
                AppError::nats(format!(
                    "get/create input consumer {} on stream {}: {error}",
                    config.input_consumer, config.input_stream
                ))
            })?;
        Ok(Self {
            consumer,
            batch_size: config.batch_size.max(1),
        })
    }

    pub async fn next_message(&mut self) -> AppResult<Option<StructuredIntelMessage>> {
        let mut messages = self
            .consumer
            .fetch()
            .max_messages(self.batch_size)
            .expires(Duration::from_secs(5))
            .messages()
            .await
            .map_err(|error| AppError::nats(format!("fetch structured intel messages: {error}")))?;
        match messages.next().await {
            Some(Ok(message)) => Ok(Some(StructuredIntelMessage { inner: message })),
            Some(Err(error)) => Err(AppError::nats(format!(
                "read structured intel message: {error}"
            ))),
            None => Ok(None),
        }
    }
}

impl StructuredIntelMessage {
    pub fn pointer(&self) -> AppResult<StructuredPointer> {
        let pointer: StructuredPointer = serde_json::from_slice(&self.inner.payload)?;
        pointer.validate()?;
        Ok(pointer)
    }

    pub async fn ack(self) -> AppResult<()> {
        self.inner
            .double_ack()
            .await
            .map_err(|error| AppError::nats(format!("structured intel double ack failed: {error}")))
    }
}

impl CandidatePublisher {
    pub async fn connect(config: &NatsConfig) -> AppResult<Self> {
        let client = async_nats::connect(&config.url)
            .await
            .map_err(|error| AppError::nats(format!("connect {}: {error}", config.url)))?;
        let jetstream = jetstream::new(client.clone());
        if config.ensure_output_stream {
            jetstream
                .get_or_create_stream(stream::Config {
                    name: config.output_stream.clone(),
                    subjects: vec![
                        config.bundle_subject.clone(),
                        config.screening_subject.clone(),
                        config.hypothesis_state_subject.clone(),
                        config.health_subject.clone(),
                    ],
                    retention: stream::RetentionPolicy::Limits,
                    storage: stream::StorageType::File,
                    max_age: Duration::from_secs(config.output_stream_max_age_secs),
                    duplicate_window: Duration::from_secs(
                        config.output_stream_duplicate_window_secs,
                    ),
                    ..Default::default()
                })
                .await
                .map_err(|error| {
                    AppError::nats(format!(
                        "get/create output stream {}: {error}",
                        config.output_stream
                    ))
                })?;
        }
        Ok(Self {
            client,
            jetstream,
            stream: config.output_stream.clone(),
            bundle_subject: config.bundle_subject.clone(),
            screening_subject: config.screening_subject.clone(),
            hypothesis_state_subject: config.hypothesis_state_subject.clone(),
        })
    }

    pub async fn publish_bundle_pointer(
        &self,
        pointer: &CandidateArtifactPointer,
    ) -> AppResult<()> {
        pointer.validate()?;
        let message_id = pointer
            .candidate_id
            .as_deref()
            .unwrap_or(pointer.screening_event_id.as_str());
        self.publish(&self.bundle_subject, message_id, pointer)
            .await
    }

    pub async fn publish_screening_pointer(
        &self,
        pointer: &CandidateArtifactPointer,
    ) -> AppResult<()> {
        pointer.validate()?;
        self.publish(
            &self.screening_subject,
            &pointer.screening_event_id,
            pointer,
        )
        .await
    }

    pub async fn publish_hypothesis_state_pointer(
        &self,
        pointer: &CandidateArtifactPointer,
    ) -> AppResult<()> {
        pointer.validate()?;
        self.publish(
            &self.hypothesis_state_subject,
            &pointer.screening_event_id,
            pointer,
        )
        .await
    }

    pub async fn flush(&self) -> AppResult<()> {
        self.client
            .flush()
            .await
            .map_err(|error| AppError::nats(format!("flush: {error}")))
    }

    async fn publish<T: Serialize>(
        &self,
        subject: &str,
        message_id: &str,
        payload: &T,
    ) -> AppResult<()> {
        let bytes = Bytes::from(serde_json::to_vec(payload)?);
        let message = jetstream::message::PublishMessage::build()
            .expected_stream(&self.stream)
            .message_id(message_id)
            .payload(bytes);
        let ack = self
            .jetstream
            .send_publish(subject.to_owned(), message)
            .await
            .map_err(|error| AppError::nats(format!("publish {subject}: {error}")))?
            .await
            .map_err(|error| AppError::nats(format!("publish ack {subject}: {error}")))?;
        if ack.stream != self.stream {
            return Err(AppError::nats(format!(
                "publish ack stream mismatch expected={} actual={}",
                self.stream, ack.stream
            )));
        }
        Ok(())
    }
}

fn deliver_policy(value: &str) -> AppResult<DeliverPolicy> {
    match value {
        "all" => Ok(DeliverPolicy::All),
        "new" => Ok(DeliverPolicy::New),
        "last" => Ok(DeliverPolicy::Last),
        "last_per_subject" => Ok(DeliverPolicy::LastPerSubject),
        other => Err(AppError::config(format!(
            "unsupported candidate deliver policy: {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_supported_deliver_policies() {
        assert!(matches!(deliver_policy("all").unwrap(), DeliverPolicy::All));
        assert!(matches!(deliver_policy("new").unwrap(), DeliverPolicy::New));
        assert!(matches!(
            deliver_policy("last").unwrap(),
            DeliverPolicy::Last
        ));
        assert!(matches!(
            deliver_policy("last_per_subject").unwrap(),
            DeliverPolicy::LastPerSubject
        ));
    }

    #[test]
    fn validates_structured_pointer_contract() {
        let pointer = StructuredPointer {
            schema_version: STRUCTURED_POINTER_SCHEMA_VERSION.to_owned(),
            packet_id: "packet_001".to_owned(),
            raw_event_id: "raw_001".to_owned(),
            terminal_decision: json!("high_confidence_structured"),
            storage_ref: S3ObjectPointer {
                bucket: "intel-l1".to_owned(),
                key: "structured-intel-packet/schema=structured_intel_packet_v1/x.jsonl".to_owned(),
                content_sha256: "sha256:abc".to_owned(),
                schema_version: STRUCTURED_PACKET_SCHEMA_VERSION.to_owned(),
            },
            manifest_key: "manifests/schema=intel_l1_manifest_v1/run.json".to_owned(),
            created_at_ms: 1,
        };
        assert!(pointer.validate().is_ok());
    }

    #[test]
    fn rejects_legacy_structured_pointer_schema_name() {
        let legacy_schema = ["structured", "intel", "pointer", "v1"].join("_");
        let pointer = StructuredPointer {
            schema_version: legacy_schema,
            packet_id: "packet_001".to_owned(),
            raw_event_id: "raw_001".to_owned(),
            terminal_decision: json!("high_confidence_structured"),
            storage_ref: S3ObjectPointer {
                bucket: "intel-l1".to_owned(),
                key: "structured-intel-packet/schema=structured_intel_packet_v1/x.jsonl".to_owned(),
                content_sha256: "sha256:abc".to_owned(),
                schema_version: STRUCTURED_PACKET_SCHEMA_VERSION.to_owned(),
            },
            manifest_key: "manifests/schema=intel_l1_manifest_v1/run.json".to_owned(),
            created_at_ms: 1,
        };
        assert!(pointer.validate().is_err());
    }

    #[test]
    fn validates_candidate_artifact_pointer_contract() {
        let pointer = CandidateArtifactPointer {
            schema_version: CANDIDATE_POINTER_SCHEMA_VERSION.to_owned(),
            artifact_family: "intel_candidate_screening_event".to_owned(),
            candidate_id: Some("cand_001".to_owned()),
            screening_event_id: "screen_001".to_owned(),
            candidate_class: "research_candidate".to_owned(),
            storage_ref: S3ObjectPointer {
                bucket: "candidate".to_owned(),
                key: "candidate-screening/schema=intel_candidate_screening_event_v1/x.jsonl"
                    .to_owned(),
                content_sha256: "sha256:def".to_owned(),
                schema_version: SCREENING_EVENT_SCHEMA_VERSION.to_owned(),
            },
            created_at_ms: 1,
        };
        assert!(pointer.validate().is_ok());
    }
}
