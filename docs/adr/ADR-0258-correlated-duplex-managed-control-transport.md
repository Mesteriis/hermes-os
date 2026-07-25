# ADR-0258: Correlated duplex managed-control transport

Статус: Принято
Дата: 2026-07-24
Состояние реализации: В работе. Correlated frame/channel, Kernel V2 pump,
Communications runtime, Blob/owner-key clients и provider-credential resolve
client реализованы. Telegram и Mail runtimes переведены на один V2 frame pump
вместе с descriptor/ready, Storage/Vault, provider credential, Event и
Blob/client delivery operations. Mail live managed conformance доказывает
correlated nested client delivery, Kernel-issued leases, event publication,
outage replay и revoke fencing без клонирования inherited control FD. Zulip и
остальные выбранные managed runtimes ещё должны быть переведены атомарно с их
signed descriptors; Telegram всё ещё требует полного live concurrency
conformance. До этого ADR не считается полностью реализованным.

Зависит от:

- [ADR-0200: clean-room module model](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0219: managed distribution integrity](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: durable envelope and contract evolution](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: module descriptor and lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0256: owner-declared client RPC route admission](ADR-0256-owner-declared-client-rpc-route-admission.md);
- [ADR-0257: event-backed Blob custody transfer](ADR-0257-event-backed-blob-custody-transfer-for-canonical-evidence.md).

## Context

The inherited managed-control FD is a private Kernel/runtime transport. It
carries both directions: a runtime asks Kernel for a narrowly typed platform
operation (for example a Vault route or Blob session), while Kernel may deliver
an owner-declared client request to the runtime. `ManagedRuntimeControl*V1`
identifies an operation but has no transport correlation ID. A synchronous
reader can therefore consume a valid frame belonging to the opposite direction
while it is waiting for its own response. Field-tag peeking or a
Communications-only deferred queue does not establish ownership, ordering or
failure semantics and would turn one domain into a transport exception.

## Decision

Managed control evolves to one versioned, correlated duplex transport owned by
the runtime platform. Its frame has exactly these transport fields:

- a non-zero, fixed-size opaque correlation ID created by the requester;
- an exhaustive typed request or typed response union; and
- a bounded protocol version and frame-size limit.

No generic byte payload, `Any`, map, provider data, business payload or secret
is introduced. Existing operation messages remain the semantic contracts; the
new frame supplies transport identity only.

Each endpoint owns a single frame pump for its inherited FD. While awaiting a
response, it dispatches a complete opposite-direction request through the
registered typed handler and writes a response with that request's correlation
ID. It delivers a response only to the pending request with the same ID.
Unknown, duplicate, expired, zero or oversized correlation IDs are terminal
protocol failures for that managed session. Pending requests and deferred frame
queues are bounded; timeout, runtime exit, revoke and generation change fail
all pending requests without retrying a non-idempotent operation implicitly.
Every accepted request receives its correlated typed response; one-way lifecycle
signals use the explicit `ManagedRuntimeControlAckV1`, never an empty frame,
repurposed `describe` result or implicit socket ordering.

The Kernel owns the server-side pump and lifecycle fencing. The reusable
platform runtime-control package owns frame encoding, validation and client
call mechanics. A domain runtime supplies only its typed owner request handler;
it neither inspects control tags nor owns a transport queue. Integrations,
domains and workflows do not import each other to make this work.

`describe` and `ready` remain explicit lifecycle messages. They use the same
versioned transport after the managed endpoint has been selected, and readiness
is not inferred merely because a process accepted a socket.

## Migration and compatibility

This is an atomic managed-runtime protocol cut. Kernel must select one exact
control transport version from the signed descriptor/launch binding before the
child is started; it must never guess a frame version or mix V1 and correlated
frames on one FD. Every bundled managed binary in the selected inventory is
rebuilt and re-pinned with the matching descriptor and executable digest.
External modules stay `pending` until their separately approved contract
supports the selected version.

The temporary V1 correction in this slice routes Vault requests through the
existing typed `route_vault_ciphertext` oneof, eliminating its collision with
`describe`. It is a compatibility repair, not the duplex solution and does not
authorize V1 frame-tag dispatch as an enduring pattern.

## Required implementation and evidence

1. Add the correlated frame and fail-closed validation to the platform runtime
   protocol, including bounded pending state and correlation-ID tests.
2. Implement the Kernel and managed-runtime frame pumps with typed dispatch;
   remove raw tag inspection and domain-local control-frame mailboxes.
3. Migrate all bundled platform and admitted owner runtimes atomically through
   signed manifest/descriptor bindings; reject an unpinned or mixed endpoint.
4. Prove concurrent opposite-direction Blob, Vault, event-credential,
   owner-key and client-delivery flows without loss, cross-request response or
   deadlock.
5. Prove stale generation, revoked grant, malformed/duplicate correlation and
   bounded-queue failures close the session without leaking payloads or
   credentials.

## Consequences

Communications retains an owner-local custody worker and query handler, but it
does not become the owner of IPC scheduling. The same transport rule applies to
Storage, Blob, Vault, Scheduler, Event Hub and every future admitted domain.
The first-owner inventory and its capabilities are unchanged; this ADR changes
only the private managed-control transport needed to operate that inventory
correctly.
