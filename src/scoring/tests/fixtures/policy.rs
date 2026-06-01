use crate::policy::{ScoringPolicy, load_policy};
use std::path::Path;

pub(in crate::scoring::tests) fn policy() -> ScoringPolicy {
    load_policy(&Path::new(env!("CARGO_MANIFEST_DIR")).join("policies/scoring-policy.v1.json"))
        .expect("default scoring policy loads")
}
