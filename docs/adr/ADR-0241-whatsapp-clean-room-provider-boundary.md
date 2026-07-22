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

The clean-room API contract, core operation policy, provider projections,
PostgreSQL-owned durable schema, runtime transport port, typed client IPC,
versioned host-bridge contract/metadata adapter, provider-local capability
catalog, admitted Unix client transport, Tauri host emission, generated
provider client payloads and the managed identity/storage-lease bootstrap are
implemented, including a generated client response envelope with typed
account, lifecycle, query, message, dialog, participant, runtime-status,
realtime, command and event response decoding.
Kernel launch wiring still has to pass the non-secret storage
binding/topology contract to this bootstrap; a database URL environment
variable is not an allowed alternative. Live provider execution through the
host-owned WebView remains migration work and must not be represented as
complete.
