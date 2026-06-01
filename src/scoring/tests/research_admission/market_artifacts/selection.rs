use super::*;

#[test]
fn market_feature_delta_selection_prefers_latest_admissible_artifact() {
    let policy = policy();
    let universe = universe(true);
    let mut old_delta = market_feature_delta("price");
    old_delta.feature_delta_id = "old_delta".to_owned();
    old_delta.window_end_ms = 1_100;
    old_delta.known_as_of_ms = 1_150;

    let mut latest_delta = market_feature_delta("price");
    latest_delta.feature_delta_id = "latest_delta".to_owned();
    latest_delta.window_end_ms = 1_900;
    latest_delta.known_as_of_ms = 1_950;

    let mut future_delta = market_feature_delta("price");
    future_delta.feature_delta_id = "future_delta".to_owned();
    future_delta.window_end_ms = 7_200_001;
    future_delta.known_as_of_ms = 7_200_001;

    let regime_contexts = vec![market_regime_context()];
    let result = process_packet_with_artifacts(
        packet(),
        &policy,
        market_artifacts(
            &universe,
            &[old_delta, latest_delta, future_delta],
            &regime_contexts,
        ),
        7_200_000,
    );

    let bundle = result
        .evidence_bundle
        .expect("admissible market artifacts create bundle");
    assert!(bundle.selected_market_artifacts.iter().any(|artifact| {
        artifact.artifact_type == "market_feature_delta_summary"
            && artifact.artifact_id == "latest_delta"
            && artifact.window_end_ms == 1_900
            && artifact.known_as_of_ms == 1_950
    }));
    assert!(
        !bundle
            .selected_market_artifacts
            .iter()
            .any(|artifact| artifact.artifact_id == "future_delta")
    );
}
