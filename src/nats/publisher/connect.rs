use super::CandidatePublisher;
use crate::error::{AppError, AppResult};
use crate::nats::config::NatsConfig;
use async_nats::jetstream;
use async_nats::jetstream::stream;
use std::time::Duration;

impl CandidatePublisher {
    pub async fn connect(config: &NatsConfig) -> AppResult<Self> {
        let client = async_nats::connect(&config.url)
            .await
            .map_err(|error| AppError::nats(format!("connect {}: {error}", config.url)))?;
        let jetstream = jetstream::new(client.clone());
        if config.ensure_output_stream {
            ensure_output_stream(&jetstream, config).await?;
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
}

async fn ensure_output_stream(
    jetstream: &jetstream::Context,
    config: &NatsConfig,
) -> AppResult<()> {
    jetstream
        .get_or_create_stream(output_stream_config(config))
        .await
        .map_err(|error| {
            AppError::nats(format!(
                "get/create output stream {}: {error}",
                config.output_stream
            ))
        })?;
    Ok(())
}

fn output_stream_config(config: &NatsConfig) -> stream::Config {
    stream::Config {
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
        duplicate_window: Duration::from_secs(config.output_stream_duplicate_window_secs),
        ..Default::default()
    }
}
