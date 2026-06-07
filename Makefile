COMPOSE = docker compose --env-file $(shell test -f docker/.env && printf docker/.env || printf docker/.env.example) --project-directory docker -f docker/docker-compose.yml
BACKEND_MANIFEST := backend/Cargo.toml

.PHONY: help docker-env compose-config validate lint lint-rust lint-frontend lint-architecture pre-commit-install pre-commit-run dev compose-dev up down restart logs ps shell db-up db-down db-shell clean reset-data frontend-install frontend-dev frontend-lint frontend-lint-ts frontend-check frontend-build frontend-tauri-dev frontend-tauri-build backend-run backend-run-dev backend-watch-dev backend-smoke-dev backend-storage-smoke-dev backend-secrets-smoke-dev backend-event-log-smoke-dev backend-communication-smoke-dev backend-email-sync-smoke-dev backend-email-provider-network-smoke-dev backend-email-sync-cache-dev backend-email-fixture-export-icloud-dev backend-email-fixture-import-dev backend-email-fixture-project-dev backend-account-setup-smoke-dev backend-email-import-smoke-dev backend-messages-smoke-dev backend-contacts-smoke-dev backend-documents-smoke-dev backend-graph-smoke-dev backend-workflow-smoke-dev backend-ai-smoke-dev backend-telegram-smoke-dev backend-whatsapp-smoke-dev backend-graph-project-dev backend-document-processing-dev backend-search-smoke-dev backend-projection-smoke-dev backend-projection-runner-smoke-dev backend-events-api-smoke-dev backend-v1-api-smoke-dev backend-check backend-fmt backend-fmt-check backend-clippy backend-test backend-test-unit backend-test-integration backend-test-all backend-validate

