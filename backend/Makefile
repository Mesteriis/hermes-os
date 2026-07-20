SHELL := /usr/bin/env bash

RUST_TOOLCHAIN ?= 1.97.0
DOCKER ?= docker
LINUX_CI_IMAGE ?= rust:1.97.0-bookworm
LINUX_CI_PLATFORM ?= linux/amd64
TARGET ?= $(shell rustc +$(RUST_TOOLCHAIN) -vV | sed -n 's/^host: //p')

.PHONY: build test dev docker tauri clean \
	architecture-policy-check srp-policy-check cargo-boundaries-check \
	test-architecture fmt-check clippy test-workspace test-integration \
	dependency-policy-check supply-chain-check check-target check-linux-container ci

build:
	cargo +$(RUST_TOOLCHAIN) build --locked --workspace
	cd ../frontend && pnpm build

test:
	node scripts/run-impacted-tests.mjs

dev: docker
	@set -euo pipefail; \
	data_dir="$${HERMES_DEV_DATA_DIR:-$$(cd .. && pwd)/.local/kernel-dev}"; \
	cleanup() { \
		status=$$?; \
		trap - EXIT INT TERM; \
		kill "$${backend_pid:-}" "$${frontend_pid:-}" 2>/dev/null || true; \
		wait "$${backend_pid:-}" "$${frontend_pid:-}" 2>/dev/null || true; \
		exit "$$status"; \
	}; \
	trap cleanup EXIT INT TERM; \
	(cargo +1.97.0 run --locked --package hermes-kernel -- --data-dir "$$data_dir" serve) & \
	backend_pid=$$!; \
	(cd ../frontend && pnpm dev --host "$${HERMES_DEV_FRONTEND_HOST:-0.0.0.0}") & \
	frontend_pid=$$!; \
	wait -n "$$backend_pid" "$$frontend_pid"

docker:
	docker compose -f development/compose.yaml up --detach --wait

tauri: build
	cd ../frontend && pnpm exec tauri build

# Removes only reproducible Hermes build and test output. It intentionally
# preserves runtime data, Control Store state, Docker volumes and releases.
clean:
	cargo clean
	rm -rf -- ../frontend/dist ../frontend/coverage ../frontend/test-results ../frontend/playwright-report ../frontend/storybook-static ../frontend/node_modules/.cache ../frontend/node_modules/.vite

architecture-policy-check:
	node scripts/check-architecture-policy.mjs

srp-policy-check:
	node scripts/check-srp-policy.mjs

cargo-boundaries-check:
	node scripts/check-cargo-boundaries.mjs

test-architecture:
	node --test tests/architecture/*.test.mjs

fmt-check:
	cargo +$(RUST_TOOLCHAIN) fmt --check

clippy:
	cargo +$(RUST_TOOLCHAIN) clippy --locked --workspace --all-targets -- -D warnings

test-workspace:
	cargo nextest run --locked --workspace --profile ci

test-integration:
	cargo nextest run --locked --workspace --profile integration

dependency-policy-check:
	cargo deny check bans licenses sources

supply-chain-check:
	cargo audit
	mkdir -p target/evidence
	syft dir:. --output cyclonedx-json > target/evidence/backend-sbom.cdx.json
	test -s target/evidence/backend-sbom.cdx.json

check-target:
	@test "$(TARGET)" = "aarch64-apple-darwin" -o "$(TARGET)" = "x86_64-unknown-linux-gnu" || { echo "unsupported CI target: $(TARGET)" >&2; exit 2; }
	@if test "$(TARGET)" = "x86_64-unknown-linux-gnu"; then $(MAKE) check-linux-container; fi

check-linux-container:
	$(DOCKER) run --rm --platform $(LINUX_CI_PLATFORM) --mount type=bind,src="$(CURDIR)",dst=/workspace,readonly --workdir /workspace -e CARGO_TARGET_DIR=/tmp/hermes-target $(LINUX_CI_IMAGE) cargo +$(RUST_TOOLCHAIN) check --locked --workspace --target x86_64-unknown-linux-gnu

ci: check-target architecture-policy-check srp-policy-check cargo-boundaries-check test-architecture fmt-check clippy test-workspace test-integration dependency-policy-check supply-chain-check
