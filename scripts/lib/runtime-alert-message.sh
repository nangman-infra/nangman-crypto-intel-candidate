# shellcheck shell=bash

message() {
  local priority="$1"
  local title="$2"
  local output_file="$3"
  local next_action="$4"
  local now_kst
  now_kst="$(TZ=Asia/Seoul date '+%Y-%m-%d %H:%M:%S KST')"
  cat <<EOF
[${priority}][intel-candidate-app] ${title}

결론:
Intel candidate runtime 상태를 확인했습니다.

현재 상태:
- env: ${ALERT_ENV}
- cluster: ${CLUSTER}
- service: ${SERVICE}
- app_dir: ${APP_DIR}

주요 원인:
$(tail -n 18 "$output_file" | sed 's/^/- /')

다음 행동:
${next_action}

안전 상태:
- 이 알림은 candidate 생성 상태 알림입니다.
- paper/live/order execution을 변경하지 않습니다.

발송 시각: ${now_kst}
EOF
}
