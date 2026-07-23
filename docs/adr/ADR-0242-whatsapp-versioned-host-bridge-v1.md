# ADR-0242: WhatsApp versioned host bridge v1

Status: accepted

## Decision

The hidden Tauri WebView remains host-owned. Its only provider-to-runtime
submission surface is the typed `WhatsAppHostBridgeEnvelopeV1` contract in
`hermes-whatsapp-api::host_bridge`.

The bridge carries sanitized provider observation metadata only. Cookies,
local storage, IndexedDB, session material, message bodies, media bytes and
command completion are forbidden. Host code does not decide business state or
invoke domain commands; it forwards observations to the admitted WhatsApp
runtime transport.

The envelope is versioned by exact protocol major/revision and includes
account, provider event identity and observed time. The old loopback HTTP relay
is a migration adapter only and cannot be treated as the v1 contract.

## Implementation state

The typed API contract, exact route-binding handshake and runtime-side durable
metadata ingress exist. Kernel publishes an owner-private route descriptor
only for the lifetime of its admitted runtime; Tauri consumes it natively and
returns only fenced attachment state. The host executor emits only native-derived
`host_route_attached` and `webview_loaded` lifecycle observations through the
exact admitted route. The remote WebView has no relay payload and cannot select
an account, event ID, timestamp, state, command, or observation content.

The host executor intentionally has no provider-DOM relay, polling or
acknowledgement command yet. JSON fallback and fake provider-command acceptance
are forbidden. Live WebView smoke evidence, a metadata-only provider DOM
extractor, and actual provider command execution remain migration work; no
public availability is claimed until those gates are present.
