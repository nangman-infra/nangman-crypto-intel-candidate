use super::config::NatsConfig;
use super::pointer::StructuredPointer;
use crate::error::{AppError, AppResult};
use async_nats::jetstream;
use async_nats::jetstream::consumer::PullConsumer;
use async_nats::jetstream::consumer::{AckPolicy, DeliverPolicy};
use futures_util::StreamExt;
use std::time::Duration;

pub struct StructuredIntelConsumer {
    consumer: PullConsumer,
    batch_size: usize,
}

pub struct StructuredIntelMessage {
    inner: async_nats::jetstream::Message,
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

pub(super) fn deliver_policy(value: &str) -> AppResult<DeliverPolicy> {
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
