#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=./lib/env.sh
source "$SCRIPT_DIR/lib/env.sh"
# shellcheck source=./lib/resources.sh
source "$SCRIPT_DIR/lib/resources.sh"

load_hermes_env
ensure_frontend_dependencies
ensure_command cargo
ensure_command node
ensure_command pnpm

backend_target_dir="${CARGO_TARGET_DIR:-$CARGO_BUILD_TARGET_DIR}"

info "Building backend release binary"
CARGO_TARGET_DIR="$backend_target_dir" \
	cargo build --manifest-path "$REPO_ROOT/backend/Cargo.toml" --bin hermes-hub-backend --release

info "Building frontend release assets"
(
	cd "$REPO_ROOT/frontend"
	pnpm build
)

info "Preparing bundled desktop resources"
prepare_google_oauth_resource
prepare_tdlib_macos
CARGO_TARGET_DIR="$backend_target_dir" prepare_backend_sidecar_macos

info "Building Tauri release artifacts"
(
	cd "$REPO_ROOT/frontend"
	pnpm tauri build
)

success "Release build completed"
