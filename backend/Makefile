.PHONY: build test dev docker tauri clean

build:
	cargo +1.97.0 build --locked --workspace
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
