pub fn print_help() {
    println!("{}", help_text());
}

pub(super) fn help_text() -> &'static str {
    r#"intel-candidate-app
Usage:
  intel-candidate-app \
    --input-file /Volumes/WD/Developments/nangman-crypto/data/examples/structured-intel-packets.jsonl \
    --policy-file /Volumes/WD/Developments/nangman-crypto/apps/intel-candidate-app/policies/scoring-policy.v1.json \
    --universe-snapshot-file /Volumes/WD/Developments/nangman-crypto/data/examples/symbol-universe-snapshot.json \
    --market-feature-delta-file /Volumes/WD/Developments/nangman-crypto/data/examples/market-feature-delta.json \
    --market-regime-context-file /Volumes/WD/Developments/nangman-crypto/data/examples/market-regime-context.json \
    --output-dir /Volumes/WD/Developments/nangman-crypto/data/spool/intel-candidate

Without --output-dir, the app prints screening events and evidence bundles to stdout.
This app does not run external adapters, does not publish orders, and does not emit buy/sell/long/short decisions."#
}
