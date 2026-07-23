# ADR-0254: Communications derived search index and private-content boundary

Статус: Принято
Дата: 2026-07-23
Состояние реализации: typed public search contract, pure owner-local domain
lifecycle/normalization и owner-local PostgreSQL digest projection реализованы.
`communications.search.index.v1` capability с exact owner-derived-key purpose
declared, а runtime-only adapter для bounded private Blob read и zeroized Vault
owner-key lease реализован. Durable replay/rebuild, query execution, Gateway
route и managed conformance ещё не реализованы; до них query fail-closed as
unavailable. Legacy full-text HTTP search не переносится.

Зависит от:

- [ADR-0220: durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0223: Vault leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0230: Blob boundary](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0231: private Blob session](ADR-0231-private-blob-data-session-and-vault-route.md);
- [ADR-0240: Communications owner](ADR-0240-canonical-communications-owner-clean-room-migration.md);
- [ADR-0253: legacy surface disposition](ADR-0253-communications-legacy-surface-disposition-and-clean-room-completion.md).

## Context

The canonical Communications owner stores source identity, typed state and an
opaque admitted `BlobRefV1`; it deliberately does not store provider subjects,
message body or snippets in PostgreSQL. Historical full-text search cannot be
copied because it would put raw provider content in events, generic metadata or
another owner’s storage.

Search is derived and rebuildable, never canonical truth. It must also work
without a provider-specific branch in the Communications domain.

## Decision

Communications search is an owner-local derived projection with this pipeline:

```text
typed observation with admitted Blob reference
  -> Communications canonical evidence transition
  -> owner-local index request (evidence/blob reference only)
  -> bounded private Blob read by Communications runtime
  -> deterministic token normalization
  -> keyed token digest projection in Communications PostgreSQL
  -> typed owner query result containing canonical IDs only
```

The index request and completion event carry only canonical IDs, projection
revision, result status and correlation/causation. They never carry plaintext,
provider payload, URL, path, content hash, token, snippet or filename.

The runtime obtains a dedicated Communications search-key lease from Vault with
an exact purpose, runtime generation and grant epoch. For every normalized
token it stores a deterministic keyed digest, not the token plaintext. Query
text is accepted only through the typed public Communications query contract,
normalized in the owner runtime and compared by keyed digest. The client never
receives a database query, index key or Blob capability.

The first search profile is deliberately bounded:

- exact normalized token matching only;
- bounded byte read and token count per Blob;
- typed attachment filename metadata may be indexed through the same digest
  path;
- result contains canonical evidence/message/conversation IDs and fixed typed
  ranking metadata only;
- phrase, prefix, fuzzy, semantic and cross-owner search each require a later
  ADR and explicit privacy/performance evidence.

An unavailable Blob, denied lease, invalid content, exhausted limit or stale
fence records a typed owner-local projection failure. It does not retry through
a provider, expose content in telemetry or turn missing content into a false
empty result.

## Boundary rules

- Integration packages continue to publish only `hermes-communications-ingress`
  observations and exact durable envelopes. They do not tokenize, index or
  query canonical Communications state.
- Communications may read only its own admitted Blob reference through the
  bounded private Blob protocol. It does not mount Blob storage or access an
  integration session store.
- No domain imports the search implementation. Cross-owner search is a future
  use-case workflow over public owner query contracts, never a shared index.
- PostgreSQL index tables are owned by Communications and carry no provider
  identifier beyond canonical provenance already owned by the evidence model.
- Index rows are deleted or rebuilt after canonical deletion, Blob revocation,
  owner revoke, grant epoch change or index schema/key revision change.

## Required implementation slice

The implementation must add independent responsibilities, not a large search
service:

1. versioned typed API request/response and exact limits;
2. owner-local domain validation and index lifecycle decision;
3. private Blob reader adapter and Vault search-key lease adapter in runtime;
4. owner-local persistence adapter for digests and projection status;
5. durable event/replay handler and rebuild operation;
6. generated Gateway route and live managed conformance.

Each package remains inside the existing Communications API/domain/persistence/
runtime split. No integration, Kernel or Gateway package gains a search
implementation dependency.

## Acceptance evidence

Before this profile is marked implemented, tests must prove:

1. plaintext, provider DTOs and generic metadata do not enter durable events,
   PostgreSQL index rows, logs, errors or client result payloads;
2. a permitted owner runtime indexes bounded admitted content and returns only
   matching canonical IDs;
3. replay is idempotent and a rebuild from canonical evidence/Blob state gives
   the same digest projection;
4. stale/revoked Blob, Vault, runtime or grant fences fail closed;
5. Mail, Telegram, WhatsApp and Zulip retain only ingress/event edges;
6. the managed Storage/NATS/Vault contour proves the public owner query path.

## Consequences

Communications receives a search capability without becoming a provider
facade, raw-content database or global cross-domain index. The historical
search endpoint remains historical evidence until this independently verifiable
clean-room path exists.
