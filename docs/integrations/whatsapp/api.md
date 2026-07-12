# WhatsApp Integration API

Status: current repository API surface for the WhatsApp integration runtime and
fixture foundation.
Date: 2026-06-26.

This document describes the currently implemented backend routes under:

```text
/api/v1/integrations/whatsapp/*
```

These routes are integration/runtime surfaces. Provider-neutral user-facing
writes and reads belong under:

```text
/api/v1/communications/*
```

## Runtime and capability routes

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/api/v1/integrations/whatsapp/capabilities` | Provider-family capability matrix. |
| `GET` | `/api/v1/integrations/whatsapp/accounts` | List WhatsApp integration accounts; `include_removed=true` includes logically removed entries. |
| `GET` | `/api/v1/integrations/whatsapp/accounts/{account_id}/capabilities` | Account-scoped capability overlay. |
| `POST` | `/api/v1/integrations/whatsapp/accounts` | Create blocked live account for explicit provider shape. For `whatsapp_business_cloud`, `api_access_token` is required and is stored only in host vault as `whatsapp_business_cloud_access_token`; optional `app_secret` and `webhook_verify_token` are also stored only in host vault for webhook signature/challenge verification. PostgreSQL stores metadata and binding refs only. |
| `GET` | `/api/v1/integrations/whatsapp/runtime/status` | Runtime lifecycle/account status. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/start` | Start runtime or return blocked-safe lifecycle state. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/stop` | Stop runtime or return blocked-safe lifecycle state. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/revoke` | Revoke linked runtime/session state. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/relink` | Move account back to relink-required state. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/rotate` | Alias for relink-safe session migration/rotation into link-required state. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/remove` | Logical runtime/account removal surface. |
| `GET` | `/api/v1/integrations/whatsapp/runtime/health` | Sanitized runtime health summary with `available` / `degraded` / `blocked` status plus nested `session`, `storage`, `runtime`, `webview` and `validation` diagnostics. |
| `POST` | `/api/v1/integrations/whatsapp/login/qr/start` | Start QR link flow when provider shape supports it. |
| `POST` | `/api/v1/integrations/whatsapp/login/pair-code/start` | Start pair-code flow when provider shape supports it. |

## Provider sync and listing routes

For `whatsapp_web_companion`, `runtime/health` also exposes
`checks.web_companion_bridge` and `checks.runtime.web_companion_bridge`. This is
a contract manifest, not a live availability flag. It requires an owner-visible
desktop WebView, forbids hidden/headless mode, binds restorable session state to
the host-vault `whatsapp_web_session_key`, lists the protected
`/runtime-bridge/*` event routes the producer must use, fixes provider writes to
the durable outbox claim/failure paths and excludes session material, cookies,
browser profile secrets, QR/pair-code artifacts, message bodies and media bytes
from health/event-like payloads.

The desktop shell also exposes Tauri commands
`open_whatsapp_web_companion` and `whatsapp_web_companion_manifest`. They are
local shell commands, not backend HTTP API. The opener creates or focuses an
owner-visible `https://web.whatsapp.com/` companion WebView and returns a
sanitized manifest containing only runtime-bridge paths, command-channel policy
and secret-storage policy. It does not read or return cookies, session material,
browser profile secrets, message bodies or media bytes. The WhatsApp Runtime
panel exposes this path through its owner-visible `Open Companion` action. The
visible shell is still blocked for public availability until manual smoke
passes.
Frontend code uses `openWhatsappWebCompanion` and
`getWhatsappWebCompanionManifest` from the integration API layer to invoke
these Tauri commands directly; this bridge deliberately does not use
`ApiClient` or backend HTTP routes.

The companion window installs a main-frame-only initialization script guarded to
`https://web.whatsapp.com`, and navigation is constrained to that origin. The
script exposes a frozen `__HERMES_WHATSAPP_COMPANION__` metadata contract plus a
local DOM readiness event and an allowlisted metadata-only relay dispatch. It
does not read cookies, Web Storage, IndexedDB, browser profile secrets, session
material, message bodies or media bytes, and it does not call `fetch`, XHR,
`postMessage` or domain APIs. Runtime health and the frontend manifest report
this as `contract_injected_relay_dispatch_available` with
`tauri_allowlisted_companion_runtime_bridge_dispatch`. The Tauri relay command
posts sanitized metadata observations to the protected local
`/api/v1/integrations/whatsapp/runtime-bridge/runtime-events` route using
`X-Hermes-Secret` from the Tauri process environment only. This creates
runtime-event evidence but does not project typed messages/status/media until a
richer typed WebView payload exists.

For `whatsapp_native_md`, `runtime/health` also exposes
`checks.native_md_driver.provider_command_surface.wa_rs_sdk_command_gap` and
the same block under `checks.runtime.native_driver`. This is evidence metadata,
not live availability. It names the verified `wa-rs 0.2.0` public SDK methods
used by the smoke-gated subset, including the forwarded-text reemit contract for
`forward`, and names the missing safe write APIs for `publish_status`,
dialog-state writes, `mark_unread` and join-by-invite. Commands in that gap may
only produce structured
terminal dead-letter evidence with `native_md_command_kind_unsupported`; they
must not complete until a provider-observed event reconciles them. The native
executor performs this unsupported-command preflight before the smoke gate and
runtime driver lookup, so missing SDK/API support is not reported as a transient
runtime-not-running condition and is not retried as if the provider might later
accept it.

For native media uploads, `send_media` / `send_voice_note` provider submissions
include a sanitized `provider_observed_completion_target`. The command may only
complete after accepted `signal.accepted.whatsapp.media` evidence observes the
provider message id returned by `wa-rs::Client::send_message`; fixture blob-id
matching remains a fallback for deterministic tests. Raw bytes, media keys,
direct paths and provider URLs are not part of the target payload.

For Business Cloud, Graph `send_text`, `send_template`, `send_media` and
`send_voice_note` submissions also include a sanitized
`provider_observed_completion_target`. The target points at accepted
`signal.accepted.whatsapp.receipt` evidence from webhook `statuses[]`; webhook
status `id` must match the stored Graph message id/provider request id before
the durable outbox command can complete. Access tokens, raw provider payloads,
template components and media bytes are excluded from the target payload.

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/chats` | Fixture-backed/provider-runtime chat sync control surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/history` | Fixture-backed/provider-runtime history sync control surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/conversations/{provider_chat_id}/members` | Fixture-backed/provider-runtime roster sync control surface backed by canonical participant projection. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/statuses` | Fixture-backed/provider-runtime status-feed sync control surface backed by canonical projected `status-feed` messages. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/presence` | Fixture-backed/provider-runtime presence sync control surface backed by canonical identity presence metadata; optional `provider_chat_id` scopes the snapshot to one chat. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/calls` | Fixture-backed/provider-runtime call sync control surface backed by the shared calls read model; optional `provider_chat_id` scopes the snapshot to one chat. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/contacts` | Fixture-backed/provider-runtime contact sync control surface backed by canonical communication identities plus compatibility WhatsApp/phone trace metadata. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/media` | Fixture-backed/provider-runtime media sync control surface backed by canonical communication attachments and local blob metadata; optional `provider_chat_id` and `content_type` filter the snapshot. |
| `GET` | `/api/v1/integrations/whatsapp/sessions` | Provider session list. Accountless listing aggregates across provider shapes. |
| `GET` | `/api/v1/communications/messages?channel_kind=whatsapp` | Provider-neutral WhatsApp message list. Accountless listing aggregates across provider shapes through Communications projections. |

## Provider command routes

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/send` | Queue send-text provider command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/reply` | Queue reply provider command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/forward` | Queue forward provider command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/edit` | Queue edit provider command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/delete` | Queue delete provider command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/reactions` | Queue add-reaction provider command. |
| `DELETE` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/reactions` | Queue remove-reaction provider command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-media/upload` | Queue media upload/send command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-media/download` | Queue media download command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/voice-note` | Queue voice-note send command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/statuses/publish` | Queue status publish command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/join` | Queue join-group command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/leave` | Queue leave-group command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/read` | Queue mark-read command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/unread` | Queue mark-unread command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/archive` | Queue archive command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/unarchive` | Queue unarchive command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/mute` | Queue mute command. |

Media upload and voice-note routes require a locally imported `attachment_id` with a `clean` scan verdict. They reject raw bytes and a bare `blob_id`; when a `blob_id` is supplied with an attachment, Hermes validates that both references match.
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/unmute` | Queue unmute command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/pin` | Queue pin command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/unpin` | Queue unpin command. |
| `GET` | `/api/v1/integrations/whatsapp/commands` | List durable provider commands. |
| `POST` | `/api/v1/integrations/whatsapp/commands/{command_id}/retry` | Manual retry transition. |
| `POST` | `/api/v1/integrations/whatsapp/commands/{command_id}/dead-letter` | Manual dead-letter transition. |

`whatsapp_business_cloud` does not use the personal WhatsApp provider shape as
a personal-chat substitute. Its smoke-gated official Business Platform
submission subset is represented as `business.messages.send_text`,
`business.templates` and `business.media_endpoints`, and is executed from
durable outbox rows only when the account is explicitly configured with:

```json
{
  "runtime": "business_cloud_smoke",
  "business_cloud_live_smoke_enabled": true,
  "business_cloud_phone_number_id": "<phone-number-id>",
  "business_cloud_graph_api_version": "v24.0"
}
```

The worker reads `whatsapp_business_cloud_access_token` from host vault into
process memory only. `send_text` and `send_template` submit through the Graph
messages endpoint shape. `send_media` and `send_voice_note` read local blob
bytes in worker memory, validate size and SHA-256, upload through the Graph
media endpoint, then send the returned media id through the Graph messages
endpoint. Result metadata is sanitized: access tokens, raw provider payloads,
template component payloads, media bytes, captions and filenames are excluded.
Command completion still waits for provider-observed webhook/event
reconciliation; the command stores the Graph message id as a sanitized
provider-observed receipt target, and webhook status evidence reconciles by
matching that id through the Signal Hub receipt path.
Provider failures stay in the durable command lifecycle: HTTP 429 maps to
`business_cloud_rate_limited`, `Retry-After` is persisted as
`retry_after_seconds`, provider error code/type are sanitized into failure
metadata, and the command is retried or dead-lettered by the existing outbox
policy.

Cloud webhook evidence enters through the runtime-bridge contract, not through
domain services. The local bridge accepts Meta-like payloads with
`entry[].changes[].value.messages[]` and `entry[].changes[].value.statuses[]`.
Account resolution uses `_hermes.account_id` / `account_id` when present, or the
webhook `metadata.phone_number_id` against linked `whatsapp_business_cloud`
accounts. Text messages are normalized into live message evidence; delivery
statuses `sent`, `delivered`, `read` and `played` are normalized into receipt
evidence for provider-observed command reconciliation. Unsupported message
types, failed statuses and missing required fields are preserved as sanitized
degraded runtime evidence instead of being dropped.

Webhook security is account-scoped and host-vault backed:

- `GET /api/v1/integrations/whatsapp/runtime-bridge/business-cloud/webhooks`
  validates `hub.mode=subscribe`, `hub.verify_token` and `hub.challenge` against
  the account-scoped `whatsapp_business_cloud_webhook_verify_token` secret and
  returns the challenge on success.
- `POST /api/v1/integrations/whatsapp/runtime-bridge/business-cloud/webhooks`
  verifies `X-Hub-Signature-256: sha256=<hex>` against the raw request body with
  the account-scoped `whatsapp_business_cloud_app_secret` before parsing evidence
  into the Signal Hub path.
- `GET /api/v1/integrations/whatsapp/runtime-bridge/business-cloud/proxy-manifest`
  returns the protected edge/proxy forwarding contract: local path, required
  query params and headers, raw-body forwarding policy, secret-purpose names and
  per-account readiness booleans. It reports binding state only and does not
  read or return host-vault secret values.
- The route is still part of the ADR-0056 protected local API. A production
  public Meta webhook must terminate at an explicit local proxy/edge bridge that
  can call Hermes with `X-Hermes-Secret`; Hermes itself is not opened as a public
  unauthenticated endpoint by this contract.

### Business Cloud edge proxy binary

The repository includes a standalone local edge bridge binary:

```text
hermes-whatsapp-business-cloud-edge-proxy
```

Its public surface is intentionally smaller than Hermes:

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/healthz` | Local proxy process health. |
| `GET` | `/readyz` | Protected readiness check against Hermes proxy manifest using `X-Hermes-Secret`. |
| `GET` | `/manifest` | Static proxy behavior manifest; never returns secret values. |
| `GET` | `/webhooks/whatsapp/business-cloud` | Forward Meta challenge query params to the protected Hermes webhook route. |
| `POST` | `/webhooks/whatsapp/business-cloud` | Forward the exact raw webhook body plus `X-Hub-Signature-256` to the protected Hermes webhook route. |

Configuration:

- `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND_ADDR` defaults to
  `127.0.0.1:8787`.
- `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_BASE_URL` defaults to
  `http://127.0.0.1:8080`.
- `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_SECRET` is preferred for local
  Hermes auth; `HERMES_LOCAL_API_SECRET` is accepted as the development
  fallback.

Docker/Makefile packaging:

- `make whatsapp-business-cloud-edge-readiness` runs a fast static preflight for
  the edge proxy binary, Compose profile, docs and public/protected forwarding
  contract. With `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_PROBE=1`, it also probes a
  locally running proxy `/healthz`, `/manifest` and unsigned POST rejection.
- `make whatsapp-business-cloud-edge-config` validates the opt-in Compose
  profile without starting it.
- `make whatsapp-business-cloud-edge-up` builds and starts the dedicated edge
  proxy container.
- `make whatsapp-business-cloud-edge-stop` stops only the edge proxy service.
- `make whatsapp-business-cloud-edge-logs` tails only the edge proxy service.

The Compose profile is named `whatsapp-business-cloud-edge`, publishes only the
edge proxy port and keeps Hermes itself non-public. The default bind in
`docker/.env.example` is loopback (`127.0.0.1`) so public ingress must be an
explicit operator decision.
- `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_ACCOUNT_ID` is optional and is appended
  only to forwarded GET challenge requests when the public proxy is
  account-scoped. It is not added to `/readyz` manifest checks or POST webhook
  delivery.

The proxy does not parse or rewrite webhook JSON. POST bodies are forwarded
byte-for-byte, the Meta signature is forwarded as metadata, and Hermes performs
the host-vault-backed challenge/signature validation plus Signal Hub ingestion.
The proxy injects `X-Hermes-Secret` only on its local Hermes request, sanitizes
upstream failures and does not read host-vault secrets. Unsigned POST webhook
requests are rejected by the proxy before they can reach Hermes.

## Fixture ingest routes

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/api/v1/integrations/whatsapp/fixtures/accounts` | Create fixture account/session foundation. `provider_shape` may optionally pin `whatsapp_native_md` or `whatsapp_business_cloud`; `whatsapp_native_md` still uses compatibility `provider_kind = whatsapp_web`. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/messages` | Ingest fixture message evidence. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/message-updates` | Ingest fixture message-update evidence. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/message-deletes` | Ingest fixture message-delete evidence. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/receipts` | Ingest fixture receipt evidence. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/dialogs` | Ingest fixture dialog evidence. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/participants` | Ingest fixture participant evidence. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/reactions` | Ingest fixture reaction evidence. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/media` | Ingest fixture media metadata evidence. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/statuses` | Ingest fixture status evidence. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/status-views` | Ingest fixture status-view evidence. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/status-deletes` | Ingest fixture status-delete evidence. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/presence` | Ingest fixture presence evidence. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/calls` | Ingest fixture call metadata evidence. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/runtime-events` | Ingest fixture runtime-event evidence. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/sessions/authorized` | Persist authorized session credential into host vault binding. |

## Runtime bridge routes

These routes are the live-runtime bridge surface for an external WhatsApp
runtime process. They reuse the same typed ingest contracts and event-spine
projection path as fixture ingest, but are not fixture/dev-only endpoints.

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/messages` | Ingest provider-observed live message evidence. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/message-updates` | Ingest provider-observed live message-update evidence. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/message-deletes` | Ingest provider-observed live message-delete evidence. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/receipts` | Ingest provider-observed live receipt evidence. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/dialogs` | Ingest provider-observed live dialog evidence. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/participants` | Ingest provider-observed live participant evidence. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/reactions` | Ingest provider-observed live reaction evidence. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/media` | Ingest provider-observed live media metadata evidence. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/media-lifecycle` | Ingest live media upload/download lifecycle phases (`requested`, `started`, `progress`, `completed`, `failed`) into sanitized `whatsapp.media.*` and accepted runtime-event evidence. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/statuses` | Ingest provider-observed live status evidence. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/status-views` | Ingest provider-observed live status-view evidence. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/status-deletes` | Ingest provider-observed live status-delete evidence. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/presence` | Ingest provider-observed live presence evidence. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/calls` | Ingest provider-observed live call metadata evidence. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/runtime-events` | Ingest live runtime lifecycle/health events. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle` | Ingest live `chats` / `history` / `members` / `statuses` / `presence` / `calls` / `contacts` / `media` sync phases (`started`, `progress`, `completed`, `failed`) into sanitized `whatsapp.sync.*` and accepted runtime-event evidence. |
| `GET` | `/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/webhooks` | Verify Business Cloud webhook challenge using the host-vault `whatsapp_business_cloud_webhook_verify_token`. Still protected by `X-Hermes-Secret`; intended for a local/proxy bridge, not direct public exposure. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/webhooks` | Verify `X-Hub-Signature-256` with the host-vault `whatsapp_business_cloud_app_secret`, then ingest Meta-like Business Cloud webhook payloads into the same message/receipt/runtime evidence spine. |
| `GET` | `/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/proxy-manifest` | Return the protected edge/proxy forwarding manifest and per-account readiness booleans without returning secret refs or secret values. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/commands/claim` | Claim due live WhatsApp provider commands for external runtime execution and move them into `executing`. Response items include worker-facing provider/runtime metadata (`provider_kind`, `provider_shape`, `runtime_kind`, `lifecycle_state`, `session_restore_available`, `runtime_blockers`) plus durable command execution fields (`capability_state`, `action_class`, `confirmation_decision`, `provider_state`, `result_payload`). |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/commands/{command_id}/failed` | Report live runtime execution failure and reschedule or dead-letter the command through existing retry policy. Optional `error_code` and `retry_after_seconds` let the worker persist structured failure metadata and suggest the next retry delay without bypassing the durable outbox lifecycle. |
| `POST` | `/api/v1/integrations/whatsapp/runtime-bridge/sessions/authorized` | Persist successful live authorization session material into the host-vault binding path. |

## Current limits

- Live runtime execution remains blocked-safe unless an explicit provider runtime
  implementation is enabled.
- Business Cloud is modelled as a separate provider shape and must not be used
  as a personal WhatsApp substitute.
- Business Cloud `send_text`, `send_template`, `send_media` and
  `send_voice_note` provider submissions plus local webhook/status evidence
  ingestion, signature/challenge verification, protected proxy manifest and
  standalone edge proxy binary are smoke-gated and not public availability:
  external public exposure/deployment and live smoke remain open.
- Provider-neutral message sends plus message reply/forward/edit/delete,
  conversation read/unread, pin/unpin, archive/unarchive, mute/unmute,
  reaction, evidence and search surfaces remain owned by Communications, not by
  this integration API.
