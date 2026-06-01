# shellcheck shell=bash

send_pipeline_alert() {
  local priority="$1"
  local title="$2"
  local text="$3"
  require_alert_priority "pipeline alert priority" "$priority"
  if [[ -z "$PIPELINE_ALERT_S3_BUCKET" ]]; then
    die "NANGMAN_PIPELINE_ALERT_S3_BUCKET or INTEL_CANDIDATE_PIPELINE_ALERT_S3_BUCKET is required"
  fi
  require_s3_bucket_name \
    "NANGMAN_PIPELINE_ALERT_S3_BUCKET or INTEL_CANDIDATE_PIPELINE_ALERT_S3_BUCKET" \
    "$PIPELINE_ALERT_S3_BUCKET"
  require_s3_object_prefix \
    "NANGMAN_PIPELINE_ALERT_S3_PREFIX or INTEL_CANDIDATE_PIPELINE_ALERT_S3_PREFIX" \
    "$PIPELINE_ALERT_S3_PREFIX"
  local now_ms dt hour event_id key payload_file
  now_ms="$(date -u +%s000)"
  dt="$(date -u +%Y-%m-%d)"
  hour="$(date -u +%H)"
  event_id="pipeline_alert_intel_candidate_${now_ms}_$$"
  key="${PIPELINE_ALERT_S3_PREFIX%/}/dt=${dt}/hour=${hour}/app=${APP_NAME}/priority=${priority}/${event_id}.json"
  payload_file="$(mktemp)"
  local payload
  payload="$(jq -nc \
    --arg event_id "$event_id" \
    --arg dedupe_key "${APP_NAME}:${priority}:${title}" \
    --arg app "$APP_NAME" \
    --arg env "$ALERT_ENV" \
    --arg priority "$priority" \
    --arg title "$title" \
    --arg rendered_text "$text" \
    --argjson created_at_ms "$now_ms" \
    '{schema_version:"pipeline_alert_event_v1",event_id:$event_id,dedupe_key:$dedupe_key,app:$app,environment:$env,priority:$priority,title:$title,conclusion:"Runtime wrapper emitted a pipeline alert.",rendered_text:$rendered_text,current_state:["pre-rendered runtime alert"],reasons:[],next_actions:[],safety:["paper/live/order execution unchanged"],created_at_ms:$created_at_ms}')"
  printf '%s\n' "$payload" > "$payload_file"
  local upload_status=0
  aws s3api put-object \
    --region "$AWS_REGION" \
    --bucket "$PIPELINE_ALERT_S3_BUCKET" \
    --key "$key" \
    --body "$payload_file" \
    --content-type application/json >/dev/null || upload_status=$?
  remove_temp_file "$payload_file"
  return "$upload_status"
}
