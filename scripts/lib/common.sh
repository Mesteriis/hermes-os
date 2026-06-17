#!/usr/bin/env bash

set -euo pipefail

COMMON_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPTS_DIR="$(cd "$COMMON_DIR/.." && pwd)"
REPO_ROOT="$(cd "$SCRIPTS_DIR/.." && pwd)"
LOG_ROOT="$REPO_ROOT/.local/dev-logs"
BACKUPS_ROOT="$REPO_ROOT/backups"
TOOLS_ROOT="$REPO_ROOT/.local/tools"
TOOLS_BIN="$TOOLS_ROOT/bin"

color_reset=$'\033[0m'
color_blue=$'\033[34m'
color_green=$'\033[32m'
color_yellow=$'\033[33m'
color_red=$'\033[31m'
color_cyan=$'\033[36m'
color_dim=$'\033[2m'

now_utc() {
	date -u +"%Y-%m-%dT%H:%M:%SZ"
}

today_utc() {
	date -u +"%Y-%m-%d"
}

timestamp_compact_utc() {
	date -u +"%Y%m%dT%H%M%SZ"
}

status_line() {
	local color="$1"
	local label="$2"
	local message="$3"
	printf '%b[%s]%b %s\n' "$color" "$label" "$color_reset" "$message"
}

info() {
	status_line "$color_blue" "info" "$1"
}

success() {
	status_line "$color_green" "ok" "$1"
}

warn() {
	status_line "$color_yellow" "warn" "$1"
}

error() {
	status_line "$color_red" "error" "$1" >&2
}

dim() {
	printf '%b%s%b\n' "$color_dim" "$1" "$color_reset"
}

ensure_dir() {
	mkdir -p "$1"
}

prepend_tools_bin_to_path() {
	case ":$PATH:" in
		*":$TOOLS_BIN:"*) ;;
		*) export PATH="$TOOLS_BIN:$PATH" ;;
	esac
}

ensure_command() {
	local command_name="$1"
	if ! command -v "$command_name" >/dev/null 2>&1; then
		error "Required command not found: $command_name"
		exit 1
	fi
}

ensure_one_of() {
	local first="$1"
	local second="$2"
	if command -v "$first" >/dev/null 2>&1; then
		printf '%s\n' "$first"
		return 0
	fi
	if command -v "$second" >/dev/null 2>&1; then
		printf '%s\n' "$second"
		return 0
	fi
	error "Required command not found: need $first or $second"
	exit 1
}

require_port_free() {
	local port="$1"
	local label="$2"
	if command -v lsof >/dev/null 2>&1 && lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
		error "$label port $port is already in use."
		exit 1
	fi
}

confirm_or_exit() {
	local prompt="$1"
	local expected="${2:-DELETE}"
	local answer
	printf '%s Type %s to continue: ' "$prompt" "$expected"
	read -r answer
	if [ "$answer" != "$expected" ]; then
		error "Confirmation did not match. Aborting."
		exit 1
	fi
}

json_escape() {
	local value="$1"
	value=${value//\\/\\\\}
	value=${value//\"/\\\"}
	value=${value//$'\t'/\\t}
	value=${value//$'\r'/\\r}
	printf '%s' "$value"
}

emit_json_log() {
	local file_path="$1"
	local service="$2"
	local pid="$3"
	local level="$4"
	local flow_id="$5"
	local message="$6"
	printf '{"timestamp":"%s","service":"%s","pid":%s,"level":"%s","flow_id":"%s","message":"%s"}\n' \
		"$(now_utc)" \
		"$(json_escape "$service")" \
		"$pid" \
		"$(json_escape "$level")" \
		"$(json_escape "$flow_id")" \
		"$(json_escape "$message")" >>"$file_path"
}

emit_live_log() {
	local file_path="$1"
	local service="$2"
	local pid="$3"
	local level="$4"
	local flow_id="$5"
	local message="$6"
	printf '[%s] service=%s pid=%s level=%s flow_id=%s %s\n' \
		"$(now_utc)" \
		"$service" \
		"$pid" \
		"$level" \
		"$flow_id" \
		"$message" >>"$file_path"
}

stream_service_pipe() {
	local pipe_path="$1"
	local service="$2"
	local pid="$3"
	local level="$4"
	local flow_id="$5"
	local color="$6"
	local log_file="$7"
	local live_log_file="$8"
	local line
	while IFS= read -r line || [ -n "$line" ]; do
		emit_json_log "$log_file" "$service" "$pid" "$level" "$flow_id" "$line"
		emit_live_log "$live_log_file" "$service" "$pid" "$level" "$flow_id" "$line"
		printf '%b[%s]%b %s\n' "$color" "$service" "$color_reset" "$line"
	done <"$pipe_path"
}

wait_for_http() {
	local url="$1"
	local label="$2"
	local attempts="${3:-60}"
	local sleep_seconds="${4:-1}"
	local index=1
	while [ "$index" -le "$attempts" ]; do
		if curl --silent --show-error --fail "$url" >/dev/null 2>&1; then
			return 0
		fi
		sleep "$sleep_seconds"
		index=$((index + 1))
	done
	error "$label did not become ready: $url"
	return 1
}
