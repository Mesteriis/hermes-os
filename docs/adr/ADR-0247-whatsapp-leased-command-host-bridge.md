# ADR-0247: WhatsApp leased command host bridge

Status: accepted

## Decision

ADR-0242 remains the metadata-only inbound observation contract. This ADR adds
one separate, versioned result path for a command that the admitted WhatsApp
runtime has durably leased to an owner-controlled hidden Tauri WebView.

The runtime owns the command queue. It persists exact typed command bytes,
grants a bounded single-host claim lease, and accepts completion only when the
account, operation ID, host claim ID and unexpired lease all match. A host
cannot invent an operation, complete another host's operation, or turn a stale
lease into evidence.

Completion atomically transitions the owner-local command state and appends its
provider-neutral `DeliveryStateChanged` observation to the WhatsApp outbox.
Communications receives only the durable event. It does not receive a host
claim, provider command, browser state, or WebView implementation.

The host receives provider command payload only for a currently leased
operation. It may return success/failure and an optional provider request ID.
It cannot return cookies, browser storage, inbound message body, media bytes,
or arbitrary observations through this result path.

## Implementation state

The durable queue and claim lease exist. The result contract is implemented by
the admitted WhatsApp runtime; Kernel-to-Tauri route delivery and the bounded
WebView executor remain separate implementation work. No public availability
is implied before those paths have live evidence.
