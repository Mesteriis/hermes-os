#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

info "Removing build artifacts, temporary files, and logs"
rm -rf "$REPO_ROOT/target"
rm -rf "$REPO_ROOT/frontend/src-tauri/target"
rm -rf "$REPO_ROOT/frontend/node_modules/.vite"
rm -rf "$REPO_ROOT/frontend/dist"
rm -rf "$LOG_ROOT"
rm -rf "$REPO_ROOT/tmp/hermes"
find "$REPO_ROOT" -maxdepth 1 -type f -name '*.log' -delete
success "Clean completed without deleting database data"

