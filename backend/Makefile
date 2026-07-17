NODE ?= node
RUST_TOOLCHAIN ?= 1.97.0
MACOS_KERNEL_TARGET ?= aarch64-apple-darwin
DOCKER ?= docker
LINUX_CI_IMAGE ?= rust:1.97.0-bookworm
LINUX_CI_PLATFORM ?= linux/amd64
TAURI_SIDECAR_DIR ?= ../frontend/src-tauri/binaries
RELEASE_TRUST_ROOT ?=
SIGNED_DISTRIBUTION_MANIFEST ?=
DISTRIBUTION_BUNDLE ?=
RELEASE_DISTRIBUTION_INPUT ?=
RELEASE_SIGNING_KEY ?=
TARGET ?=
MANIFEST ?=
COSIGN_CERTIFICATE_IDENTITY ?=
COSIGN_OIDC_ISSUER ?=
MANIFEST_SIGNATURE ?=
MANIFEST_CERTIFICATE ?=
COMPOSE ?=
SYSTEMD_UNIT ?=

.PHONY: architecture-check architecture-policy-check cargo-boundaries-check test-architecture validate test-kernel-recovery test-vault test-development-simulations check-target check-linux-container ci verify-linux-release render-linux-release-compose render-linux-systemd-unit verify-macos-release generate-release-signing-key build-distribution-release package-kernel-macos development-platform-up development-platform-down development-platform-status development-platform-endpoints development-platform-smoke development-runtime-check

architecture-policy-check:
	$(NODE) scripts/check-architecture-policy.mjs

cargo-boundaries-check:
	$(NODE) scripts/check-cargo-boundaries.mjs

