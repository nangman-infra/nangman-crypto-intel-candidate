#!/usr/bin/env bash

write_source_gap_input_manifest() {
  require_safe_output_file_path "source gap input manifest output" "$manifest_output"
  jq -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg source_dt "$SOURCE_DT" \
    --arg output_dir "$OUTPUT_DIR" \
    --arg task_family_revision "$task_family_revision" \
    --arg structured_dir "$structured_dir" \
    --arg screening_dir "$screening_dir" \
    --arg hypothesis_dir "$hypothesis_dir" \
    --arg evidence_dir "$evidence_dir" \
    --argjson structured_files "$structured_files" \
    --argjson screening_files "$screening_files" \
    --argjson hypothesis_files "$hypothesis_files" \
    --argjson evidence_files "$evidence_files" \
    -f "$SCRIPT_DIR/jq/source-gap-input-export-manifest.jq" > "$manifest_output"
}

print_source_gap_export_summary() {
  {
    echo "source_gap_input_manifest=$manifest_output"
    echo "structured_packet_paths=$structured_dir"
    echo "screening_event_paths=$screening_dir"
    echo "hypothesis_state_paths=$hypothesis_dir"
    echo "evidence_bundle_paths=$evidence_dir"
    echo "structured_files=$structured_files"
    echo "screening_files=$screening_files"
    echo "hypothesis_files=$hypothesis_files"
    echo "evidence_files=$evidence_files"
    echo "source gap input export completed"
  } >&2
}
