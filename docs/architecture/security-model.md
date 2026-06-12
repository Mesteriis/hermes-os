# Security Model

## Security Goals

- protect personal communication and documents
- prevent accidental data exfiltration
- keep local-first trust boundaries clear
- make AI/tool actions auditable
- avoid hidden outbound behavior

## Trust Boundaries

| Boundary | Risk | Control |
| --- | --- | --- |
| Provider adapters | credential leakage, malformed data | secret store, strict parsing, scoped permissions |
| AI runtime | prompt injection, untrusted content | tool capability checks, source labeling, constrained actions |
| Plugin host | arbitrary file/network access | explicit capability manifest and runtime enforcement |
| Tauri bridge | UI-to-system escalation | narrow commands, input validation |
| Search and export | bulk data disclosure | confirmation, audit, export scopes |

## Authentication and Local Access

Initial product mode is single-user local desktop. The architecture must still distinguish:

- local app session
- provider credentials
- plugin capabilities
- agent tool permissions
- export and backup permissions

ADR-0056 defines the current local API guard. The backend applies a router-level
shared secret check with `HERMES_LOCAL_API_SECRET` and the `X-Hermes-Secret`
header. Audit records use the constant `hermes-frontend` actor and never store
API secrets.

Older token and actor-id ADRs are superseded. `HERMES_LOCAL_API_TOKEN` and
`X-Hermes-Actor-Id` are historical compatibility terms, not the current
application auth boundary.

ADR-0052 defines the long-term capability runtime direction: backend application-layer policy checks, scoped capability grants, explicit confirmation for high-risk actions unless a scoped automation policy applies, and audit metadata for allowed and rejected high-risk decisions.

## Secrets

Secrets must never be hardcoded or committed. Provider tokens, passwords, app passwords, private keys and recovery material belong behind the secret resolver boundary.

ADR-0076 defines the current vault model. New secret payloads live in the dedicated host vault under `~/.hermes/vault`, backed by a local `vault.db` SQLite database. PostgreSQL stores only non-secret `secret_references`, provider account metadata and account-to-secret bindings.

Release runtime is macOS-only and uses Keychain for the master key. Docker/debug development may use `HERMES_DEV_MODE=true` with `HERMES_DEV_KEY_PATH`; release builds must reject dev storage. `HERMES_SECRET_VAULT_KEY` is legacy migration compatibility only and is not the normal runtime vault key.

Secret values must remain out of ordinary application tables, provider account config, event payloads, audit records, logs, tests and docs. Recovery phrases and recovery files are sensitive and must not be logged.

## AI Tool Safety

Agents may propose actions. Execution requires:

- declared tool capability
- user-visible intent
- permission check
- audit event
- rollback path where feasible

High-risk actions, such as sending messages, deleting data or changing external state, require explicit confirmation.

## Prompt Injection Defense

Imported messages and documents are untrusted input. The agent runtime must treat them as evidence, not instructions. Tools exposed to agents must be scoped, typed and permissioned.
