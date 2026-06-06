# V4 Closure Checklist

## Release Goal

Version 4.0 is complete when Hermes Hub provides a desktop-configurable Telegram client foundation with multiple Telegram user and bot accounts, policy-approved automated sending, 1:1 audio call state, local call transcription artifacts, plugin/capability policy visibility and backup-aware V4 data handling.

## In Scope

- ADR-0050 governs Telegram runtime, policy-backed sending and call intelligence.
- Multiple `telegram_user` and `telegram_bot` accounts.
- Telegram fixture runtime for CI and smoke validation.
- Account-scoped Telegram raw records, checkpoints, chats and projected messages.
- UI-configured templates and automation policies.
- Automated-send dry-run and audit trail.
- 1:1 audio call metadata and transcript artifact storage.
- Local speech-to-text provider boundary with fixture provider by default.
- Desktop-only V4 surfaces for Telegram, policies and call transcripts.
- Protected V4 capability contract for available, blocked and unsupported capabilities.

## Out Of Scope For V4

- Video calls.
- Group calls.
- Screen sharing.
- Hidden recording.
- Cloud transcription by default.
- Mobile UI.
- Training or fine-tuning on Telegram data.
- Third-party plugin code execution.

## Acceptance Gate Status

- [x] ADR-0050 documents V4 Telegram, policy automation and call intelligence constraints.
- [x] V4 roadmap closure checklist exists.
- [x] Provider account model accepts `telegram_user` and `telegram_bot` without breaking email providers.
- [x] Telegram secret purposes are account-scoped and compatible only with non-plaintext secret references.
- [x] Backend migration creates Telegram chat, outbound policy and call transcript tables.
- [x] Backend exposes protected `/api/v4/telegram/*`, `/api/v4/policies/*` and `/api/v4/calls/*` foundation endpoints.
- [x] Automated-send dry-run rejects sends outside enabled policies.
- [x] Automated-send dry-run records auditable preview metadata without storing secret values.
- [x] Call transcript storage preserves account, call and source provenance.
- [x] Protected `/api/v4/capabilities` exposes fixture-ready, live-blocked and V4.x unsupported capabilities.
- [x] Desktop V4 Telegram account, policy, call transcript and runtime guardrail surfaces call protected backend APIs.
- [x] `make backend-v4-smoke-dev` covers V4 fixture runtime, policy and transcript storage.
- [x] `make validate`, `make frontend-check` and `make frontend-build` pass after V4 UI integration.
- [x] Desktop browser smoke validates Telegram, policy and call transcript V4 surfaces render without layout breakage.
