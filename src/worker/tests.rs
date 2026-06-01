use super::*;
use crate::model::{
    MarketFeatureDelta, MarketFeatureDeltaSummary, STRUCTURED_PACKET_SCHEMA_VERSION,
    STRUCTURED_POINTER_SCHEMA_VERSION, StructuredIntelPacket,
};
use crate::nats::{S3ObjectPointer, StructuredPointer};
use serde_json::json;
use std::path::Path;

#[path = "tests/args.rs"]
mod args;
#[path = "tests/content.rs"]
mod content;
#[path = "tests/fixtures.rs"]
mod fixtures;
#[path = "tests/market.rs"]
mod market;
#[path = "tests/repair.rs"]
mod repair;
#[path = "tests/revision.rs"]
mod revision;

use fixtures::*;
