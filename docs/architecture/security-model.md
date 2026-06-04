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

Current implementation uses a temporary `HERMES_LOCAL_API_TOKEN` guard for local event API reads and writes until the full capability runtime exists.

Authorized local event API reads and writes require a non-secret `X-Hermes-Actor-Id` client identity header and are recorded in an append-only `api_audit_log` without storing API tokens or secrets.

## Secrets

Secrets must never be hardcoded or committed. Provider tokens, passwords, app passwords, private keys and recovery material belong in an OS-backed secret store or encrypted local vault.

Current implementation stores only non-secret `secret_references` metadata in PostgreSQL. Secret values must remain outside ordinary application tables.

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
