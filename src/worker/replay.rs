use super::*;

impl CandidateWorker {
    pub async fn write_replay_artifacts(
        &self,
        result: &CandidateProcessingResult,
    ) -> AppResult<()> {
        let screening_key = screening_event_key(
            result.screening_event.created_at_ms,
            &result.screening_event.screening_event_id,
        );
        self.output_store
            .put_jsonl_record_or_existing(&screening_key, &result.screening_event)
            .await?;
        if let Some(bundle) = &result.evidence_bundle {
            self.output_store
                .put_jsonl_record_or_existing(&bundle.bundle_key, bundle)
                .await?;
        }
        if let Some(state) = &result.hypothesis_state {
            self.output_store
                .put_jsonl_record_or_existing(&state.state_key, state)
                .await?;
        }
        Ok(())
    }

    pub async fn list_replay_input_keys(
        &self,
        prefix: &str,
        max_keys: usize,
    ) -> AppResult<Vec<String>> {
        self.list_replay_input_key_page(prefix, max_keys, None)
            .await
            .map(|page| page.keys)
    }

    pub async fn list_replay_input_key_page(
        &self,
        prefix: &str,
        max_keys: usize,
        start_after: Option<&str>,
    ) -> AppResult<ReplayInputKeyPage> {
        let ListKeysPage {
            mut keys,
            next_start_after,
        } = self
            .input_store
            .list_keys_page(prefix, max_keys, start_after)
            .await?;
        keys.retain(|key| {
            let normalized = key.to_ascii_lowercase();
            normalized.ends_with(".json") || normalized.ends_with(".jsonl")
        });
        keys.sort();
        Ok(ReplayInputKeyPage {
            keys,
            next_start_after,
        })
    }
}
