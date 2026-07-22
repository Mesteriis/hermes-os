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

The typed API contract, validation, runtime-side metadata adapter, durable
runtime ingress, admitted Unix client transport, Tauri emission, generated
provider client payloads and provider-local capability/read contract are
implemented. Runtime startup remains fail-closed until the Kernel-managed
identity and scoped Vault/storage credential handoff is wired. Tauri host
polling and acknowledgement now use the generated client response envelope;
JSON response fallback and fake provider-command acceptance are forbidden.
The host bridge also exposes a typed command-result relay carrying only
operation/provider request identity, success state, and observation time;
runtime reconciliation remains the sole owner of command lifecycle state.
Live WebView smoke evidence and actual provider command execution remain
migration work; no public availability is claimed until those gates are
present.
