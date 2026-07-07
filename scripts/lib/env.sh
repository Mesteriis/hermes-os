#!/usr/bin/env bash

set -euo pipefail

# shellcheck source=./common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

DOCKER_ENV_FILE="$REPO_ROOT/docker/.env"
DOCKER_ENV_TEMPLATE="$REPO_ROOT/docker/.env.example"
LOCAL_ENV_FILE="$REPO_ROOT/.env"

provider_runtime_env_names() {
	printf '%s\n' \
		HERMES_TDJSON_PATH \
		HERMES_TELEGRAM_API_ID \
		HERMES_TELEGRAM_API_HASH \
		HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH \
		HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE \
		HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_JSON \
		HERMES_GOOGLE_OAUTH_CLIENT_ID \
		HERMES_GOOGLE_OAUTH_CLIENT_SECRET
}

ensure_docker_env_file() {
	if [ ! -f "$DOCKER_ENV_FILE" ]; then
		cp "$DOCKER_ENV_TEMPLATE" "$DOCKER_ENV_FILE"
		warn "Created docker/.env from docker/.env.example. Review local secrets before continuing."
	fi
}

source_env_file() {
	local env_file="$1"
	if [ -z "$env_file" ]; then
		return 0
	fi
	if [ ! -f "$env_file" ]; then
		error "Hermes env file was not found: $env_file"
		exit 1
	fi
	set -a
	# shellcheck disable=SC1090
	. "$env_file"
	set +a
}

source_env_file_if_exists() {
	local env_file="$1"
	if [ -f "$env_file" ]; then
		source_env_file "$env_file"
	fi
}

load_hermes_env() {
	prepend_tools_bin_to_path
	ensure_docker_env_file
	source_env_file "$DOCKER_ENV_FILE"
	source_env_file_if_exists "$LOCAL_ENV_FILE"
	import_launchctl_env_name HERMES_ENV_FILE
	source_env_file "${HERMES_ENV_FILE:-}"

	: "${HERMES_POSTGRES_DB:=hermes_hub}"
	: "${HERMES_POSTGRES_USER:=hermes}"
	: "${HERMES_POSTGRES_PASSWORD:=change-me-local-dev-only}"
	: "${HERMES_POSTGRES_BIND:=127.0.0.1}"
	: "${HERMES_POSTGRES_PORT:=30432}"
	: "${HERMES_NATS_BIND:=127.0.0.1}"
	: "${HERMES_NATS_PORT:=34222}"
	: "${HERMES_NATS_MONITOR_BIND:=127.0.0.1}"
	: "${HERMES_NATS_MONITOR_PORT:=38222}"
	: "${HERMES_BACKEND_BIND:=127.0.0.1}"
	: "${HERMES_BACKEND_PORT:=8080}"
	: "${HERMES_BACKEND_STARTUP_ATTEMPTS:=300}"
	: "${HERMES_BACKEND_STARTUP_SLEEP_SECONDS:=1}"
	: "${HERMES_FRONTEND_BIND:=127.0.0.1}"
	: "${HERMES_FRONTEND_PORT:=5174}"
	: "${HERMES_FRONTEND_STARTUP_ATTEMPTS:=120}"
	: "${HERMES_FRONTEND_STARTUP_SLEEP_SECONDS:=1}"
	: "${HERMES_LOCAL_API_SECRET:=change-me-local-api-secret}"
	: "${HERMES_DEV_MODE:=true}"
	: "${HERMES_HOST_VAULT_HOME:=$HOME/.hermes/vault}"
	: "${HERMES_SECRET_VAULT_KEY:=change-me-local-secret-vault-key}"
	: "${HERMES_OLLAMA_BASE_URL:=http://127.0.0.1:11434}"
	: "${HERMES_OLLAMA_CHAT_MODEL:=qwen3:4b}"
	: "${HERMES_OLLAMA_EMBED_MODEL:=qwen3-embedding:4b}"
	: "${HERMES_OLLAMA_TIMEOUT_SECONDS:=120}"

	HERMES_VAULT_HOME="$HERMES_HOST_VAULT_HOME"
	HERMES_DEV_KEY_PATH="$HERMES_HOST_VAULT_HOME/dev/master.key"
	DATABASE_URL="postgres://${HERMES_POSTGRES_USER}:${HERMES_POSTGRES_PASSWORD}@127.0.0.1:${HERMES_POSTGRES_PORT}/${HERMES_POSTGRES_DB}"
	HERMES_NATS_SERVER_URL="${HERMES_NATS_SERVER_URL:-nats://127.0.0.1:${HERMES_NATS_PORT:-34222}}"

	export HERMES_VAULT_HOME
	export HERMES_DEV_KEY_PATH
	export DATABASE_URL
	export HERMES_NATS_BIND
	export HERMES_NATS_PORT
	export HERMES_NATS_MONITOR_BIND
	export HERMES_NATS_MONITOR_PORT
	export HERMES_NATS_SERVER_URL
	import_launchctl_provider_runtime_env
	export_provider_runtime_env
}

