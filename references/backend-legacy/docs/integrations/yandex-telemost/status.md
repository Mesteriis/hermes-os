# Yandex Telemost Status

Status: `FOUNDATION_PATCH_APPLIED`, 2026-06-28.

## Proposed code paths

```text
backend/src/integrations/yandex_telemost
backend/src/app/provider_runtime_handlers/yandex_telemost.rs
frontend/src/integrations/yandexTelemost
frontend/src-tauri/src/yandex_telemost_companion.rs
```

## Validation state

The source patch has been applied to the current repository structure. Local
validation must use the repository-configured tooling and package manager.

Required Telemost-domain validation:

```text
git diff --check
make architecture-check
make code-boundaries-check
make backend-fmt-check
make backend-clippy
cargo test --manifest-path backend/Cargo.toml --lib app::provider_runtime_handlers::yandex_telemost::tests::unknown_cohosts_review_item_uses_relationship_flow -- --exact
cargo test --manifest-path backend/Cargo.toml --lib app::provider_runtime_handlers::yandex_telemost::tests::unmatched_meeting_link_review_item_uses_project_flow -- --exact
cargo test --manifest-path backend/Cargo.toml --lib workflows::yandex_telemost_calendar_matching::tests::telemost_cohosts_are_projected_into_matched_calendar_event_participants -- --exact
cd frontend && pnpm lint
cd frontend && pnpm typecheck
cargo test --manifest-path frontend/src-tauri/Cargo.toml yandex_telemost_companion::tests::initialization_script_contains_multi_selector_speaker_heuristics -- --exact
```

Broader repository suites such as `make backend-test` and `cd frontend &&
pnpm test:unit` remain useful CI signals, but they are not treated here as
Telemost-domain completion gates because they currently include unrelated
modules and unrelated failing boundary tests outside the Telemost scope.

## Known follow-up work

- No known Telemost-domain follow-up gaps remain from the available documentation and implemented provider/runtime boundary. Remaining unsupported items in README are explicit non-goals or later-scope capabilities, not foundation-domain gaps.
