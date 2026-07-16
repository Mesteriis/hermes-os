NODE ?= node
RUST_TOOLCHAIN ?= 1.97.0
MACOS_KERNEL_TARGET ?= aarch64-apple-darwin
TAURI_SIDECAR_DIR ?= ../frontend/src-tauri/binaries
TARGET ?=
MANIFEST ?=
COMPOSE ?=
SYSTEMD_UNIT ?=

.PHONY: architecture-check architecture-policy-check cargo-boundaries-check test-architecture validate test-kernel-recovery check-target ci verify-linux-release render-linux-release-compose render-linux-systemd-unit verify-macos-release package-kernel-macos development-platform-up development-platform-down development-platform-status development-platform-endpoints development-platform-smoke development-runtime-check

architecture-policy-check:
	$(NODE) scripts/check-architecture-policy.mjs

cargo-boundaries-check:
	$(NODE) scripts/check-cargo-boundaries.mjs

test-architecture:
	$(NODE) --test tests/architecture/*.test.mjs

architecture-check: architecture-policy-check cargo-boundaries-check

validate: architecture-check test-architecture

test-kernel-recovery:
	cargo +$(RUST_TOOLCHAIN) test --locked -p hermes-kernel-recovery-testkit
	$(NODE) --test tests/lifecycle/kernel-recovery.test.mjs

check-target:
	@test -n "$(TARGET)" || (echo "TARGET is required" >&2; exit 2)
	cargo +$(RUST_TOOLCHAIN) check --locked --workspace --target $(TARGET)

ci: validate test-kernel-recovery check-target

verify-linux-release:
	@test -n "$(MANIFEST)" || (echo "MANIFEST is required" >&2; exit 2)
	$(NODE) scripts/verify-linux-release-manifest.mjs $(MANIFEST)

render-linux-release-compose:
	@test -n "$(MANIFEST)" || (echo "MANIFEST is required" >&2; exit 2)
	@test -n "$(COMPOSE)" || (echo "COMPOSE is required" >&2; exit 2)
	$(NODE) scripts/verify-linux-release-manifest.mjs $(MANIFEST) --write-compose $(COMPOSE)

render-linux-systemd-unit:
	@test -n "$(COMPOSE)" || (echo "COMPOSE is required" >&2; exit 2)
	@test -n "$(SYSTEMD_UNIT)" || (echo "SYSTEMD_UNIT is required" >&2; exit 2)
	$(NODE) scripts/render-linux-systemd-unit.mjs $(COMPOSE) $(SYSTEMD_UNIT)

verify-macos-release:
	@test -n "$(MANIFEST)" || (echo "MANIFEST is required" >&2; exit 2)
	$(NODE) scripts/verify-macos-release-manifest.mjs $(MANIFEST)

package-kernel-macos:
	cargo +$(RUST_TOOLCHAIN) build --locked --manifest-path Cargo.toml --package hermes-kernel --target $(MACOS_KERNEL_TARGET) --release
	install -m 755 target/$(MACOS_KERNEL_TARGET)/release/hermes-kernel $(TAURI_SIDECAR_DIR)/hermes-kernel-$(MACOS_KERNEL_TARGET)

development-platform-up:
	docker compose -f development/compose.yaml up --detach --wait

development-platform-down:
	docker compose -f development/compose.yaml down

development-platform-status:
	docker compose -f development/compose.yaml ps

development-platform-endpoints:
	$(NODE) scripts/print-development-platform-endpoints.mjs

development-platform-smoke:
	$(NODE) scripts/smoke-development-platform.mjs

development-runtime-check:
	cargo +$(RUST_TOOLCHAIN) check --locked --package hermes-development-platform-runtime
