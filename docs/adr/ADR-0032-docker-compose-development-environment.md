# ADR-0032 Docker Compose Development Environment

Status: Proposed

## Context

Hermes Hub targets a local-first desktop product with PostgreSQL, Rust, SvelteKit and Tauri. Development needs a repeatable local environment without introducing production deployment semantics or scattering Docker files across the repository.

## Decision

Use Docker Compose for local development infrastructure. Keep Docker-specific files under `docker/`, including `docker/docker-compose.yml`, `docker/Dockerfile`, environment examples and bind-mounted development data under `docker/data/`.

Expose developer commands through the root `Makefile`.

## Consequences

- Local development can start from a consistent Compose entry point.
- Persistent dev data stays under `docker/data/` and is ignored by Git.
- Docker files do not leak into backend or frontend implementation directories.
- This setup is not a production deployment model.
- Any future production/self-hosted deployment design requires a separate ADR.
