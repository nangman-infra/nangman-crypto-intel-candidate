use sha2::{Digest, Sha256};

pub fn sha256_hex(bytes: impl AsRef<[u8]>) -> String {
    let digest = Sha256::digest(bytes.as_ref());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn stable_id(prefix: &str, parts: &[&str]) -> String {
    let joined = parts.join("|");
    format!(
        "{prefix}_{}",
        sha256_hex(joined.as_bytes()).get(..24).unwrap_or("")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_id_is_deterministic() {
        assert_eq!(
            stable_id("cand", &["a", "b"]),
            stable_id("cand", &["a", "b"])
        );
        assert_ne!(
            stable_id("cand", &["a", "b"]),
            stable_id("cand", &["a", "c"])
        );
    }
}
