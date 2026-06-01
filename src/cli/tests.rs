use super::*;
use serde_json::json;
use std::fs;

#[path = "tests/args_validation.rs"]
mod args_validation;
#[path = "tests/artifact_admission.rs"]
mod artifact_admission;
#[path = "tests/fixtures.rs"]
mod fixtures;

use fixtures::*;
