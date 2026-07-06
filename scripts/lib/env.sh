#!/usr/bin/env bash

set -euo pipefail

# shellcheck source=./common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

DOCKER_ENV_FILE="$REPO_ROOT/docker/.env"
DOCKER_ENV_TEMPLATE="$REPO_ROOT/docker/.env.example"

ensure_docker_env_file() {
	if [ ! -f "$DOCKER_ENV_FILE" ]; then
		cp "$DOCKER_ENV_TEMPLATE" "$DOCKER_ENV_FILE"
		warn "Created docker/.env from docker/.env.example. Review local secrets before continuing."
	fi
}

load_hermes_env() {
	prepend_tools_bin_to_path
	ensure_docker_env_file
	set -a
	# shellcheck disable=SC1090
	. "$DOCKER_ENV_FILE"
	set +a

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
