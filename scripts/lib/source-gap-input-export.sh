#!/usr/bin/env bash

SOURCE_GAP_INPUT_EXPORT_LIB_DIR="${SOURCE_GAP_INPUT_EXPORT_LIB_DIR:-$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)}"

# shellcheck source=scripts/lib/source-gap-input-export-core.sh
source "$SOURCE_GAP_INPUT_EXPORT_LIB_DIR/source-gap-input-export-core.sh"
# shellcheck source=scripts/lib/source-gap-input-export-runtime.sh
source "$SOURCE_GAP_INPUT_EXPORT_LIB_DIR/source-gap-input-export-runtime.sh"
# shellcheck source=scripts/lib/source-gap-input-export-sync.sh
source "$SOURCE_GAP_INPUT_EXPORT_LIB_DIR/source-gap-input-export-sync.sh"
# shellcheck source=scripts/lib/source-gap-input-export-output.sh
source "$SOURCE_GAP_INPUT_EXPORT_LIB_DIR/source-gap-input-export-output.sh"
