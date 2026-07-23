# ADR-0241: WhatsApp clean-room provider boundary

Status: accepted; supersedes the WhatsApp restriction in ADR-0212

## Decision

WhatsApp is an independent integration owner. Its API, lifecycle policy,
provider projections, persistence and runtime packages do not import Telegram,
Mail, a business domain, Gateway implementation or WebView implementation.

The WhatsApp runtime communicates with the external provider only through an
owner-local `WhatsAppProviderTransport` port. The concrete hidden WebView and
its browser/session state remain host-owned. Rust runtime code never evaluates
provider page scripts, stores cookies, or invents a provider-neutral business
model.

This decision explicitly opens the versioned `host_bridge_v1` seam that ADR-
0212 previously required before adding backend WhatsApp packages. It does not
move the WebView implementation into `backend/` and does not admit the legacy
unversioned HTTP relay as the final contract.

Provider commands are validated in `hermes-whatsapp-api`, operation lifecycle
is owned by `hermes-whatsapp-core`, projections and operation state are owned by
`hermes-whatsapp-persistence`, and orchestration is owned by
`hermes-whatsapp-runtime`. Provider observations enter through typed events and
are not promoted directly to durable business entities.

## Implementation state

The versioned API and host bridge contracts, metadata-only core policy,
owner-local durable observation/outbox persistence, typed client port and
managed identity/storage bootstrap now exist. Provider projections beyond the
metadata-only evidence boundary, Kernel launch wiring, command execution and
live WebView execution remain open migration work. No database URL environment
variable or provider secret handoff is admitted.

The backend API/core/persistence/runtime packages remain an independent
WhatsApp integration build unit. "Host-owned" applies only to browser/WebView
execution and session state; it does not prohibit the integration's typed
runtime, owner-local durable queue or event outbox. Backend WhatsApp packages
must not depend on Tauri, Wry, WebKit or WebView runtimes.
