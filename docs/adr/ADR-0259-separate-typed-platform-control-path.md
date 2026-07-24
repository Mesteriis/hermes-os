# ADR-0259: Separate typed platform-control path

Статус: Принято
Дата: 2026-07-24
Состояние реализации: В работе. ADR-0258 correlated managed-control transport
реализован как platform primitive, но active endpoint migration ещё не
завершена. Existing raw platform relay remains legacy transport until the
separate typed path is admitted atomically.

Зависит от:

- [ADR-0200: clean-room module model](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0212: crate topology and compile isolation](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0219: managed distribution integrity](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0258: correlated duplex managed-control transport](ADR-0258-correlated-duplex-managed-control-transport.md).

## Context

The inherited managed-control FD currently carries two unrelated kinds of
traffic: owner-neutral runtime lifecycle/capability operations and platform
owner control requests such as Storage, Blob, Scheduler, Vault, Event Authority
and Telemetry status/control contracts. Storage's control contract belongs to
the separate Storage Protocol package, which already depends on Runtime
Protocol. Adding every platform request into Runtime Protocol would create a
dependency cycle and turn Runtime Protocol into an aggregation facade.

Opaque relay bytes avoid the cycle only by hiding contract identity and
correlation. That is not an acceptable V2 transport contract.

## Decision

Each managed platform process receives two exact inherited private channels:

1. **managed-control V2** is owner-neutral and carries only descriptor,
   lifecycle, explicit capability operations and owner-declared client delivery
   through correlated typed frames from ADR-0258;
2. **platform-control** is a separate, process-specific, typed contract owned
   by the platform package that defines it. It carries no domain request,
   provider data, generic map, `Any` or opaque relay payload.

Kernel creates, fences and supervises both channels but does not translate one
contract into another. The exact platform process ID, executable binding,
runtime generation and grant epoch bind both inherited descriptors. A managed
platform child rejects a missing, extra, stale or cross-process control FD.

The process-specific platform-control contract may use its existing owner
protocol package (for example Storage Protocol) without Runtime Protocol
depending on it. A non-platform domain or integration cannot receive a
platform-control FD. Communications keeps only managed-control V2 and its
owner-local platform clients; it never receives Storage, Blob or provider
implementation control surfaces.

## Migration

The legacy raw relay is removed only after each platform child has an exact
typed platform-control descriptor and Kernel validates the two-FD launch
binding. V2 managed-control and platform-control migration is atomic per
signed platform inventory; no process may accept V1 and V2 on the same FD.
The current endpoint handshake edits are preparation only and do not prove the
new topology.

## Required evidence

1. Cargo boundaries prove Runtime Protocol does not depend on Storage or any
   platform implementation protocol.
2. Kernel launch validation pins both FDs to the exact signed child binding.
3. Each platform runtime rejects a wrong/missing/stale platform-control FD.
4. Communications architecture tests prove it has neither a platform-control
   FD nor a dependency on platform implementation packages.
5. Concurrent managed-control client delivery and platform-control status work
   without cross-frame consumption, opaque relay or a generic proxy.

## Consequences

The transport boundary now follows assembly ownership: a domain runtime owns
its domain process; a platform process owns its platform control contract; and
Kernel supervises both without becoming an application facade. The additional
FD is intentional capability separation, not a backchannel between domains.
