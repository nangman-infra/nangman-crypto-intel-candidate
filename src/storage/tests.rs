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
