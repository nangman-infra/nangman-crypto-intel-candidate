use super::*;

#[test]
fn worker_defaults_follow_candidate_contract_names() {
    let args = WorkerArgs::default();
    assert_eq!(args.nats.input_stream, "STRUCTURED_INTEL");
    assert_eq!(args.nats.output_stream, "INTEL_CANDIDATE");
    assert_eq!(
        args.nats.hypothesis_state_subject,
        "intel_candidate_hypothesis_state.created"
    );
    assert_eq!(args.output_store.bucket, DEFAULT_OUTPUT_BUCKET);
    assert_eq!(
        args.policy_file,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("policies/scoring-policy.v1.json")
    );
}

#[test]
fn parses_required_nats_url() {
    let args = WorkerArgs::parse(
        [
            "--nats-url",
            "nats://127.0.0.1:4222",
            "--input-s3-bucket",
            "test-structured-l1",
            "--output-s3-bucket",
            "test-candidate",
            "--market-l1-s3-bucket",
            "test-market-l1",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .unwrap()
    .unwrap();
    assert_eq!(args.nats.url, "nats://127.0.0.1:4222");
}

#[test]
fn accepts_nats_url_from_environment() {
    let args = super::super::args::parse_with_nats_url_env(
        [
            "--input-s3-bucket",
            "test-structured-l1",
            "--output-s3-bucket",
            "test-candidate",
            "--market-l1-s3-bucket",
            "test-market-l1",
        ]
        .into_iter()
        .map(str::to_owned),
        Some("nats://127.0.0.1:4222"),
    )
    .unwrap()
    .unwrap();
    assert_eq!(args.nats.url, "nats://127.0.0.1:4222");
}

#[test]
fn rejects_invalid_nats_urls_before_connecting() {
    for (flag_value, env_value, expected) in [
        (Some("http://127.0.0.1:4222"), None, "must start with"),
        (Some("nats://127.0.0.1:4222 dev"), None, "must not contain"),
        (Some("nats://"), None, "include a server"),
        (None, Some("http://127.0.0.1:4222"), "must start with"),
    ] {
        let mut raw = vec![
            "--input-s3-bucket".to_owned(),
            "test-structured-l1".to_owned(),
            "--output-s3-bucket".to_owned(),
            "test-candidate".to_owned(),
            "--market-l1-s3-bucket".to_owned(),
            "test-market-l1".to_owned(),
        ];
        if let Some(value) = flag_value {
            raw.splice(0..0, ["--nats-url".to_owned(), value.to_owned()]);
        }

        let err = super::super::args::parse_with_nats_url_env(raw.into_iter(), env_value)
            .unwrap_err()
            .to_string();

        assert!(
            err.contains(expected),
            "expected {expected:?} for flag={flag_value:?} env={env_value:?}, got {err:?}"
        );
    }
}

#[test]
fn rejects_public_doc_bucket_placeholder() {
    let err = WorkerArgs::parse(
        [
            "--nats-url",
            "nats://127.0.0.1:4222",
            "--input-s3-bucket",
            DEFAULT_INPUT_BUCKET,
            "--output-s3-bucket",
            "test-candidate",
            "--market-l1-s3-bucket",
            "test-market-l1",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .unwrap_err();

    assert!(err.to_string().contains("--input-s3-bucket"));
    assert!(err.to_string().contains("public-doc placeholder"));
}

#[test]
fn rejects_invalid_bucket_names_before_connecting() {
    let err = WorkerArgs::parse(
        [
            "--nats-url",
            "nats://127.0.0.1:4222",
            "--input-s3-bucket",
            "test_structured_l1",
            "--output-s3-bucket",
            "test-candidate",
            "--market-l1-s3-bucket",
            "test-market-l1",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .unwrap_err();

    assert!(err.to_string().contains("--input-s3-bucket"));
    assert!(err.to_string().contains("lowercase letters"));
}

#[test]
fn rejects_ambiguous_policy_file_path() {
    let err = WorkerArgs::parse(
        [
            "--nats-url",
            "nats://127.0.0.1:4222",
            "--input-s3-bucket",
            "test-structured-l1",
            "--output-s3-bucket",
            "test-candidate",
            "--market-l1-s3-bucket",
            "test-market-l1",
            "--policy-file",
            "/tmp/../policy.json",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .unwrap_err();

    assert!(err.to_string().contains("--policy-file"));
    assert!(err.to_string().contains("relative path components"));
}
