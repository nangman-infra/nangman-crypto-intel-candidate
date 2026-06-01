mod connect;
mod message_id;
mod publish;

use async_nats::jetstream;

pub struct CandidatePublisher {
    pub(super) client: async_nats::Client,
    pub(super) jetstream: jetstream::Context,
    pub(super) stream: String,
    pub(super) bundle_subject: String,
    pub(super) screening_subject: String,
    pub(super) hypothesis_state_subject: String,
}
