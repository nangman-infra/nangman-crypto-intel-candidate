use super::config::NatsConfig;
use super::pointer::CandidateArtifactPointer;
use crate::error::{AppError, AppResult};
use async_nats::jetstream;
use async_nats::jetstream::stream;
use bytes::Bytes;
use serde::Serialize;
use std::time::Duration;

pub struct CandidatePublisher {
    client: async_nats::Client,
    jetstream: jetstream::Context,
    stream: String,
    bundle_subject: String,
    screening_subject: String,
    hypothesis_state_subject: String,
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
