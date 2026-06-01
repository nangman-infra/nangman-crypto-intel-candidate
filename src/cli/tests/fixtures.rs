#[path = "fixtures/files.rs"]
mod files;
#[path = "fixtures/market.rs"]
mod market;
#[path = "fixtures/packet.rs"]
mod packet;
#[path = "fixtures/universe.rs"]
mod universe;

pub(in crate::cli::tests) use files::{test_policy_path, test_root, write_json};
pub(in crate::cli::tests) use market::{market_feature_delta_json, market_regime_context_json};
pub(in crate::cli::tests) use packet::packet_json;
pub(in crate::cli::tests) use universe::universe_json;