help:
	@printf '%s\n' 'Hermes Hub development commands:'
	@printf '%s\n' '  make docker-env      Create docker/.env from docker/.env.example if missing'
	@printf '%s\n' '  make compose-config  Validate and render Docker Compose config'
	@printf '%s\n' '  make validate        Run the full local/CI validation gate'
	@printf '%s\n' '  make lint            Run strict lint-only gates without tests'
	@printf '%s\n' '  make lint-rust       Run Rust fmt-check and clippy without tests'
	@printf '%s\n' '  make lint-frontend   Run frontend style and TS/Svelte lint without tests'
	@printf '%s\n' '  make lint-architecture Run repository architecture/code boundary guards'
	@printf '%s\n' '  make pre-commit-install Install configured pre-commit hooks'
	@printf '%s\n' '  make pre-commit-run Run configured pre-commit hooks against all files'
	@printf '%s\n' '  make dev             Start PostgreSQL, backend auto-restart, and frontend HMR'
	@printf '%s\n' '  make compose-dev     Start Docker Compose development services in foreground'
	@printf '%s\n' '  make up              Start development services in background'
	@printf '%s\n' '  make down            Stop development services'
	@printf '%s\n' '  make restart         Restart development services'
	@printf '%s\n' '  make logs            Follow development service logs'
	@printf '%s\n' '  make ps              Show development service status'
	@printf '%s\n' '  make shell           Open a shell in the dev container'
	@printf '%s\n' '  make db-up           Start only PostgreSQL in background'
	@printf '%s\n' '  make db-down         Stop PostgreSQL'
	@printf '%s\n' '  make db-shell        Open psql in the Postgres container'
	@printf '%s\n' '  make clean           Stop services and remove Compose orphans'
	@printf '%s\n' '  make reset-data      Delete docker/data contents; requires CONFIRM=yes'
	@printf '%s\n' '  make frontend-install Install frontend dependencies with pnpm'
	@printf '%s\n' '  make frontend-dev    Run the SvelteKit frontend with Vite HMR'
	@printf '%s\n' '  make frontend-lint   Run frontend style and TS/Svelte lint without tests'
	@printf '%s\n' '  make frontend-check  Run frontend style and SvelteKit type checks'
	@printf '%s\n' '  make frontend-build  Build the SvelteKit frontend'
	@printf '%s\n' '  make frontend-tauri-dev Run the Tauri desktop shell in development'
	@printf '%s\n' '  make frontend-tauri-build Build the Tauri desktop shell'
	@printf '%s\n' '  make backend-run     Run the Rust backend locally'
	@printf '%s\n' '  make backend-run-dev Run the Rust backend locally with docker/.env DATABASE_URL'
	@printf '%s\n' '  make backend-watch-dev Run the Rust backend with auto-restart on code changes'
	@printf '%s\n' '  make backend-smoke-dev Run health/readiness smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-storage-smoke-dev Run storage readiness smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-secrets-smoke-dev Run secret reference smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-event-log-smoke-dev Run event log smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-communication-smoke-dev Run communication ingestion smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-email-sync-smoke-dev Run email sync preflight smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-email-provider-network-smoke-dev Run Gmail/IMAP provider network smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-email-sync-cache-dev Fetch read-only IMAP mail into persistent dev cache and projections'
	@printf '%s\n' '  make backend-email-fixture-export-icloud-dev Export redacted iCloud IMAP fixture JSON from read-only latest messages'
	@printf '%s\n' '  make backend-email-fixture-import-dev Import redacted fixture JSON into dev PostgreSQL'
	@printf '%s\n' '  make backend-email-fixture-project-dev Import fixture JSON, project messages/contacts, and rebuild graph'
	@printf '%s\n' '  make backend-account-setup-smoke-dev Run account setup/vault smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-email-import-smoke-dev Run fixture email import smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-messages-smoke-dev Run canonical message projection smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-contacts-smoke-dev Run contacts projection smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-documents-smoke-dev Run document import smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-graph-smoke-dev Run graph store/projection/API smoke tests with dev PostgreSQL'
	@printf '%s\n' '  make backend-workflow-smoke-dev Run domain workflow smoke tests with dev PostgreSQL'
	@printf '%s\n' '  make backend-ai-smoke-dev Run live Ollama AI smoke test'
	@printf '%s\n' '  make backend-telegram-smoke-dev Run Telegram/policy/call fixture smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-whatsapp-smoke-dev Run WhatsApp Web fixture smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-graph-project-dev Project current dev V1 data into graph tables'
	@printf '%s\n' '  make backend-document-processing-dev Run queued document processing jobs with dev PostgreSQL'
	@printf '%s\n' '  make backend-search-smoke-dev Run Tantivy search boundary smoke test'
	@printf '%s\n' '  make backend-projection-smoke-dev Run replay/projection cursor smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-projection-runner-smoke-dev Run projection runner smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-events-api-smoke-dev Run event HTTP API smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-v1-api-smoke-dev Run V1 read API smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-check   Run cargo check for the backend'
	@printf '%s\n' '  make backend-fmt     Format backend Rust code'
	@printf '%s\n' '  make backend-clippy  Run clippy with warnings denied'
	@printf '%s\n' '  make backend-test    Run backend tests'
	@printf '%s\n' '  make backend-validate Run fmt-check, clippy and tests'

