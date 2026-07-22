# ADR-0243: Managed provider credential leases

Status: accepted for implementation

## Context

Managed integration runtimes need provider credentials such as Telegram API
hashes, bot tokens, OAuth refresh material, and provider session keys. The
current inherited control plane supports Event credentials and Storage
credentials, but it does not expose a generic provider-credential admission
contract. Passing provider secrets through environment variables, launch
arguments, runtime configuration, or the client port is forbidden.

## Decision

Provider credentials use a platform-owned, inherited managed-runtime control
operation. The runtime submits a typed request containing:

- request id and recipient HPKE public key;
- provider capability/purpose id;
- exact secret revision;
- allowed secret class and requested lease TTL.

Kernel accepts the request only when the managed runtime identity, runtime
generation, grant epoch, registration state, descriptor digest, and effective
approved capability all match. The requested purpose and secret revision must
be an exact match for the effective grant. Kernel never receives provider
plaintext and never interprets provider payloads.

The approved request is relayed to Vault as an encrypted `IssueLease` and
`ResolveLease` session. The runtime receives only an HPKE-encrypted response
bound to its runtime identity, grant epoch, request digest, and Vault
generation. A lease is single-use and invalidated by runtime restart, revoke,
grant epoch change, Vault generation change, or expiry.

Provider integrations own their typed purpose catalog and credential binding
semantics. Kernel owns admission and fencing; Vault owns secret storage and
lease issuance; the integration runtime owns plaintext handling and immediate
zeroization. No provider business logic enters Kernel or the generic control
plane.

## Consequences

- Telegram and WhatsApp can use the same managed credential boundary.
- Provider credentials are not representable in managed launch configuration.
- A provider runtime cannot request an arbitrary secret by inventing a purpose
  id.
- Existing Events and Storage credential routes remain specialized contracts.
- The next implementation slice must add the protobuf operation, inbound
  validation, Kernel grant resolver, and a reusable ciphertext-only runtime
  adapter before Telegram managed launch is admitted.

## Implementation state

Not implemented yet. Telegram runtime currently has typed local lease
validation and a Vault route wrapper, but its provider lease issuance remains
unwired until this platform contract is implemented.
