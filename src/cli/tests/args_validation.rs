use super::*;

#[test]
fn rejects_cli_paths_with_relative_components() {
    for (flag, value) in [
        ("--input-file", "/tmp/../structured.json"),
        ("--policy-file", "/tmp/./policy.json"),
        ("--output-dir", "/tmp/../intel-candidate-out"),
    ] {
        let error = parse_args(
            [
                "--input-file",
                "/tmp/structured.json",
                "--policy-file",
                test_policy_path().to_str().expect("utf8 path"),
                flag,
                value,
            ]
            .into_iter()
            .map(ToOwned::to_owned),
        )
        .unwrap_err()
        .to_string();

        assert!(
            error.contains(flag),
            "expected {flag} in error for {value:?}, got {error}"
        );
        assert!(
            error.contains("relative path components"),
            "expected relative path error for {value:?}, got {error}"
        );
    }
}