docker-env:
	@if [ ! -f docker/.env ]; then \
		cp docker/.env.example docker/.env; \
		printf '%s\n' 'Created docker/.env from docker/.env.example. Review it before running services.'; \
	else \
		if ! grep -q '^HERMES_LOCAL_API_SECRET=' docker/.env; then \
			printf '\nHERMES_LOCAL_API_SECRET=change-me-local-api-secret\n' >> docker/.env; \
			printf '%s\n' 'Added HERMES_LOCAL_API_SECRET to docker/.env. Review it before running services.'; \
		fi; \
		if ! grep -q '^HERMES_SECRET_VAULT_KEY=' docker/.env; then \
			printf '\nHERMES_SECRET_VAULT_KEY=change-me-local-secret-vault-key\n' >> docker/.env; \
			printf '%s\n' 'Added HERMES_SECRET_VAULT_KEY to docker/.env. Review it before running services.'; \
		fi; \
		if ! grep -q '^HERMES_FRONTEND_BIND=' docker/.env; then \
			printf '\nHERMES_FRONTEND_BIND=127.0.0.1\n' >> docker/.env; \
			printf '%s\n' 'Added HERMES_FRONTEND_BIND to docker/.env. Review it before running services.'; \
		fi; \
		if ! grep -q '^HERMES_FRONTEND_PORT=' docker/.env; then \
			printf '\nHERMES_FRONTEND_PORT=5174\n' >> docker/.env; \
			printf '%s\n' 'Added HERMES_FRONTEND_PORT to docker/.env. Review it before running services.'; \
		fi; \
		if ! grep -q '^HERMES_OLLAMA_BASE_URL=' docker/.env; then \
			printf '\nHERMES_OLLAMA_BASE_URL=http://127.0.0.1:11434\n' >> docker/.env; \
			printf '%s\n' 'Added HERMES_OLLAMA_BASE_URL to docker/.env. Review it before running services.'; \
		fi; \
		if ! grep -q '^HERMES_OLLAMA_CHAT_MODEL=' docker/.env; then \
			printf '\nHERMES_OLLAMA_CHAT_MODEL=qwen3:4b\n' >> docker/.env; \
			printf '%s\n' 'Added HERMES_OLLAMA_CHAT_MODEL to docker/.env. Review it before running services.'; \
		fi; \
		if ! grep -q '^HERMES_OLLAMA_EMBED_MODEL=' docker/.env; then \
			printf '\nHERMES_OLLAMA_EMBED_MODEL=qwen3-embedding:4b\n' >> docker/.env; \
			printf '%s\n' 'Added HERMES_OLLAMA_EMBED_MODEL to docker/.env. Review it before running services.'; \
		fi; \
		if ! grep -q '^HERMES_OLLAMA_TIMEOUT_SECONDS=' docker/.env; then \
			printf '\nHERMES_OLLAMA_TIMEOUT_SECONDS=120\n' >> docker/.env; \
			printf '%s\n' 'Added HERMES_OLLAMA_TIMEOUT_SECONDS to docker/.env. Review it before running services.'; \
		fi; \
		printf '%s\n' 'docker/.env already exists.'; \
	fi

compose-config: docker-env
	$(COMPOSE) config

validate: compose-config backend-validate backend-storage-smoke-dev backend-secrets-smoke-dev backend-event-log-smoke-dev backend-communication-smoke-dev backend-email-sync-smoke-dev backend-email-provider-network-smoke-dev backend-account-setup-smoke-dev backend-email-import-smoke-dev backend-messages-smoke-dev backend-contacts-smoke-dev backend-documents-smoke-dev backend-graph-smoke-dev backend-workflow-smoke-dev backend-ai-smoke-dev backend-telegram-smoke-dev backend-whatsapp-smoke-dev backend-search-smoke-dev backend-events-api-smoke-dev backend-v1-api-smoke-dev backend-projection-runner-smoke-dev backend-smoke-dev frontend-check frontend-build

lint: lint-rust lint-frontend lint-architecture

lint-rust: backend-fmt-check backend-clippy

lint-frontend: frontend-lint

lint-architecture:
	node scripts/check-architecture.mjs
	node scripts/check-code-boundaries.mjs

pre-commit-install:
	pre-commit install

pre-commit-run:
	pre-commit run --all-files

