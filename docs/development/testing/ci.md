# CI Test Topology

The CI split is:

## Pull Requests

- architecture
- backend fmt
- backend clippy
- backend unit
- backend snapshots
- frontend lint/test/build

## Push to `main`

Everything from pull requests, plus:

- backend integration
- coverage
- security

## Nightly

- backend e2e
- mutation testing

This keeps the default PR gate fast enough for iteration while leaving heavy container-backed and mutation-based checks to the slower lanes.
