SHELL := /usr/bin/env bash

.DEFAULT_GOAL := help

.PHONY: help dev logs build migrate vault-backup vault-restore clean clean-data clean-vault

help:
	@printf '%s\n' 'Hermes development commands:'
	@printf '%s\n' '  make dev           Start PostgreSQL, backend watcher, and Vite dev server'
	@printf '%s\n' '  make logs          Tail the active live development log'
	@printf '%s\n' '  make build         Build backend, frontend, and Tauri release artifacts'
	@printf '%s\n' '  make migrate       Start PostgreSQL if needed and run backend-managed migrations'
	@printf '%s\n' '  make vault-backup  Create a timestamped PostgreSQL + vault backup'
	@printf '%s\n' '  make vault-restore Interactively restore PostgreSQL + vault from a backup'
	@printf '%s\n' '  make clean         Remove build artifacts, temporary files, and logs'
	@printf '%s\n' '  make clean-data    Delete local PostgreSQL data after confirmation'
	@printf '%s\n' '  make clean-vault   Delete local vault data after confirmation'

dev:
	@./scripts/dev.sh

logs:
	@./scripts/logs.sh

build:
	@./scripts/build.sh

migrate:
	@./scripts/migrate.sh

vault-backup:
	@./scripts/vault-backup.sh

vault-restore:
	@./scripts/vault-restore.sh

clean:
	@./scripts/clean.sh

clean-data:
	@./scripts/clean-data.sh

clean-vault:
	@./scripts/clean-vault.sh
