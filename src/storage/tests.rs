use super::*;

#[test]
fn rejects_public_doc_bucket_placeholder() {
    let config = ObjectStoreConfig {
        bucket: "nangman-crypto-dev-intel-candidate-<account-suffix>".to_owned(),
        region: "ap-northeast-2".to_owned(),
        profile: None,
        access_key_id: None,
        secret_access_key: None,
    };
    let err = validate_config(&config).unwrap_err();
    assert!(err.to_string().contains("public-doc placeholder"));
}

#[test]
fn rejects_invalid_bucket_names_before_connecting() {
    for bucket in [
        "",
        "ab",
        &"a".repeat(64),
        "nangman_crypto_bucket",
        "NangmanCryptoBucket",
        "-nangman-crypto",
        "nangman-crypto-",
        "nangman..crypto",
        "nangman.-crypto",
        "nangman-.crypto",
        "192.168.5.4",
        "xn--nangman-crypto",
        "sthree-nangman-crypto",
        "amzn-s3-demo-nangman-crypto",
        "nangman-crypto-s3alias",
        "nangman-crypto--ol-s3",
        "nangman-crypto.mrap",
        "nangman-crypto--x-s3",
        "nangman-crypto--table-s3",
    ] {
        assert!(
            validate_bucket_name(bucket, "test bucket").is_err(),
            "expected bucket to be rejected: {bucket:?}"
        );
    }
}

#[test]
fn accepts_valid_bucket_names() {
    for bucket in [
        "nangman-crypto-dev-intel-candidate-123456",
        "nangman.crypto.dev",
        "a12",
    ] {
        validate_bucket_name(bucket, "test bucket").unwrap();
    }
}

#[test]
fn accepts_safe_object_keys_and_prefixes() {
    validate_object_key(
        "candidate-screening-event/schema=intel_candidate_screening_event_v1/dt=2026-05-31/hour=05/part-000001.jsonl",
        "candidate key",
    )
    .unwrap();
    validate_object_prefix(
        "structured-intel-packet/schema=structured_intel_packet_v1/",
        "structured prefix",
    )
    .unwrap();
}

#[test]
fn rejects_unsafe_object_keys() {
    for key in [
        "",
        " s3/key.json",
        "/absolute/key.json",
        "s3://bucket/key.json",
        "prefix/../key.json",
        "prefix//key.json",
        "prefix/key.json?version=1",
        "prefix/key.json#fragment",
        "prefix\\key.json",
        "prefix/key with space.json",
    ] {
        assert!(
            validate_object_key(key, "candidate key").is_err(),
            "expected key to be rejected: {key:?}"
        );
    }
}

#[test]
fn rejects_unsafe_object_prefixes() {
    for prefix in [
        "",
        " structured-intel-packet/",
        "/structured-intel-packet/",
        "s3://bucket/structured-intel-packet/",
        "structured-intel-packet/../",
        "structured-intel-packet//",
        "structured-intel-packet/?scan=true",
        "structured-intel-packet/#fragment",
        "structured-intel-packet\\",
        "structured intel packet/",
    ] {
        assert!(
            validate_object_prefix(prefix, "structured prefix").is_err(),
            "expected prefix to be rejected: {prefix:?}"
        );
    }
}

#[test]
fn rejects_object_key_over_s3_limit() {
    let key = "a".repeat(1025);
    let err = validate_object_key(&key, "candidate key").unwrap_err();
    assert!(err.to_string().contains("1024 bytes"));
}
