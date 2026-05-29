use super::validation::{RESEARCH_COMPATIBLE_HORIZONS, validate_policy};
use super::*;

fn repo_policy() -> ScoringPolicy {
    load_policy(&Path::new(env!("CARGO_MANIFEST_DIR")).join("policies/scoring-policy.v1.json"))
        .expect("repo policy must load")
}

#[test]
fn packaged_policy_only_uses_research_compatible_horizons() {
    let policy = repo_policy();

    for horizons in policy.event_type_to_allowed_horizons.values() {
        assert!(!horizons.is_empty());
        for horizon in horizons {
            assert!(
                RESEARCH_COMPATIBLE_HORIZONS.contains(&horizon.as_str()),
                "{horizon} must be accepted by downstream research"
            );
        }
    }
}

#[test]
fn general_intel_tracks_short_mid_and_daily_horizons() {
    let policy = repo_policy();

    assert_eq!(policy.policy_version, "intel_candidate_scoring_v2");
    assert_eq!(
        policy.event_type_to_allowed_horizons.get("other"),
        Some(&vec!["1h".to_owned(), "4h".to_owned(), "24h".to_owned()])
    );
}

#[test]
fn policy_validation_rejects_horizon_beyond_research_contract() {
    let mut policy = repo_policy();
    policy
        .event_type_to_allowed_horizons
        .insert("project_notice".to_owned(), vec!["7d".to_owned()]);

    let error = validate_policy(&policy).expect_err("7d should be rejected");

    assert!(
        error
            .to_string()
            .contains("unsupported downstream research horizon 7d")
    );
}