import_launchctl_env_name() {
	local name="$1"
	if [ "$(uname -s 2>/dev/null || true)" != "Darwin" ]; then
		return 0
	fi
	if ! command -v launchctl >/dev/null 2>&1; then
		return 0
	fi
	if [ "${!name+x}" = "x" ]; then
		return 0
	fi

	local value uid
	value="$(launchctl getenv "$name" 2>/dev/null || true)"
	if [ -z "$value" ]; then
		uid="$(id -u)"
		value="$(launchctl asuser "$uid" getenv "$name" 2>/dev/null || true)"
	fi
	if [ -n "$value" ]; then
		export "$name=$value"
	fi
}

import_launchctl_provider_runtime_env() {
	local name
	while IFS= read -r name; do
		import_launchctl_env_name "$name"
	done < <(provider_runtime_env_names)
}

export_provider_runtime_env() {
	local name
	while IFS= read -r name; do
		if [ "${!name+x}" = "x" ]; then
			export "$name"
		fi
	done < <(provider_runtime_env_names)
}

prepare_bundled_provider_runtime_env() {
	if [ -z "${HERMES_BUNDLED_TELEGRAM_API_ID:-}" ] && [ -n "${HERMES_TELEGRAM_API_ID:-}" ]; then
		HERMES_BUNDLED_TELEGRAM_API_ID="$HERMES_TELEGRAM_API_ID"
		export HERMES_BUNDLED_TELEGRAM_API_ID
	fi
	if [ -z "${HERMES_BUNDLED_TELEGRAM_API_HASH:-}" ] && [ -n "${HERMES_TELEGRAM_API_HASH:-}" ]; then
		HERMES_BUNDLED_TELEGRAM_API_HASH="$HERMES_TELEGRAM_API_HASH"
		export HERMES_BUNDLED_TELEGRAM_API_HASH
	fi
	if [ -z "${HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON:-}" ]; then
		if [ -n "${HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_JSON:-}" ]; then
			HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON="$HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_JSON"
			export HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON
		else
			local google_oauth_source
			google_oauth_source="${HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE:-${HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH:-}}"
			if [ -n "$google_oauth_source" ] && [ -f "$google_oauth_source" ]; then
				HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON="$(cat "$google_oauth_source")"
				export HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON
			fi
		fi
	fi
	if [ -z "${HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_ID:-}" ] && [ -n "${HERMES_GOOGLE_OAUTH_CLIENT_ID:-}" ]; then
		HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_ID="$HERMES_GOOGLE_OAUTH_CLIENT_ID"
		export HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_ID
	fi
	if [ -z "${HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_SECRET:-}" ] && [ -n "${HERMES_GOOGLE_OAUTH_CLIENT_SECRET:-}" ]; then
		HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_SECRET="$HERMES_GOOGLE_OAUTH_CLIENT_SECRET"
		export HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_SECRET
	fi
}

ensure_bacon_available() {
	prepend_tools_bin_to_path
	if command -v bacon >/dev/null 2>&1; then
		return 0
	fi

	ensure_command cargo
	ensure_dir "$TOOLS_ROOT"
	info "Installing local bacon into $TOOLS_ROOT"
	cargo install --locked --root "$TOOLS_ROOT" bacon
	prepend_tools_bin_to_path

	if ! command -v bacon >/dev/null 2>&1; then
		error "bacon installation completed but binary was not found in $TOOLS_BIN"
		exit 1
	fi
}

ensure_frontend_dependencies() {
	ensure_command pnpm
	if [ ! -d "$REPO_ROOT/frontend/node_modules" ] || [ ! -x "$REPO_ROOT/frontend/node_modules/.bin/tauri" ]; then
		info "Installing frontend dependencies"
		(
			cd "$REPO_ROOT/frontend"
			pnpm install --frozen-lockfile
		)
	fi
}
