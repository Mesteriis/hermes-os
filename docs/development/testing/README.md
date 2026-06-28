# Testing Infrastructure

Status: documentation package aligned to the current repository structure.

Hermes uses a split test stack:

- Rust backend execution runs through `cargo-nextest`.
- Backend integration and coverage runs go through the `crates/testkit` session harness so PostgreSQL and NATS testcontainers are reused and cleaned correctly.
- Local wrapper scripts and Makefile nextest targets force a visible progress mode via `--show-progress`.
- Post-run JUnit analysis prints a compact completed/passed/failed/flaky summary with an ASCII progress bar, so non-interactive Codex/CI output is not silent after a test set finishes.
- Frontend unit tests stay on Vitest.
- Architecture checks remain first-class and are part of the test taxonomy.

## Command map

- `make test-fast` - local fast loop: backend unit + architecture + snapshots + frontend unit tests
- `make test` - full local validation-oriented test run
- `make test-ci` - backend CI-oriented nextest run plus frontend unit tests
- `make test-unit`
- `make test-integration`
- `make test-e2e`
- `make test-architecture`
- `make test-snapshot`
- `make coverage`
- `make coverage-html`
- `make coverage-ci`
- `make mutants`
- `make audit`
- `make deny`
- `make security`
- `make udeps`
- `make watch-test`
- `make watch-unit`
- `make watch-integration`
- `make cache-stats`
- `make cache-reset`
- `make test-performance-report`

## Classification model

Hermes does not yet physically relocate every backend test into `tests/unit`, `tests/integration`, `tests/e2e`, `tests/architecture`, `tests/snapshots`. The repository now uses a stable logical classification generated from the current target naming and a dedicated snapshot target:

- `unit` - Rust library tests under `backend/src` and `crates/testkit/src`
- `integration` - backend integration targets that are not architecture/e2e/snapshot targets
- `e2e` - high-surface API/runtime targets such as `*_api`, stream/websocket API targets, `communications_connectrpc`, `omniroute`, `hard_v1_routes`
- `architecture` - `*_architecture.rs` targets plus JS architecture guards
- `snapshot` - backend snapshot targets using `insta`

The classifier lives in `scripts/test/backend-test-targets.mjs`.

## Reports

`cargo-nextest` writes JUnit XML into `target/nextest/<profile>/junit.xml`.

Post-run summaries are written into `reports/test-performance/` by `scripts/test/analyze-nextest-junit.mjs`.

Current baseline and optimization notes live in:

- `reports/test-performance/README.md`
- `reports/test-performance/2026-06-23-baseline.md`
- `reports/test-performance/backend-full.md`
- `docs/development/testing/status.md`

## Navigation

- [Status](./status.md)
- [CI](./ci.md)
- [Coverage](./coverage.md)
- [Mutation Testing](./mutation-testing.md)
- [Nextest](./nextest.md)
- [Security](./security.md)
- [Snapshots](./snapshots.md)
