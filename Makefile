COMPOSE = docker compose --env-file $(shell test -f docker/.env && printf docker/.env || printf docker/.env.example) --project-directory docker -f docker/docker-compose.yml
BACKEND_MANIFEST := backend/Cargo.toml

.PHONY: help docker-env compose-config validate dev up down restart logs ps shell db-up db-down db-shell clean reset-data frontend-install frontend-check frontend-build frontend-tauri-dev frontend-tauri-build backend-run backend-run-dev backend-smoke-dev backend-storage-smoke-dev backend-secrets-smoke-dev backend-event-log-smoke-dev backend-communication-smoke-dev backend-email-sync-smoke-dev backend-email-provider-network-smoke-dev backend-account-setup-smoke-dev backend-email-import-smoke-dev backend-messages-smoke-dev backend-contacts-smoke-dev backend-documents-smoke-dev backend-search-smoke-dev backend-projection-smoke-dev backend-projection-runner-smoke-dev backend-events-api-smoke-dev backend-v1-api-smoke-dev backend-check backend-fmt backend-fmt-check backend-clippy backend-test backend-validate

help:
	@printf '%s\n' 'Hermes Hub development commands:'
	@printf '%s\n' '  make docker-env      Create docker/.env from docker/.env.example if missing'
	@printf '%s\n' '  make compose-config  Validate and render Docker Compose config'
	@printf '%s\n' '  make validate        Run the full local/CI validation gate'
	@printf '%s\n' '  make dev             Start development services in foreground'
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
	@printf '%s\n' '  make frontend-check  Run SvelteKit type checks'
	@printf '%s\n' '  make frontend-build  Build the SvelteKit frontend'
	@printf '%s\n' '  make frontend-tauri-dev Run the Tauri desktop shell in development'
	@printf '%s\n' '  make frontend-tauri-build Build the Tauri desktop shell'
	@printf '%s\n' '  make backend-run     Run the Rust backend locally'
	@printf '%s\n' '  make backend-run-dev Run the Rust backend locally with docker/.env DATABASE_URL'
	@printf '%s\n' '  make backend-smoke-dev Run health/readiness smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-storage-smoke-dev Run storage readiness smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-secrets-smoke-dev Run secret reference smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-event-log-smoke-dev Run event log smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-communication-smoke-dev Run communication ingestion smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-email-sync-smoke-dev Run email sync preflight smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-email-provider-network-smoke-dev Run Gmail/IMAP provider network smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-account-setup-smoke-dev Run account setup/vault smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-email-import-smoke-dev Run fixture email import smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-messages-smoke-dev Run canonical message projection smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-contacts-smoke-dev Run contacts projection smoke test with dev PostgreSQL'
	@printf '%s\n' '  make backend-documents-smoke-dev Run document import smoke test with dev PostgreSQL'
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
		if ! grep -q '^HERMES_LOCAL_API_TOKEN=' docker/.env; then \
			legacy_token="$$(awk -F= '$$1 == "HERMES_LOCAL_WRITE_TOKEN" { print $$2; exit }' docker/.env)"; \
			if [ -n "$$legacy_token" ]; then \
				printf '\nHERMES_LOCAL_API_TOKEN=%s\n' "$$legacy_token" >> docker/.env; \
			else \
				printf '\nHERMES_LOCAL_API_TOKEN=change-me-local-api-token\n' >> docker/.env; \
			fi; \
			printf '%s\n' 'Added HERMES_LOCAL_API_TOKEN to docker/.env. Review it before running services.'; \
		fi; \
		if ! grep -q '^HERMES_SECRET_VAULT_PATH=' docker/.env; then \
			printf '\nHERMES_SECRET_VAULT_PATH=docker/data/secrets/hermes-secrets.vault.json\n' >> docker/.env; \
			printf '%s\n' 'Added HERMES_SECRET_VAULT_PATH to docker/.env. Review it before running services.'; \
		fi; \
		if ! grep -q '^HERMES_SECRET_VAULT_KEY=' docker/.env; then \
			printf '\nHERMES_SECRET_VAULT_KEY=change-me-local-secret-vault-key\n' >> docker/.env; \
			printf '%s\n' 'Added HERMES_SECRET_VAULT_KEY to docker/.env. Review it before running services.'; \
		fi; \
		printf '%s\n' 'docker/.env already exists.'; \
	fi

compose-config: docker-env
	$(COMPOSE) config

validate: compose-config backend-validate backend-storage-smoke-dev backend-secrets-smoke-dev backend-event-log-smoke-dev backend-communication-smoke-dev backend-email-sync-smoke-dev backend-email-provider-network-smoke-dev backend-account-setup-smoke-dev backend-email-import-smoke-dev backend-messages-smoke-dev backend-contacts-smoke-dev backend-documents-smoke-dev backend-search-smoke-dev backend-events-api-smoke-dev backend-v1-api-smoke-dev backend-projection-runner-smoke-dev backend-smoke-dev frontend-check frontend-build

dev: docker-env
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
		HERMES_LOCAL_API_TOKEN="$${HERMES_LOCAL_API_TOKEN}" \
		HERMES_SECRET_VAULT_PATH="$${HERMES_SECRET_VAULT_PATH}" \
		HERMES_SECRET_VAULT_KEY="$${HERMES_SECRET_VAULT_KEY}" \
		cargo run --manifest-path $(BACKEND_MANIFEST)

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
		HERMES_LOCAL_API_TOKEN="$${HERMES_LOCAL_API_TOKEN}" \
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

backend-validate: backend-fmt-check backend-clippy backend-test
