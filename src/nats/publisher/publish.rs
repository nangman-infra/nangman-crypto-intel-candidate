use super::CandidatePublisher;
use super::message_id::bundle_message_id;
use crate::error::{AppError, AppResult};
use crate::nats::pointer::CandidateArtifactPointer;
use async_nats::jetstream;
use bytes::Bytes;
use serde::Serialize;

impl CandidatePublisher {
    pub async fn publish_bundle_pointer(
        &self,
        pointer: &CandidateArtifactPointer,
    ) -> AppResult<()> {
        pointer.validate()?;
        self.publish(&self.bundle_subject, bundle_message_id(pointer), pointer)
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