dev: docker-env
	@set -eu; \
		if ! command -v pnpm >/dev/null 2>&1; then \
			printf '%s\n' 'pnpm is required for frontend dev. Install pnpm and re-run make dev.'; \
			exit 1; \
		fi; \
		if ! command -v watchexec >/dev/null 2>&1 && ! cargo watch --version >/dev/null 2>&1; then \
			printf '%s\n' 'Backend auto-restart requires watchexec or cargo-watch.'; \
			printf '%s\n' 'Install one of:'; \
			printf '%s\n' '  brew install watchexec'; \
			printf '%s\n' '  cargo install cargo-watch'; \
			exit 1; \
		fi; \
		set -a; . docker/.env; set +a; \
		backend_bind="$${HERMES_BACKEND_BIND:-127.0.0.1}"; \
		backend_port="$${HERMES_BACKEND_PORT:-8080}"; \
		frontend_bind="$${HERMES_FRONTEND_BIND:-127.0.0.1}"; \
		frontend_port="$${HERMES_FRONTEND_PORT:-5174}"; \
		if command -v lsof >/dev/null 2>&1 && lsof -nP -iTCP:"$$backend_port" -sTCP:LISTEN >/dev/null 2>&1; then \
			printf '%s\n' "Backend port $$backend_port is already in use. Stop the existing process or change HERMES_BACKEND_PORT in docker/.env."; \
			exit 1; \
		fi; \
		if command -v lsof >/dev/null 2>&1 && lsof -nP -iTCP:"$$frontend_port" -sTCP:LISTEN >/dev/null 2>&1; then \
			printf '%s\n' "Frontend port $$frontend_port is already in use. Stop the existing process or change HERMES_FRONTEND_PORT in docker/.env."; \
			exit 1; \
		fi; \
		backend_pid=""; \
		frontend_pid=""; \
		cleanup() { \
			status="$$?"; \
			trap - EXIT INT TERM; \
			if [ -n "$$backend_pid" ]; then \
				kill "$$backend_pid" 2>/dev/null || true; \
				wait "$$backend_pid" 2>/dev/null || true; \
			fi; \
			if [ -n "$$frontend_pid" ]; then \
				kill "$$frontend_pid" 2>/dev/null || true; \
				wait "$$frontend_pid" 2>/dev/null || true; \
			fi; \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
			exit "$$status"; \
		}; \
		trap cleanup EXIT INT TERM; \
		$(MAKE) db-up; \
		export DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}"; \
		export HERMES_LOCAL_API_SECRET="$${HERMES_LOCAL_API_SECRET}"; \
		export HERMES_SECRET_VAULT_KEY="$${HERMES_SECRET_VAULT_KEY}"; \
		export HERMES_HTTP_ADDR="$$backend_bind:$$backend_port"; \
		if command -v watchexec >/dev/null 2>&1; then \
			watchexec --restart --watch backend/src --watch backend/migrations --watch backend/Cargo.toml --watch backend/Cargo.lock --exts rs,toml,sql -- cargo run --manifest-path $(BACKEND_MANIFEST) & \
		else \
			cargo watch -w backend/src -w backend/migrations -w backend/Cargo.toml -w backend/Cargo.lock -x 'run --manifest-path backend/Cargo.toml' & \
		fi; \
		backend_pid="$$!"; \
		( \
			cd frontend && \
			exec env \
				VITE_HERMES_API_BASE_URL="http://$$backend_bind:$$backend_port" \
				VITE_HERMES_LOCAL_API_SECRET="$${HERMES_LOCAL_API_SECRET}" \
				pnpm dev --host "$$frontend_bind" --port "$$frontend_port" --strictPort \
		) & \
		frontend_pid="$$!"; \
		printf '%s\n' "Backend:  http://$$backend_bind:$$backend_port (auto-restart)"; \
		printf '%s\n' "Frontend: http://$$frontend_bind:$$frontend_port (Vite HMR)"; \
		printf '%s\n' 'Press Ctrl+C to stop frontend/backend and PostgreSQL.'; \
		while kill -0 "$$backend_pid" 2>/dev/null && kill -0 "$$frontend_pid" 2>/dev/null; do \
			sleep 1; \
		done; \
		wait "$$backend_pid" "$$frontend_pid"

compose-dev: docker-env
	$(COMPOSE) up --build

up: docker-env
	$(COMPOSE) up -d --build --wait

down:
	$(COMPOSE) down

restart: down up

logs:
	$(COMPOSE) logs -f

ps:
	$(COMPOSE) ps

shell: docker-env
	$(COMPOSE) run --rm dev bash

db-up: docker-env
	$(COMPOSE) up -d --wait postgres

db-down:
	$(COMPOSE) stop postgres

db-shell: docker-env
	$(COMPOSE) exec postgres sh -lc 'psql -U "$$POSTGRES_USER" -d "$$POSTGRES_DB"'

clean:
	$(COMPOSE) down --remove-orphans

