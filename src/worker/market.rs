use crate::hash::stable_id;
use crate::model::{MarketFeatureDelta, MarketFeatureDeltaSummary, StructuredIntelPacket};
use std::collections::BTreeSet;

pub(super) fn expand_market_feature_delta_summary(
    summary: MarketFeatureDeltaSummary,
) -> Vec<MarketFeatureDelta> {
    let mut deltas = Vec::new();
    for row in summary.rows {
        for metric in row.metrics {
            deltas.push(MarketFeatureDelta {
                schema_version: summary.schema_version.clone(),
                feature_delta_id: stable_id(
                    "market_delta_summary_metric",
                    &[
                        &summary.feature_delta_summary_id,
                        &row.venue,
                        &row.symbol_native,
                        &row.symbol_canonical,
                        &row.market_type,
                        &metric.metric_name,
                        &metric.window_start_ms.to_string(),
                        &metric.window_end_ms.to_string(),
                    ],
                ),
                l1_run_id: summary.l1_run_id.clone(),
                metric_name: metric.metric_name,
                venue: row.venue.clone(),
                symbol_native: row.symbol_native.clone(),
                symbol_canonical: row.symbol_canonical.clone(),
                market_type: row.market_type.clone(),
                value_now: metric.value_now,
                value_15m_ago: metric.value_15m_ago,
                value_1h_ago: metric.value_1h_ago,
                change_pct_15m: metric.change_pct_15m,
                change_pct_1h: metric.change_pct_1h,
                price_change_same_window: metric.price_change_same_window,
                volume_change_same_window: metric.volume_change_same_window,
                oi_price_divergence: metric.oi_price_divergence,
                window_start_ms: metric.window_start_ms,
                window_end_ms: metric.window_end_ms,
                known_as_of_ms: row.known_as_of_ms,
                quality_status: if metric.quality_status.trim().is_empty() {
                    row.quality_status.clone()
                } else {
                    metric.quality_status
                },
                missing_reasons: row.missing_reasons.clone(),
            });
        }
    }
    deltas
}

pub(super) fn market_feature_deltas_satisfy_packet(
    packet: &StructuredIntelPacket,
    deltas: &[MarketFeatureDelta],
) -> bool {
    let Some(decision_available_at_ms) = packet.decision_available_at_ms else {
        return false;
    };
    packet.normalized_symbols.iter().any(|symbol| {
        let candidates = canonical_symbol_candidates(symbol);
        deltas.iter().any(|delta| {
            candidates.contains(&delta.symbol_canonical.to_ascii_uppercase())
                && delta.window_end_ms <= decision_available_at_ms
                && delta.known_as_of_ms <= decision_available_at_ms
                && is_usable_market_artifact_quality(&delta.quality_status)
                && (delta.change_pct_1h.is_some() || delta.change_pct_15m.is_some())
        })
    })
}

pub(super) fn repair_raw_event_id(packet: &StructuredIntelPacket, key: &str) -> String {
    path_value(key, "raw_event_id")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| packet.packet_id.clone())
}

fn path_value(key: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    key.split('/')
        .find_map(|segment| segment.strip_prefix(&prefix))
        .map(ToOwned::to_owned)
}

fn canonical_symbol_candidates(symbol: &str) -> BTreeSet<String> {
    let upper = symbol.trim().to_ascii_uppercase();
    let mut values = BTreeSet::from([upper.clone()]);
    for suffix in ["USDT", "USDC", "USD", "BUSD", "BTC", "ETH"] {
        if upper.len() > suffix.len() && upper.ends_with(suffix) {
            values.insert(upper.trim_end_matches(suffix).to_owned());
        }
    }
    values
}

fn is_usable_market_artifact_quality(status: &str) -> bool {
    matches!(
        status.trim().to_ascii_lowercase().as_str(),
        "complete" | "partial" | "available"
    )
}