test-architecture:
	$(NODE) --test tests/architecture/*.test.mjs

architecture-check: architecture-policy-check cargo-boundaries-check

validate: architecture-check test-architecture
	node scripts/check-srp-policy.mjs

test-kernel-recovery:
	cargo +$(RUST_TOOLCHAIN) test --locked -p hermes-kernel-recovery-testkit
	$(NODE) --test tests/lifecycle/control-store-migrations.test.mjs
	$(NODE) --test tests/lifecycle/production-cli-separation.test.mjs
	$(NODE) --test tests/lifecycle/kernel-recovery.test.mjs
	$(NODE) --test tests/lifecycle/owner-control-ipc.test.mjs
	$(NODE) --test tests/lifecycle/module-registration-ipc.test.mjs
	$(NODE) --test tests/lifecycle/capability-router.test.mjs
	$(NODE) --test tests/lifecycle/external-runtime-session-ipc.test.mjs
	$(NODE) --test tests/lifecycle/settings-schema-admission.test.mjs
	$(NODE) --test tests/lifecycle/server-bootstrap-pairing.test.mjs

test-vault:
	cargo +$(RUST_TOOLCHAIN) test --locked -p hermes-vault-testkit

test-development-simulations:
	cargo +$(RUST_TOOLCHAIN) build --locked -p hermes-development-kernel-operator
	$(NODE) --test tests/lifecycle/development-pairing-state.test.mjs
	$(NODE) --test tests/lifecycle/development-module-operator-simulation.test.mjs

check-target:
	@test -n "$(TARGET)" || (echo "TARGET is required" >&2; exit 2)
	@if [ "$(TARGET)" = "x86_64-unknown-linux-gnu" ]; then \
		$(MAKE) check-linux-container; \
	else \
		cargo +$(RUST_TOOLCHAIN) check --locked --workspace --target $(TARGET); \
	fi

check-linux-container:
	$(DOCKER) run --rm --platform $(LINUX_CI_PLATFORM) \
		--mount type=bind,src="$(CURDIR)",dst=/workspace,readonly \
		--workdir /workspace \
		--env CARGO_TARGET_DIR=/tmp/hermes-target \
		$(LINUX_CI_IMAGE) \
		cargo +$(RUST_TOOLCHAIN) check --locked --workspace --target x86_64-unknown-linux-gnu

ci: validate test-kernel-recovery test-vault test-development-simulations check-target

verify-linux-release:
	@test -n "$(MANIFEST)" || (echo "MANIFEST is required" >&2; exit 2)
	@test -n "$(COSIGN_CERTIFICATE_IDENTITY)" || (echo "COSIGN_CERTIFICATE_IDENTITY is required" >&2; exit 2)
	@test -n "$(COSIGN_OIDC_ISSUER)" || (echo "COSIGN_OIDC_ISSUER is required" >&2; exit 2)
	@test -n "$(MANIFEST_SIGNATURE)" || (echo "MANIFEST_SIGNATURE is required" >&2; exit 2)
	@test -f "$(MANIFEST_SIGNATURE)" || (echo "MANIFEST_SIGNATURE must be a regular file" >&2; exit 2)
	@test -n "$(MANIFEST_CERTIFICATE)" || (echo "MANIFEST_CERTIFICATE is required" >&2; exit 2)
	@test -f "$(MANIFEST_CERTIFICATE)" || (echo "MANIFEST_CERTIFICATE must be a regular file" >&2; exit 2)
	$(NODE) scripts/verify-linux-release-manifest.mjs "$(MANIFEST)" --certificate-identity "$(COSIGN_CERTIFICATE_IDENTITY)" --certificate-oidc-issuer "$(COSIGN_OIDC_ISSUER)" --manifest-signature "$(MANIFEST_SIGNATURE)" --manifest-certificate "$(MANIFEST_CERTIFICATE)"

render-linux-release-compose:
	@test -n "$(MANIFEST)" || (echo "MANIFEST is required" >&2; exit 2)
	@test -n "$(COMPOSE)" || (echo "COMPOSE is required" >&2; exit 2)
	@test -n "$(COSIGN_CERTIFICATE_IDENTITY)" || (echo "COSIGN_CERTIFICATE_IDENTITY is required" >&2; exit 2)
	@test -n "$(COSIGN_OIDC_ISSUER)" || (echo "COSIGN_OIDC_ISSUER is required" >&2; exit 2)
	@test -n "$(MANIFEST_SIGNATURE)" || (echo "MANIFEST_SIGNATURE is required" >&2; exit 2)
	@test -f "$(MANIFEST_SIGNATURE)" || (echo "MANIFEST_SIGNATURE must be a regular file" >&2; exit 2)
	@test -n "$(MANIFEST_CERTIFICATE)" || (echo "MANIFEST_CERTIFICATE is required" >&2; exit 2)
	@test -f "$(MANIFEST_CERTIFICATE)" || (echo "MANIFEST_CERTIFICATE must be a regular file" >&2; exit 2)
	$(NODE) scripts/verify-linux-release-manifest.mjs "$(MANIFEST)" --certificate-identity "$(COSIGN_CERTIFICATE_IDENTITY)" --certificate-oidc-issuer "$(COSIGN_OIDC_ISSUER)" --manifest-signature "$(MANIFEST_SIGNATURE)" --manifest-certificate "$(MANIFEST_CERTIFICATE)" --write-compose "$(COMPOSE)"

render-linux-systemd-unit:
	@test -n "$(COMPOSE)" || (echo "COMPOSE is required" >&2; exit 2)
	@test -n "$(SYSTEMD_UNIT)" || (echo "SYSTEMD_UNIT is required" >&2; exit 2)
	$(NODE) scripts/render-linux-systemd-unit.mjs $(COMPOSE) $(SYSTEMD_UNIT)

verify-macos-release:
	@test -n "$(MANIFEST)" || (echo "MANIFEST is required" >&2; exit 2)
	$(NODE) scripts/verify-macos-release-manifest.mjs $(MANIFEST)

generate-release-signing-key:
	@test -n "$(RELEASE_SIGNING_KEY)" || (echo "RELEASE_SIGNING_KEY is required" >&2; exit 2)
	$(NODE) scripts/generate-release-signing-key.mjs --output "$(RELEASE_SIGNING_KEY)"

build-distribution-release:
	@test -n "$(RELEASE_DISTRIBUTION_INPUT)" || (echo "RELEASE_DISTRIBUTION_INPUT is required" >&2; exit 2)
	@test -n "$(RELEASE_SIGNING_KEY)" || (echo "RELEASE_SIGNING_KEY is required" >&2; exit 2)
	@test -n "$(RELEASE_TRUST_ROOT)" || (echo "RELEASE_TRUST_ROOT is required" >&2; exit 2)
	@test -n "$(SIGNED_DISTRIBUTION_MANIFEST)" || (echo "SIGNED_DISTRIBUTION_MANIFEST is required" >&2; exit 2)
	$(NODE) scripts/build-distribution-release.mjs --input "$(RELEASE_DISTRIBUTION_INPUT)" --signing-key "$(RELEASE_SIGNING_KEY)" --trust-root "$(RELEASE_TRUST_ROOT)" --signed-manifest "$(SIGNED_DISTRIBUTION_MANIFEST)"

package-kernel-macos:
	@test -n "$(RELEASE_TRUST_ROOT)" || (echo "RELEASE_TRUST_ROOT is required" >&2; exit 2)
	@test -f "$(RELEASE_TRUST_ROOT)" || (echo "RELEASE_TRUST_ROOT must be a regular file" >&2; exit 2)
	@test -n "$(SIGNED_DISTRIBUTION_MANIFEST)" || (echo "SIGNED_DISTRIBUTION_MANIFEST is required" >&2; exit 2)
	@test -f "$(SIGNED_DISTRIBUTION_MANIFEST)" || (echo "SIGNED_DISTRIBUTION_MANIFEST must be a regular file" >&2; exit 2)
	@test -n "$(DISTRIBUTION_BUNDLE)" || (echo "DISTRIBUTION_BUNDLE is required" >&2; exit 2)
	@test -d "$(DISTRIBUTION_BUNDLE)" || (echo "DISTRIBUTION_BUNDLE must be a directory" >&2; exit 2)
	cargo +$(RUST_TOOLCHAIN) build --locked --manifest-path Cargo.toml --package hermes-kernel --target $(MACOS_KERNEL_TARGET) --release
	install -m 755 target/$(MACOS_KERNEL_TARGET)/release/hermes-kernel $(TAURI_SIDECAR_DIR)/hermes-kernel-$(MACOS_KERNEL_TARGET)
	install -m 644 $(RELEASE_TRUST_ROOT) $(TAURI_SIDECAR_DIR)/hermes-release-trust-root.pb
	install -m 644 $(SIGNED_DISTRIBUTION_MANIFEST) $(TAURI_SIDECAR_DIR)/hermes-signed-distribution-manifest.pb
	@test ! -e "$(TAURI_SIDECAR_DIR)/hermes-distribution" || (echo "remove stale hermes-distribution explicitly before packaging" >&2; exit 2)
	cp -R $(DISTRIBUTION_BUNDLE) $(TAURI_SIDECAR_DIR)/hermes-distribution

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
	cargo +$(RUST_TOOLCHAIN) check --locked --package hermes-development-kernel-operator
