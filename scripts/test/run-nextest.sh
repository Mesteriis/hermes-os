#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

source "${REPO_ROOT}/scripts/lib/rust-tooling.sh"

PROFILE="${1:-default}"
shift || true

require_cargo_subcommand "nextest" "cargo install --locked cargo-nextest"

CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${REPO_ROOT}/target/validate-test}"
export CARGO_TARGET_DIR
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"

cd "${REPO_ROOT}"

cargo run --manifest-path crates/testkit/Cargo.toml --bin hermes_test_session -- \
	cargo nextest run \
		--manifest-path backend/Cargo.toml \
		--profile "${PROFILE}" \
		--test-threads "${HERMES_NEXTEST_JOBS:-4}" \
		"$@"
