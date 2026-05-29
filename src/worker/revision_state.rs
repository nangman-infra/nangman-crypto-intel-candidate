use super::*;

impl CandidateWorker {
    pub(super) async fn is_stale_revision(
        &self,
        packet: &StructuredIntelPacket,
    ) -> AppResult<bool> {
        let Some(index) = self
            .latest_revision_index(
                effective_packet_family_id(packet),
                &self.policy.policy_version,
            )
            .await?
        else {
            return Ok(false);
        };
        Ok(packet.revision < index.latest_packet_revision
            || (packet.revision == index.latest_packet_revision
                && packet.packet_id == index.latest_packet_id))
    }

    pub(super) async fn write_revision_index(
        &self,
        packet: &StructuredIntelPacket,
        result: &CandidateProcessingResult,
        updated_at_ms: i64,
    ) -> AppResult<()> {
        let index = CandidateRevisionIndex {
            schema_version: CANDIDATE_REVISION_INDEX_SCHEMA_VERSION.to_owned(),
            packet_family_id: effective_packet_family_id(packet).to_owned(),
            scoring_policy_version: self.policy.policy_version.clone(),
            latest_packet_revision: packet.revision,
            latest_packet_id: packet.packet_id.clone(),
            latest_screening_event_id: result.screening_event.screening_event_id.clone(),
            latest_candidate_id: result.screening_event.candidate_id.clone(),
            updated_at_ms,
        };
        self.output_store
            .put_bytes_idempotent(
                &revision_index_key(
                    effective_packet_family_id(packet),
                    &self.policy.policy_version,
                    packet.revision,
                ),
                serde_json::to_vec_pretty(&index)?,
                "application/json",
            )
            .await
    }

    async fn latest_revision_index(
        &self,
        packet_family_id: &str,
        scoring_policy_version: &str,
    ) -> AppResult<Option<CandidateRevisionIndex>> {
        let mut latest: Option<(u32, String)> = None;
        for key in self
            .output_store
            .list_keys(
                &revision_index_prefix(packet_family_id, scoring_policy_version),
                REVISION_INDEX_MAX_KEYS,
            )
            .await?
        {
            let Some(revision) = parse_revision_from_key(&key) else {
                continue;
            };
            let replace = latest
                .as_ref()
                .is_none_or(|(current_revision, _)| revision > *current_revision);
            if replace {
                latest = Some((revision, key));
            }
        }
        let Some((_, key)) = latest else {
            return Ok(None);
        };
        self.output_store.get_json(&key).await.map(Some)
    }
}
