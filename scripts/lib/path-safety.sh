# shellcheck shell=bash

strip_trailing_slashes() {
  local path="$1"
  while [[ "$path" != "/" && "$path" == */ ]]; do
    path="${path%/}"
  done
  printf '%s\n' "$path"
}

is_allowed_system_symlink_segment() {
  case "$1" in
    /etc | /tmp | /var) return 0 ;;
    *) return 1 ;;
  esac
}

require_unambiguous_path_segments() {
  local name="$1"
  local path
  local current=""
  local segment
  local path_segments=()

  path="$(strip_trailing_slashes "$2")"
  if [[ "$path" == *"//"* || "$path" == *"/./"* || "$path" == *"/../"* || "$path" == */. || "$path" == */.. ]]; then
    echo "$name must not contain empty, current-directory, or parent-directory path segments" >&2
    exit 1
  fi

  IFS='/' read -r -a path_segments <<< "${path#/}"
  for segment in "${path_segments[@]}"; do
    [[ -z "$segment" ]] && continue
    current="$current/$segment"
    if [[ -L "$current" ]]; then
      if is_allowed_system_symlink_segment "$current"; then
        continue
      fi
      echo "$name path segment must not be a symlink: $current" >&2
      exit 1
    fi
  done
}

require_absolute_dir_path() {
  local name="$1"
  local path="$2"
  local parent_dir
  if [[ -z "$path" || "$path" != /* ]]; then
    echo "$name must be an absolute directory path" >&2
    exit 1
  fi
  path="$(strip_trailing_slashes "$path")"
  require_unambiguous_path_segments "$name" "$path"
  parent_dir="${path%/*}"
  if [[ -z "$parent_dir" ]]; then
    parent_dir="/"
  fi
  if [[ ! -d "$parent_dir" ]]; then
    echo "$name parent directory does not exist: $parent_dir" >&2
    exit 1
  fi
  if [[ -L "$path" ]]; then
    echo "$name must not be a symlink: $path" >&2
    exit 1
  fi
  if [[ -e "$path" && ! -d "$path" ]]; then
    echo "$name must be a directory path: $path" >&2
    exit 1
  fi
}

require_safe_output_file_path() {
  local name="$1"
  local path="$2"
  local parent_dir
  if [[ -z "$path" || "$path" != /* ]]; then
    echo "$name must be an absolute file path" >&2
    exit 1
  fi
  path="$(strip_trailing_slashes "$path")"
  require_unambiguous_path_segments "$name" "$path"
  parent_dir="${path%/*}"
  if [[ -z "$parent_dir" ]]; then
    parent_dir="/"
  fi
  if [[ ! -d "$parent_dir" ]]; then
    echo "$name parent directory does not exist: $parent_dir" >&2
    exit 1
  fi
  if [[ -L "$path" ]]; then
    echo "$name must not be a symlink: $path" >&2
    exit 1
  fi
  if [[ -e "$path" && ! -f "$path" ]]; then
    echo "$name must be a regular file path: $path" >&2
    exit 1
  fi
}

require_absolute_output_path() {
  local name="$1"
  local path="$2"
  if [[ -z "$path" ]]; then
    return
  fi
  require_safe_output_file_path "$name" "$path"
}
