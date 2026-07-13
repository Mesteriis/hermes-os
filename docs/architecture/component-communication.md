# Component Communication Contract

Status: Current
Date: 2026-06-20

Canonical policy:

- ADR: `docs/adr/ADR-architecture-communication-contract.md`
- Executable contract: `scripts/architecture-contract.json`
- Guard: `scripts/check-architecture.mjs`

## Interaction Kinds

- `direct_call`: same-component or allowed layer-local function call.
- `command_port`: explicit write/change API owned by a domain or runtime.
- `query_port`: explicit read API owned by a domain or runtime.
- `event`: durable asynchronous fact or intent.
- `projection`: idempotent materialized read model derived from owned facts.
- `runtime_integration_api`: provider setup/runtime control API.

## Backend

`app/` composes HTTP and calls ports or explicit public workflow APIs. It does
not import concrete domain stores or provider runtime internals. `domains/*` own business truth.
`integrations/*` observe and operate providers. `workflows/*` coordinate domains
through ports/events. `engines/*` compute neutral projections or candidates.
`ai/*` suggests; it does not decide. `platform/*` is technical substrate.
`vault/*` owns secrets and session material.

No baseline or per-file exception is allowed. If a dependency breaks the
contract, move the behavior to the owning layer or introduce an explicit
command/query/event contract.

## Frontend

Domain views and stores are isolated. Cross-domain screens belong in
`frontend/src/app` or use backend-provided review/workspace read models.

Provider business caches are communication caches:

```ts
['communications', provider, ...]
```

Provider runtime caches are integration caches:

```ts
['integrations', provider, 'runtime', ...]
```

Direct provider roots are forbidden:

```ts
['telegram', ...]
['whatsapp', ...]
['mail', ...]
```