reset-data:
	@test "$$CONFIRM" = "yes" || (printf '%s\n' 'Refusing to delete docker/data. Re-run as: make reset-data CONFIRM=yes' && exit 1)
	$(COMPOSE) down --remove-orphans
	rm -rf docker/data/*
	touch docker/data/.gitkeep

frontend-install:
	cd frontend && pnpm install

frontend-dev: docker-env
	@set -eu; \
		set -a; . docker/.env; set +a; \
		frontend_port="$${HERMES_FRONTEND_PORT:-5174}"; \
		if command -v lsof >/dev/null 2>&1 && lsof -nP -iTCP:"$$frontend_port" -sTCP:LISTEN >/dev/null 2>&1; then \
			printf '%s\n' "Frontend port $$frontend_port is already in use. Stop the existing process or change HERMES_FRONTEND_PORT in docker/.env."; \
			exit 1; \
		fi; \
		cd frontend && \
		VITE_HERMES_API_BASE_URL="http://$${HERMES_BACKEND_BIND:-127.0.0.1}:$${HERMES_BACKEND_PORT:-8080}" \
		VITE_HERMES_LOCAL_API_SECRET="$${HERMES_LOCAL_API_SECRET}" \
		pnpm dev --host "$${HERMES_FRONTEND_BIND:-127.0.0.1}" --port "$$frontend_port" --strictPort

frontend-lint: frontend-lint-ts
	cd frontend && pnpm check

frontend-lint-ts:
	cd frontend && pnpm lint:ts

frontend-check:
	cd frontend && pnpm check

frontend-build:
	cd frontend && pnpm build

frontend-tauri-dev:
	cd frontend && pnpm tauri dev

frontend-tauri-build:
	cd frontend && pnpm tauri build

backend-run:
	cargo run --manifest-path $(BACKEND_MANIFEST)

backend-run-dev: docker-env
	@set -a; . docker/.env; set +a; \
		DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		HERMES_LOCAL_API_SECRET="$${HERMES_LOCAL_API_SECRET}" \
		HERMES_SECRET_VAULT_KEY="$${HERMES_SECRET_VAULT_KEY}" \
		cargo run --manifest-path $(BACKEND_MANIFEST)

backend-watch-dev: docker-env
	@set -eu; \
		if ! command -v watchexec >/dev/null 2>&1 && ! cargo watch --version >/dev/null 2>&1; then \
			printf '%s\n' 'Backend auto-restart requires watchexec or cargo-watch.'; \
			printf '%s\n' 'Install one of:'; \
			printf '%s\n' '  brew install watchexec'; \
			printf '%s\n' '  cargo install cargo-watch'; \
			exit 1; \
		fi; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		backend_port="$${HERMES_BACKEND_PORT:-8080}"; \
		if command -v lsof >/dev/null 2>&1 && lsof -nP -iTCP:"$$backend_port" -sTCP:LISTEN >/dev/null 2>&1; then \
			printf '%s\n' "Backend port $$backend_port is already in use. Stop the existing process or change HERMES_BACKEND_PORT in docker/.env."; \
			exit 1; \
		fi; \
		export DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}"; \
		export HERMES_LOCAL_API_SECRET="$${HERMES_LOCAL_API_SECRET}"; \
		export HERMES_SECRET_VAULT_KEY="$${HERMES_SECRET_VAULT_KEY}"; \
		export HERMES_HTTP_ADDR="$${HERMES_BACKEND_BIND:-127.0.0.1}:$${HERMES_BACKEND_PORT:-8080}"; \
		printf '%s\n' "Backend: http://$${HERMES_BACKEND_BIND:-127.0.0.1}:$${HERMES_BACKEND_PORT:-8080} (auto-restart)"; \
		if command -v watchexec >/dev/null 2>&1; then \
			exec watchexec --restart --watch backend/src --watch backend/migrations --watch backend/Cargo.toml --watch backend/Cargo.lock --exts rs,toml,sql -- cargo run --manifest-path $(BACKEND_MANIFEST); \
		fi; \
		exec cargo watch -w backend/src -w backend/migrations -w backend/Cargo.toml -w backend/Cargo.lock -x 'run --manifest-path backend/Cargo.toml'

backend-smoke-dev: docker-env
	@set -eu; \
		backend_pid=""; \
		cleanup() { \
			if [ -n "$$backend_pid" ]; then \
				kill "$$backend_pid" 2>/dev/null || true; \
				wait "$$backend_pid" 2>/dev/null || true; \
			fi; \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		smoke_port="$${HERMES_BACKEND_SMOKE_PORT:-18081}"; \
		DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		HERMES_LOCAL_API_SECRET="$${HERMES_LOCAL_API_SECRET}" \
		HERMES_HTTP_ADDR="127.0.0.1:$${smoke_port}" \
		cargo run --manifest-path $(BACKEND_MANIFEST) >/tmp/hermes-hub-backend-smoke.log 2>&1 & \
		backend_pid="$$!"; \
		for _ in 1 2 3 4 5 6 7 8 9 10; do \
			if curl -fsS "http://127.0.0.1:$${smoke_port}/healthz" >/tmp/hermes-hub-healthz.json \
				&& curl -fsS "http://127.0.0.1:$${smoke_port}/readyz" >/tmp/hermes-hub-readyz.json; then \
				printf '%s\n' 'healthz:'; \
				cat /tmp/hermes-hub-healthz.json; \
				printf '\n%s\n' 'readyz:'; \
				cat /tmp/hermes-hub-readyz.json; \
				printf '\n'; \
				exit 0; \
			fi; \
			sleep 1; \
		done; \
		cat /tmp/hermes-hub-backend-smoke.log; \
		exit 1

backend-storage-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test storage against_postgres -- --nocapture --test-threads=1

backend-secrets-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test secrets against_postgres -- --nocapture --test-threads=1

backend-event-log-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test event_log against_postgres -- --nocapture --test-threads=1

backend-communication-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test communication_ingestion against_postgres -- --nocapture --test-threads=1

backend-email-sync-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test email_sync -- --nocapture --test-threads=1

backend-email-provider-network-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test email_provider_network -- --nocapture --test-threads=1

backend-email-sync-cache-dev: docker-env
	@set -eu; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo run --manifest-path $(BACKEND_MANIFEST) --bin hermes-email-sync-dev

backend-email-fixture-export-icloud-dev:
	cargo run --manifest-path $(BACKEND_MANIFEST) --bin hermes-email-fixture-export

backend-email-fixture-import-dev: docker-env
	@set -eu; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		HERMES_EMAIL_FIXTURE_MODE=import \
		cargo run --manifest-path $(BACKEND_MANIFEST) --bin hermes-email-fixture-dev

backend-email-fixture-project-dev: docker-env
	@set -eu; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		HERMES_EMAIL_FIXTURE_MODE=project \
		cargo run --manifest-path $(BACKEND_MANIFEST) --bin hermes-email-fixture-dev

backend-account-setup-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test secret_vault -- --nocapture --test-threads=1; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test email_account_setup -- --nocapture --test-threads=1

backend-email-import-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test email_import against_postgres -- --nocapture --test-threads=1

backend-messages-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test messages against_postgres -- --nocapture --test-threads=1

backend-contacts-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test contacts -- --nocapture --test-threads=1

backend-documents-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test documents -- --nocapture --test-threads=1

backend-graph-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test graph --test graph_projection --test graph_api -- --nocapture --test-threads=1

backend-workflow-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(COMPOSE) stop postgres >/dev/null 2>&1 || true; \
		}; \
		drop_test_db() { \
			if [ -n "$${test_db:-}" ]; then \
				$(COMPOSE) run --rm --no-deps postgres sh -lc 'PGPASSWORD="$$1" psql -h postgres -U "$$2" -d postgres -c "$$3"' _ "$${HERMES_POSTGRES_PASSWORD}" "$${HERMES_POSTGRES_USER}" "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '$${test_db}' AND pid <> pg_backend_pid();" >/dev/null 2>&1 || true; \
				$(COMPOSE) run --rm --no-deps postgres sh -lc 'PGPASSWORD="$$1" psql -h postgres -U "$$2" -d postgres -c "$$3"' _ "$${HERMES_POSTGRES_PASSWORD}" "$${HERMES_POSTGRES_USER}" "DROP DATABASE IF EXISTS \"$${test_db}\";" >/dev/null 2>&1 || true; \
				test_db=""; \
			fi; \
		}; \
		trap cleanup EXIT; \
		$(COMPOSE) up -d --wait postgres; \
		set -a; . docker/.env; set +a; \
		test_db=""; \
		for test_name in \
			projects \
			projects_api \
			project_link_reviews \
			task_candidates \
			task_candidates_api \
			contact_identity \
			contact_identity_api \
			document_processing \
			document_processing_api; do \
			test_db="hermes_v2_workflow_$${test_name}_$$(date +%s%N)"; \
			$(COMPOSE) run --rm --no-deps postgres sh -lc 'PGPASSWORD="$$1" psql -h postgres -U "$$2" -d postgres -c "$$3"' _ "$${HERMES_POSTGRES_PASSWORD}" "$${HERMES_POSTGRES_USER}" "CREATE DATABASE \"$${test_db}\";" >/dev/null; \
			if HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${test_db}" \
				cargo test --manifest-path $(BACKEND_MANIFEST) --test "$$test_name" -- --nocapture --test-threads=1; then \
				:; \
			else \
				status="$$?"; \
				drop_test_db; \
				exit "$$status"; \
			fi; \
			drop_test_db; \
		done

backend-ai-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		smoke_ollama_base_url="$${HERMES_AI_SMOKE_OLLAMA_BASE_URL:-http://192.168.1.2:11434}"; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		HERMES_OLLAMA_BASE_URL="$$smoke_ollama_base_url" \
		HERMES_OLLAMA_CHAT_MODEL="$${HERMES_OLLAMA_CHAT_MODEL:-qwen3:4b}" \
		HERMES_OLLAMA_EMBED_MODEL="$${HERMES_OLLAMA_EMBED_MODEL:-qwen3-embedding:4b}" \
		HERMES_OLLAMA_TIMEOUT_SECONDS="$${HERMES_OLLAMA_TIMEOUT_SECONDS:-120}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test ai --test ai_smoke -- --nocapture --test-threads=1

backend-telegram-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test telegram -- --nocapture --test-threads=1

backend-whatsapp-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test whatsapp -- --nocapture --test-threads=1

backend-graph-project-dev: docker-env
	@set -eu; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo run --manifest-path $(BACKEND_MANIFEST) --bin hermes-graph-project

backend-document-processing-dev: docker-env
	@set -eu; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo run --manifest-path $(BACKEND_MANIFEST) --bin hermes-document-process

backend-search-smoke-dev:
	cargo test --manifest-path $(BACKEND_MANIFEST) --test search search_index_returns_message_by_body_term -- --nocapture

backend-projection-smoke-dev: backend-event-log-smoke-dev backend-projection-runner-smoke-dev

backend-projection-runner-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test projection_runner -- --nocapture --test-threads=1

backend-events-api-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test events_api -- --nocapture --test-threads=1

backend-v1-api-smoke-dev: docker-env
	@set -eu; \
		cleanup() { \
			$(MAKE) db-down >/dev/null 2>&1 || true; \
		}; \
		trap cleanup EXIT; \
		$(MAKE) db-up; \
		set -a; . docker/.env; set +a; \
		HERMES_TEST_DATABASE_URL="postgres://$${HERMES_POSTGRES_USER}:$${HERMES_POSTGRES_PASSWORD}@127.0.0.1:$${HERMES_POSTGRES_PORT}/$${HERMES_POSTGRES_DB}" \
		cargo test --manifest-path $(BACKEND_MANIFEST) --test v1_api -- --nocapture --test-threads=1

backend-check:
	cargo check --manifest-path $(BACKEND_MANIFEST) --all-targets --all-features

backend-fmt:
	cargo fmt --manifest-path $(BACKEND_MANIFEST)

backend-fmt-check:
	cargo fmt --manifest-path $(BACKEND_MANIFEST) --check

backend-clippy:
	cargo clippy --manifest-path $(BACKEND_MANIFEST) --all-targets --all-features -- -D warnings

backend-test:
	cargo test --manifest-path $(BACKEND_MANIFEST)

backend-validate: backend-fmt-check backend-clippy backend-test-unit

backend-test-unit:
	cargo test --manifest-path $(BACKEND_MANIFEST) --lib

backend-test-integration:
	cargo test --manifest-path $(BACKEND_MANIFEST) --test '*'

backend-test-all: backend-test-unit backend-test-integration
