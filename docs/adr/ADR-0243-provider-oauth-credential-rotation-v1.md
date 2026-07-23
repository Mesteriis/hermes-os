# ADR-0243: Provider OAuth credential rotation v1

Status: accepted

## Decision

OAuth authorization-code exchange and refresh are provider-integration
operations. A provider adapter may exchange or refresh a token, but it never
writes a secret to its settings, database, Communications, or another domain.

The admitted integration obtains a single action-specific Vault lease through
the Kernel. `Resolve`, `Create`, and `ReplaceCas` are distinct requests. The
Kernel issues an action only when the admitted module descriptor declares the
same purpose, secret class, target scope, action and bounded lease TTL under an
owner-approved capability. The Vault record ID returned by create or replace
is opaque integration-owned binding metadata; it is not a secret and is never
published in events.

An OAuth refresh credential uses `OAuthRefreshCredential`; a current access
token uses `ProviderCredential`. Credential revision and the corresponding
opaque record ID are integration-owned durable binding state. They are not
Kernel settings and are not Communications state. A replace advances exactly
one revision and uses the prior opaque record ID for CAS.

The Kernel stays provider-neutral. It does not know OAuth endpoints, client
IDs, scopes, refresh payloads, mail accounts, or Communications contracts.

## Implementation state

The runtime control contract carries one explicit Vault action. Kernel
authorization verifies that action against the descriptor-declared purpose,
and the managed Vault client can execute one encrypted `StoreLease` or
`ReplaceLease` after the matching action lease is issued. The Gmail adapter
implements typed HTTPS authorization-code exchange and refresh requests.

Mail descriptor distribution, Mail-owned OAuth binding persistence, and the
admitted Mail setup/refresh workflow are not implemented yet. No setting,
provider-local secret store, or fallback access-token path is accepted as a
substitute.
