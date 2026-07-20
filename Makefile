SHELL := /usr/bin/env bash

.DEFAULT_GOAL := help

.PHONY: help build test dev docker tauri clean

help:
	@printf '%s\n' 'Hermes development commands:'
	@printf '%s\n' '  make build   Build the clean-room backend and browser client'
	@printf '%s\n' '  make test    Run tests impacted by the current changes'
	@printf '%s\n' '  make dev     Start the local platform, Kernel and Vite together'
	@printf '%s\n' '  make docker  Start the local PostgreSQL, PgBouncer and NATS contour'
	@printf '%s\n' '  make tauri   Build the desktop application'
	@printf '%s\n' '  make clean   Remove reproducible Hermes build and test output'

build test dev docker tauri clean:
	@$(MAKE) -C backend $@
