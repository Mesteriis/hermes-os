# Contributing to Hermes Hub

Hermes Hub is a local-first personal knowledge and communication system. The
project is early, architecture-heavy, and intentionally conservative about
privacy, provenance and safety.

## Before Contributing

- Read `AGENTS.md` and the relevant ADRs in `docs/adr/`.
- Do not add cloud dependencies, provider writes or AI agent side effects
  without ADR-backed design.
- Do not commit secrets, private mail/message data, local fixtures with
  personal content, `docker/.env` or generated data under `docker/data/`.
- Keep changes small, testable and consistent with the existing Rust,
  SvelteKit and Tauri structure.

## Development Setup

```sh
make docker-env
make validate
```

For backend-only changes:

```sh
make backend-validate
```

For frontend-only changes:

```sh
make frontend-check
make frontend-build
```

## Pull Requests

- Explain what changed and why.
- Reference ADRs or docs that constrain the change.
- Include validation commands and results.
- Add regression tests for behavior changes.
- Keep generated/local data out of the diff.

## Security and Privacy

Treat imported messages, documents and provider data as private and untrusted.
Do not include private user data in issues, pull requests, screenshots or test
fixtures.
