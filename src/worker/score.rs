use super::*;

impl CandidateWorker {
    pub async fn score_s3_key(
        &self,
        key: &str,
        created_at_ms: i64,
    ) -> AppResult<CandidateProcessingResult> {
        let packet_bytes = self.input_store.get_bytes(key).await?;
        let packet: StructuredIntelPacket =
            read_single_json_or_jsonl(&packet_bytes, Path::new(key))?;
        self.score_packet(packet, created_at_ms).await
    }

    pub(super) async fn score_packet(
        &self,
        packet: StructuredIntelPacket,
        created_at_ms: i64,
    ) -> AppResult<CandidateProcessingResult> {
        let universe = match packet
            .market_context_ref
            .as_ref()
            .and_then(|reference| reference.symbol_universe_snapshot_key.as_deref())
        {
            Some(key) if !key.trim().is_empty() => self
                .market_store
                .get_json::<SymbolUniverseSnapshot>(key)
                .await
                .ok(),
            _ => None,
        };
        let market_feature_deltas = self.read_market_feature_deltas(&packet).await?;
        let market_regime_contexts = match packet
            .market_context_ref
            .as_ref()
            .and_then(|reference| reference.market_regime_context_key.as_deref())
        {
            Some(key) if !key.trim().is_empty() => self
                .market_store
                .get_json::<Vec<MarketRegimeContext>>(key)
                .await
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let result = process_packet_with_artifacts(
            packet.clone(),
            &self.policy,
            MarketArtifactInputs {
                universe: universe.as_ref(),
                market_feature_deltas: &market_feature_deltas,
                market_regime_contexts: &market_regime_contexts,
            },
            created_at_ms,
        );
        Ok(result)
    }

    async fn read_market_feature_deltas(
        &self,
        packet: &StructuredIntelPacket,
    ) -> AppResult<Vec<MarketFeatureDelta>> {
        let reference = packet.market_context_ref.as_ref();
        let Some(reference) = reference else {
            return Ok(Vec::new());
        };
        let detail_key = reference
            .market_feature_delta_key
            .as_deref()
            .filter(|key| !key.trim().is_empty());
        if let Some(key) = reference
            .market_feature_delta_summary_key
            .as_deref()
            .filter(|key| !key.trim().is_empty())
        {
            let summary = self
                .market_store
                .get_json::<MarketFeatureDeltaSummary>(key)
                .await
                .ok();
            let Some(summary) = summary else {
                return Ok(Vec::new());
            };
            let summary_deltas = expand_market_feature_delta_summary(summary);
            if market_feature_deltas_satisfy_packet(packet, &summary_deltas) {
                return Ok(summary_deltas);
            }
            if let Some(detail_key) = detail_key {
                let detail_deltas = self
                    .market_store
                    .get_json::<Vec<MarketFeatureDelta>>(detail_key)
                    .await
                    .unwrap_or_default();
                if !detail_deltas.is_empty() {
                    return Ok(detail_deltas);
                }
            }
            return Ok(summary_deltas);
        }
        if let Some(key) = detail_key {
            return Ok(self
                .market_store
                .get_json::<Vec<MarketFeatureDelta>>(key)
                .await
                .unwrap_or_default());
        }
        Ok(Vec::new())
    }
}
