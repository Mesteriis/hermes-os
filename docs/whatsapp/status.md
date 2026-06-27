# WhatsApp Implementation Status

Статус на 2026-06-27.

Это сводный статус по текущей цели: *runtime-первый слой + fixture-полумеханизм*, не *готовый production live WhatsApp-провайдер*.

Invariant remains:

- Канал не является доменом.
- WhatsApp — integration.
- Communication — доменный объект.
- Все факты идут как source evidence и затем проектируются в Communications.

Implementation evidence checkpoints and remaining closure gates:

```text
IMPLEMENTED CHECKPOINTS = 67
DOMAIN CLOSURE          = not achieved
LIVE BLOCKERS           = manual smoke, remaining safe write APIs,
                          WebView live smoke,
                          Business Cloud public exposure/smoke
```

Blocked closure mark on 2026-06-27:

- The current WhatsApp goal is **blocked**, not complete.
- `make whatsapp-domain-closure-audit` is expected to pass only as an honesty
  check while reporting `closure_achieved = false`.
- Closure is blocked by missing sanitized live-smoke evidence artifacts for
  `whatsapp_native_md`, `whatsapp_web_companion` and
  `whatsapp_business_cloud`.
- Native MD also still lacks verified safe provider APIs and smoke evidence for
  `archive`, `unarchive`, `mute`, `unmute`, `pin`, `unpin`, `mark_unread`,
  `join_group` and `publish_status`.
- Do not claim WhatsApp domain closure until the live smoke artifacts validate
  and `make whatsapp-domain-closure-gate` passes.

Реализация уже есть (fixture/runtime-safe foundation) для:

1. `provider/account model` — `whatsapp_web`, `whatsapp_business_cloud`, `provider_shape` (`whatsapp_web_companion` / `native_md` / `business_cloud`), session metadata, account lifecycle state transitions.
   Fixture account setup can now also pin `provider_shape = whatsapp_native_md`
   on top of the compatibility `whatsapp_web` provider kind, so the native MD
   runtime boundary is no longer testable only as a blocked live shape.
2. `host-vault boundary` — привязка сессии через `whatsapp_web_session_key`, перезапись/очистка при `revoke/relink/remove`, восстановление старта без ручного вмешательства.
   Повторная успешная авторизация того же аккаунта теперь трактуется как
   `session_rotated`: тот же account-scoped secret binding сохраняется, payload
   в host vault перезаписывается, а lifecycle/runtime events больше не выглядят
   как первичная авторизация.
3. `runtime health diagnostics` — sanitized `runtime/health` теперь различает
   `available` / `degraded` / `blocked` и отдает вложенные diagnostics-блоки
   по `session`, `storage`, `runtime`, `webview` и `validation` без утечки
   session material.
4. `capability contract` — capability-маршруты и состояния `available/degraded/blocked/unsupported`.
5. `fixture/manual runtime` — детерминированный API для message/status/media/media commands/reactions/receipt/dialog/participant/call/presence.
   The same fixture/runtime-safe path can now exercise the `whatsapp_native_md`
   provider shape through account metadata and runtime status/session surfaces,
   not only `whatsapp_web_companion`.
6. `provider-write command model` — durable outbox + reconciler + retry/retry-policy + dead-letter + audit-safe events.
   Live runtime-bridge claim дополнительно отсекает команды без
   `session_restore_available = true`, а также `fixture`, `live_blocked`,
   empty-runtime и unlinked lifecycle states, даже если stale command row
   ошибочно выглядит `queued` / `available`.
7. `message ingestion/projection` — raw/signals → accepted → `communication_messages`, `communication_conversations`, `communication_identities`, tombstones/versions/reactions/attachments и raw evidence.
8. `fixture commands` — send/reply/forward/edit/delete/reactions/media/media-download/media-upload/status publish/media status, dialog state, call/join/leave, voice.
9. `realtime` — sanitized websocket/event-log events для message/dialog/status/reaction/receipt/runtime/call/presence.
10. `frontend workbench` — runtime panel, command audit/retry/dead-letter UI and fixture control surfaces.
11. `timeline/search/shared-engine hooks` — событийному потоку доступны нужные trace-цепочки.
12. `telemetry/event evidence` — наблюдательные события и projection-пути сведены в event-sourcing spine.
13. `native runtime compile boundary` — `whatsapp-rust 0.6.0` провален на stable compile spike, fallback `wa-rs 0.2.0` выбран как optional dependency за `whatsapp-native-md-runtime` без SDK SQLite storage. Rust 1.88 провален из-за `tokio-websockets`, поэтому MSRV поднят до Rust 1.89 и проверен через `cargo +1.89.0 check`. Compile feature не является public capability flag: `native_md` и `business_cloud` live availability остаются blocked до smoke/live evidence. При feature-сборке `native_md` отдает `wa-rs` smoke-gated descriptor с readiness `smoke_gated_unverified_public_blocked` и blocker `whatsapp_native_md_public_availability_blocked`; без feature остается `whatsapp_native_md_runtime_feature_disabled`. Session restore привязан к account-scoped `whatsapp_web_session_key` в host vault.
14. `native runtime actor contract` — `native_md` now has an explicit account-scoped actor contract over the selected `wa-rs` API surface. The contract fixes the command channel to the durable provider outbox, the event sink to Signal Hub raw evidence, the storage boundary to host-vault metadata bindings only, and the event-family matrix for auth/runtime/sync/messages/updates/deletes/receipts/reactions/dialogs/participants/presence/calls/status/media/command reconciliation/unsupported evidence. Public live capability remains blocked until manual live smoke and provider-observed evidence exist.
15. `native wa-rs event classifier contract` — `native_md` now has a feature-gated classifier for real `wa-rs::types::events::Event` variants. It maps auth, connection, sync, message, receipt, presence, dialog/group, participant/contact and unknown/provider-only events to Hermes raw record kinds and accepted Signal Hub event families. `Event::Message` also inspects protobuf reaction/media/call/edit/delete markers plus `MessageInfo.edit`, so reactions, edits, deletes and media metadata are not collapsed into generic messages. Raw `Notification` and `BusinessStatusUpdate` become unsupported runtime evidence, not dropped.
16. `native raw evidence envelope contract` — classified `wa-rs` events now flow through a compile-only `NativeMdRawEvidenceEnvelope` contract before any future writer can append them. The envelope fixes `provider_shape = whatsapp_native_md`, `runtime_driver = wa-rs`, raw record kind, `signal.raw.whatsapp.*.observed` event kind, accepted Signal Hub kind, stable `source_fingerprint:v5` seed, and sanitized payload policy. It explicitly forbids session/token/cookie/raw secrets, message bodies and media bytes in runtime metadata/events/log-like payloads.
17. `native sanitized inbound DTO contract` — feature-gated `native_md` now builds a compile-checked `NativeMdSanitizedProviderEventDto` for real `wa-rs::types::events::Event` values. The DTO pairs the raw-evidence envelope with metadata-only provider details: message ids, JIDs, timestamps, receipt/presence states, sync counters and safe payload-shape flags. It also fixes the dispatch target to the existing `/api/v1/integrations/whatsapp/runtime-bridge/*` family for messages, updates, deletes, receipts, reactions, media, media lifecycle, statuses, status views/deletes, presence, calls, dialogs, participants, runtime events and sync lifecycle. It deliberately excludes QR codes, pair codes, raw `Node`, protobuf action payloads, history-sync payloads, about text, push names, session material, message bodies and media bytes before any future live producer can append Hub evidence.
18. `native runtime health surface` — `runtime/health` for `whatsapp_native_md` now includes the native driver descriptor in `checks.native_md_driver` and `checks.runtime.native_driver`: driver id/readiness, live-runtime blocker, account-scoped actor scope, durable outbox command channel, Signal Hub raw-evidence sink, host-vault session purpose and metadata-only database policy. QR/pair-code and provider-command blockers also include the provider-shape blocker, so native auth/write surfaces cannot look publicly available while the live driver is smoke-gated or feature-disabled.
19. `native wa-rs host-vault backend` — `native_md` now records the real `wa-rs` runtime builder prerequisites in the compile-checked actor contract: `Backend`, `TransportFactory`, `HttpClient`, wrapper `Device`, core serializable `Device` and `PairCodeOptions`. The adapter also implements the required `wa-rs` store families (`SignalStore`, `AppSyncStore`, `ProtocolStore`, `DeviceStore`) as `NativeMdHostVaultBackend`: account-scoped encrypted host-vault snapshot under `whatsapp_web_session_key`, SDK SQLite disabled, and PostgreSQL secret payloads forbidden.
20. `native wa-rs client factory` — `native_md` now has a compile-checked `NativeMdWaRsClientFactory::configured_builder` that wires `NativeMdHostVaultBackend`, `TokioWebSocketTransportFactory`, `UreqHttpClient`, optional pair-code options and a sanitized event handler into `wa_rs::bot::BotBuilder`. The event handler derives a stable provider event id from sanitized metadata and builds the same sanitized DTO path used by the Signal Hub fixture contract. This still does not make live native execution publicly available; manual live smoke and remaining capability coverage stay blocked.
21. `native wa-rs live driver lifecycle` — `native_md` now has a compile-checked `NativeMdLiveDriver` entrypoint that builds the configured `wa_rs::bot::Bot`, starts it through `Bot::run()`, and stops it through `Client::disconnect()` plus task abort cleanup. Inbound events are converted into owned sanitized DTOs and handed to the shared `WhatsAppRuntimeEventSink` contract, not to Communications/Personas/etc. directly. Public native runtime availability remains blocked until manual live smoke and the remaining live capability matrix pass.
22. `native runtime Signal Hub sink` — `WhatsappRuntimeSignalIngestService` now implements the shared sink contract for sanitized native runtime DTOs. It records append-only `communication_raw_records`, redacts secret-like metadata recursively, dispatches `signal.raw.whatsapp.*.observed`, verifies the resulting `signal.accepted.whatsapp.*` kind and stays idempotent for duplicate provider-observed events. This is an application-level event-spine writer; it still does not enable public live runtime by itself.
23. `native runtime smoke manager` — `native_md` now has an account-scoped runtime manager wired through `WhatsAppProviderRuntime` lifecycle hooks. It can start the feature-gated live driver only when the account config explicitly opts into `native_md_live_smoke_enabled` and a `whatsapp_web_session_key` host-vault binding exists. Health exposes `checks.native_md_manager` / `checks.runtime.native_manager` with manager wiring, opt-in, feature, running, link-start vault binding, reconnect policy and public-availability gate metadata. This is a controlled smoke path, not a capability flag; public availability remains blocked until manual live smoke and the remaining live capability matrix pass.
24. `native vault-aware link startup` — `WhatsAppProviderRuntime::start_qr_link` and `start_pair_code_link` now receive `SecretReferenceStore` and `HostVault` context. For `whatsapp_native_md`, the smoke manager can create a metadata-only secret reference plus `whatsapp_web_session_key` host-vault bootstrap snapshot before starting the feature-gated driver; pair-code startup passes the phone number into `wa_rs::bot::BotBuilder::with_pair_code`. Without explicit smoke opt-in the API stays blocked and returns no QR/pair-code artifact.
25. `native transient auth artifact channel` — feature-gated `native_md` now captures `wa-rs` `PairingQrCode` / `PairingCode` events in an in-process, account-scoped, one-time transient channel. The runtime start response can expose QR SVG or pair code with expiry after the live driver observes the provider event, while sanitized runtime DTOs still redact `qr_code` / `pair_code` and health reports only `memory_only_not_postgres_events_logs`. Artifacts are not written to PostgreSQL, Signal Hub events, logs or health payloads.
26. `native startup restore attempt` — `whatsapp_runtime_restore_reconciliation` now goes beyond Signal Hub snapshot sync: for eligible `whatsapp_native_md` accounts with a host-vault `whatsapp_web_session_key` binding and explicit native smoke opt-in, the worker calls `WhatsAppProviderRuntime::start_runtime` during backend startup/ticks. Success and failure are surfaced as sanitized `startup_restore_start` / `startup_restore_start_failed` runtime/session events; raw session material is never emitted.
27. `native reconnect policy` — `native_md` now has an account-scoped lifecycle registry with bounded backoff (`base_delay_seconds`, `max_delay_seconds`, `max_attempts`) and tick-driven reconnect from the same vault-bound session used by startup restore. Provider-observed `Connected` marks `connection.recovered`, disconnect/failure events mark `connection.degraded`, and manager restarts emit sanitized `connection.reconnect.started` / `connection.reconnect.failed` runtime evidence through the shared Signal Hub sink. This remains smoke-gated and does not flip public live capability flags.
28. `native provider command execution boundary` — `native_md` now has a smoke-gated command executor path behind `WhatsAppProviderRuntime::execute_live_provider_command`. The backend claims only durable outbox rows for `provider_shape = whatsapp_native_md`, requires vault-backed `session_restore_available = true`, routes execution through the account-scoped `wa-rs` live driver, and records SDK success only as sanitized `provider_submission` metadata with `completion_rule = provider_observed_event_reconciliation_required`. The command lock is cleared after provider submission so stale recovery does not resend already-submitted writes, but `completed_at` is still set only by provider-observed reconciliation. Verified SDK operations are send text, reply, forwarded text reemit, edit, delete/revoke, react/unreact, mark-read when provider message ids are present, leave group, send media upload, send voice-note upload and smoke-gated media download into local blob storage. Status/archive/mute/pin/join/unread remain structured unsupported/failure paths until the SDK/API surface and smoke evidence are verified.
29. `native live media upload submission` — live `send_media` / `send_voice_note` now read bytes from local blob storage in the application worker, validate size and SHA-256 against stored metadata, pass bytes to the runtime only as a redacted in-memory field excluded from serialization, upload through `wa-rs::Client::upload`, and send the resulting image/video/audio/document protobuf message through `send_message`. SDK success emits only sanitized provider-submission/progress metadata; raw media bytes, `media_key`, provider URL and session material stay out of PostgreSQL, events, logs and frontend payloads. Command completion still waits for provider-observed evidence.
30. `native live command capability health matrix` — native driver/manager health now exposes the smoke-gated command surface explicitly: verified SDK submission subset is `send_text`, `reply`, `forward`, `edit`, `delete`, `react`, `unreact`, `mark_read`, `leave_group`, `send_media`, `send_voice_note`, `download_media`; unsupported live commands remain `publish_status`, `archive`, `unarchive`, `mute`, `unmute`, `pin`, `unpin`, `join_group`, `mark_unread`. Health also states the public availability gate, provider-observed reconciliation rule and sanitized payload policy, so UI/workers cannot infer broader live capability from the presence of the SDK feature.
31. `native media download ref contract` — native inbound media DTOs now extract direct image/video/audio/document/sticker download refs from `wa-rs` protobuf messages into sanitized metadata: media type, content type, file length, file hashes, direct-path/static-url hashes, media-key hash, provider media ref fingerprint and the required `whatsapp_media_download_ref` host-vault secret purpose. Raw `media_key`, `direct_path`, `static_url`, URLs, captions, filenames and bytes remain excluded from runtime DTOs/raw evidence/log-like payloads. This contract feeds the host-vault materialization and local-blob download path, but public availability still waits for smoke.
32. `native media download ref vault materialization` — live `native_md` provider event handling now materializes raw media download refs into the host vault before redaction. The event handler receives `SecretReferenceStore` plus `HostVault`, writes a metadata-only `secret_references` row and an encrypted host-vault payload under deterministic `secret:provider-account:<account>:whatsapp_media_download_ref:<fingerprint>` refs, then emits only `secret_ref`, fingerprint, hashes and materialization status in sanitized DTOs. Raw `media_key`, `direct_path`, `static_url` and URL values remain host-vault-only and are not written to PostgreSQL command payloads, raw evidence, event-log payloads, logs or frontend caches.
33. `native media download to local blob path` — live `download_media` now consumes the host-vault `whatsapp_media_download_ref`, decodes only in the application worker/runtime memory, downloads through `wa-rs::Client::download_from_params`, writes bytes to local blob storage, and emits media lifecycle plus runtime-bridge media observation evidence for projection/reconciliation. Downloaded bytes, `media_key`, `direct_path` and provider URLs remain excluded from PostgreSQL command payloads, raw evidence, event-log payloads, logs and frontend caches. Manual smoke evidence is still required before public availability can be opened.
34. `business cloud credential vault binding` — live setup for `whatsapp_business_cloud` now requires `api_access_token`, stores it only in host vault under account-scoped `whatsapp_business_cloud_access_token`, writes PostgreSQL metadata/binding only, redacts request debug output and marks the account lifecycle `linked` only after the vault write succeeds. Runtime status can now distinguish a vault-bound Cloud credential from missing setup, while public Cloud execution remains blocked until the production Business Cloud runtime/webhook/template flow is implemented and smoked.
35. `business cloud smoke-gated send submission` — `whatsapp_business_cloud` now has a provider-shape-scoped command executor for `send_text` behind `runtime = business_cloud_smoke`, `business_cloud_live_smoke_enabled = true` and a vault-bound `whatsapp_business_cloud_access_token`. The application worker reads the token from host vault only into a redacted in-memory command field, submits through the official Graph messages endpoint shape, records only sanitized provider-submission metadata and keeps completion blocked on provider-observed webhook/event reconciliation. Personal WhatsApp `messages.send_text` remains unsupported for Business Cloud; the exposed capability is `business.messages.send_text`, still public-blocked until smoke and webhook reconciliation are complete.
36. `business cloud webhook evidence ingestion` — `whatsapp_business_cloud`
    now has a local runtime-bridge webhook ingest path for Meta-like Cloud
    webhook payloads. Text messages normalize into the existing live message
    evidence route, delivery statuses normalize into receipt evidence for
    provider-observed command reconciliation, and unsupported/missing/failed
    webhook entries become sanitized degraded runtime evidence instead of being
    dropped. Account resolution is explicit `_hermes.account_id` / `account_id`
    or metadata `phone_number_id`; public webhook verification, signature
    validation and rate-limit semantics exist, while public proxy deployment and
    live smoke remain open.
37. `business cloud webhook verification hardening` — the Business Cloud
    runtime-bridge webhook route now supports GET challenge verification and
    POST raw-body HMAC-SHA256 signature verification before any evidence is
    ingested. `webhook_verify_token` and `app_secret` are separate
    account-scoped host-vault secrets with PostgreSQL metadata/bindings only;
    setup/debug output, event metadata and unsupported evidence explicitly
    exclude access tokens, app secrets, verify tokens and raw provider payloads.
    The route remains under ADR-0056 local API auth, so production deployment
    still needs an explicit local proxy/edge bridge rather than exposing Hermes
    directly to the public Internet.
38. `business cloud send_text rate-limit semantics` — the smoke-gated Business
    Cloud `send_text` executor now reads provider `Retry-After` headers,
    maps HTTP 429 into structured `business_cloud_rate_limited`, preserves
    sanitized provider error code/type in the failure message, and feeds the
    existing durable outbox retry/dead-letter path through `error_code` and
    `retry_after_seconds`. Raw provider payloads and credentials still stay out
    of command/result/event metadata.
39. `business cloud template/media submission subset` — the smoke-gated
    Business Cloud executor now claims durable outbox rows for `send_text`,
    `send_template`, `send_media` and `send_voice_note` only. Templates submit
    through the Graph messages endpoint with sanitized result metadata. Media
    and voice-note submissions read bytes from local blob storage in the worker,
    validate size and SHA-256, upload through the Graph media endpoint, send the
    returned media id through the messages endpoint and store only sanitized
    provider-submission metadata. Raw media bytes, access tokens, template
    component payloads, filenames/captions and raw provider payloads are
    excluded from command result/event metadata. Completion still waits for
    provider-observed webhook reconciliation and manual smoke.
40. `business cloud edge/proxy manifest` — the protected local runtime-bridge
    namespace now exposes
    `/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/proxy-manifest`.
    It documents the external edge/proxy forwarding contract for GET challenge
    verification and POST raw-body webhook delivery, keeps Hermes itself behind
    ADR-0056 `X-Hermes-Secret`, reports only account readiness booleans and
    secret-purpose names, and never reads or returns host-vault secret values.
    This closes the in-process proxy contract but not external edge deployment
    or live Business Cloud smoke evidence.
41. `business cloud edge proxy binary` — the repository now includes the
    standalone `hermes-whatsapp-business-cloud-edge-proxy` binary. It exposes
    public `/webhooks/whatsapp/business-cloud`, forwards GET challenge queries
    and POST raw bodies plus `X-Hub-Signature-256` to the protected Hermes
    runtime-bridge with `X-Hermes-Secret`, sanitizes upstream failures, does not
    parse webhook JSON, does not read host-vault secrets and keeps Hermes itself
    non-public. This closes the deployable local edge bridge artifact; public
    exposure/deployment and Business Cloud live smoke evidence remain open.
42. `business cloud edge proxy behavioral contract` — the edge proxy now has
    executable unit coverage around the public-to-protected forwarding contract:
    `/readyz` checks the protected manifest without account query state, GET
    challenge forwarding preserves Meta query params and appends optional
    account scope, POST webhook forwarding preserves the exact raw body,
    `Content-Type` and `X-Hub-Signature-256` without adding `account_id`, and
    unsigned POST requests fail before reaching Hermes. The Signal Hub static
    contract test also guards those route/account-scoping invariants.
43. `web companion bridge health contract` — `whatsapp_web_companion` runtime
    health now exposes a machine-readable WebView companion bridge contract. It
    keeps public availability blocked, requires an owner-visible desktop runtime,
    forbids hidden/headless mode, binds session restore to host-vault
    `whatsapp_web_session_key`, enumerates the protected runtime-bridge event
    routes for messages, updates, deletes, receipts, reactions, dialogs,
    participants, presence, calls metadata, statuses, media lifecycle, runtime
    and sync lifecycle, and fixes provider writes to the durable outbox
    claim/failure paths with provider-observed reconciliation. The contract also
    excludes session material, cookies, browser profile secrets, QR/pair-code
    artifacts, message bodies and media bytes from health/event-like payloads.
    This closes the WebView bridge contract surface but not safe extractor
    injection, live relay or live smoke.
44. `native wa-rs command gap manifest` — `whatsapp_native_md` runtime health
    now exposes a machine-readable `wa_rs_sdk_command_gap` block. It records the
    verified `wa-rs 0.2.0` public SDK methods already used for the smoke-gated
    subset (`send_message`, `edit_message`, `revoke_message`, `mark_as_read`,
    `groups().leave`, `upload` and `download_from_params`) and separately names
    the forwarded-text reemit contract for `forward`. It separately names the
    missing safe write APIs for `publish_status`, `archive/unarchive`,
    `mute/unmute`, `pin/unpin`, `mark_unread` and `join_group`.
    Unsupported native commands are explicitly allowed to be
    claimed only to write structured terminal dead-letter evidence with
    `native_md_command_kind_unsupported`; they still cannot complete without a
    provider-observed event. This closes the ambiguity around the native write
    gap but not the unsupported command implementation or manual live smoke.
45. `business cloud edge proxy compose profile` — the standalone
    `hermes-whatsapp-business-cloud-edge-proxy` is now packaged as an opt-in
    Docker Compose profile under `docker/`. The profile builds a dedicated
    runtime image target, publishes only the proxy port, forwards to a protected
    local Hermes base URL with `X-Hermes-Secret`, defaults to loopback bind for
    local safety, exposes health checks and has Makefile entry points for
    config/up/stop/logs. `docker/.env.example` contains only non-secret
    placeholders. This closes the local deployable bridge packaging surface but
    not real public ingress configuration, Meta webhook registration or live
    Business Cloud smoke evidence.
46. `native unsupported command preflight` — `whatsapp_native_md` command
    execution now performs a deterministic unsupported-command preflight before
    the smoke gate and before runtime driver lookup. Known missing `wa-rs`
    write surfaces (`publish_status`, dialog state writes, `mark_unread`,
    `join_group`) fail with
    `native_md_command_kind_unsupported` before being masked as
    `native_md_runtime_not_running` or SDK submission. Unknown command kinds
    remain `native_md_command_kind_unverified`.
    This closes the structured unsupported-failure ambiguity, but not the
    missing provider APIs or live smoke evidence.
47. `native media upload observed completion target` — live `native_md`
    `send_media` / `send_voice_note` submissions now persist a sanitized
    `provider_observed_completion_target` and the media reconciler can complete
    those commands when accepted `signal.accepted.whatsapp.media` evidence
    carries a `provider_message_id` equal to the stored `wa-rs`
    `provider_request_id`. The existing blob-id fallback remains for fixtures.
    Raw media bytes, provider URLs, direct paths and media keys remain excluded.
    This closes the provider-request-id reconciliation gap for observed upload
    completion, but still requires manual live smoke to prove the provider emits
    the expected media observation in practice.
48. `business cloud receipt observed completion target` — smoke-gated
    Business Cloud Graph submissions for `send_text`, `send_template`,
    `send_media` and `send_voice_note` now persist a sanitized
    `provider_observed_completion_target` for accepted
    `signal.accepted.whatsapp.receipt` evidence. The receipt reconciler can
    complete those commands when webhook `statuses[]` evidence carries a
    provider message id equal to the stored Graph message id/provider request
    id, even when the original outbox row did not have a `provider_message_id`
    column value. Access tokens, template components, raw provider payloads and
    media bytes remain excluded. This closes the Business Cloud
    provider-request-id reconciliation gap, but public ingress/Meta webhook
    registration and live smoke evidence remain open.
49. `native unsupported command terminal dead-letter` — known missing `wa-rs`
    write surfaces now become terminal dead-letter evidence immediately after
    the native unsupported preflight. The live worker writes sanitized failure
    metadata with `retry_policy = terminal`, clears the execution lock, keeps
    `reconciliation_status = not_observed` and does not reschedule retries for
    provider APIs that do not exist locally. Transient native SDK/network
    failures remain retryable through the existing retry/dead-letter path. This
    closes the retry-loop/operational-noise risk for unsupported native
    commands, but not the underlying provider API gap or manual live smoke.
50. `web companion visible desktop producer shell` — the Tauri shell now exposes
    `open_whatsapp_web_companion` and `whatsapp_web_companion_manifest`
    commands. The opener creates or focuses an owner-visible
    `https://web.whatsapp.com/` WebView window with an account-scoped
    `whatsapp-companion-*` label and returns a sanitized manifest that points
    only at protected `/runtime-bridge/*` event routes, the authorized-session
    host-vault bridge and durable outbox claim/failure paths. The companion
    window is not granted broad Tauri IPC in the default capability scope, and
    the command does not read or return cookies, browser profile secrets,
    session material, message bodies or media bytes. This closes the missing
    visible desktop shell artifact, but safe extractor injection, relay dispatch
    and manual live smoke remain open.
51. `web companion frontend Tauri bridge` — the frontend now has a typed
    `@tauri-apps/api` bridge for the WebView companion shell:
    `openWhatsappWebCompanion` and `getWhatsappWebCompanionManifest` call the
    Tauri commands directly through `invoke` and do not route through
    `ApiClient`, `fetch` or backend/domain HTTP routes. Unit coverage verifies
    account-id trimming, empty-account rejection, sanitized manifest shape,
    authorized-session/runtime-bridge paths, provider-observed reconciliation
    policy and absence of secret-bearing fields. This makes the visible shell
    reachable from frontend code without crossing Communications boundaries,
    but safe extractor injection, backend dispatch and manual live smoke remain
    open.
52. `web companion extractor injection contract` — the owner-visible Tauri
    companion window now installs a main-frame-only initialization script guarded
    to `https://web.whatsapp.com` and navigation is constrained to the same
    origin. The script exposes only a frozen
    `__HERMES_WHATSAPP_COMPANION__` metadata contract and a local DOM readiness
    event; it does not read cookies, Web Storage, IndexedDB, browser profile
    secrets, message bodies or media bytes, and it does not call `fetch`, XHR,
    `postMessage`, Tauri invoke or domain APIs. Runtime health and the frontend
    manifest now report an allowlisted relay contract. This closes the safe
    injection contract, but a dispatching relay command and manual live smoke
    remain open.
53. `web companion allowlisted relay command ACL` — the Tauri shell now registers
    `whatsapp_web_companion_relay_observation` and an explicit remote
    capability for `https://web.whatsapp.com` scoped to `whatsapp-companion-*`
    windows only. The default `main` capability gets only
    `open_whatsapp_web_companion` and `whatsapp_web_companion_manifest`; the
    remote companion does not get `core:default`. The relay command validates
    that the caller window label matches the account-scoped companion label,
    accepts only known WhatsApp event families, maps them to the protected
    `/runtime-bridge/*` target paths, recursively drops secret-like/private
    content metadata keys, and returns a sanitized relay receipt with
    `provider_observed_event_reconciliation_required`. It does not mutate
    Communications/Personas/etc. and does not complete provider commands. This
    closes the Tauri ACL risk; backend runtime-event dispatch and live smoke
    remain open.
54. `web companion runtime-event relay dispatch` — the Tauri relay command now
    converts metadata-only WebView observations into `NewWhatsappWebRuntimeEvent`
    payloads and POSTs them to the protected local
    `/api/v1/integrations/whatsapp/runtime-bridge/runtime-events` route with
    `X-Hermes-Secret` read from the Tauri process environment only. Dispatch is
    limited to loopback HTTP targets, the remote companion capability still only
    grants `whatsapp_web_companion_relay_observation`, metadata is recursively
    sanitized before dispatch, and the receipt records both the actual
    runtime-events target and the richer typed path that should be used later
    when a typed extractor payload exists. This creates real raw/accepted
    runtime-event evidence through the Hub spine without pretending to project
    full messages/status/media from partial WebView metadata; manual live smoke
    remains open.
55. `live smoke readiness harness` — `make whatsapp-live-smoke-readiness` now
    runs `scripts/whatsapp-live-smoke-readiness.mjs`, a fast static preflight
    for the manual WhatsApp smoke gates. It verifies the WebView relay dispatch
    contract, narrow Tauri remote capability, backend runtime-health dispatch
    state, Signal Hub static guard coverage, smoke checklist exit criteria and
    current remaining blocker language. With `HERMES_LIVE_SMOKE_STRICT_ENV=1`
    it also requires `HERMES_LOCAL_API_SECRET` and
    `HERMES_WHATSAPP_SMOKE_ACCOUNT_ID`. This does not perform provider live
    actions and does not close the live smoke gate; it makes the eventual manual
    smoke run reproducible and auditable without running the broad backend suite.
56. `web companion runtime-panel action` — the WhatsApp Runtime panel now has an
    owner-visible `Open Companion` action for selected `whatsapp_web_companion`
    accounts. The action calls the typed frontend Tauri bridge
    `openWhatsappWebCompanion`, never `fetch` / `ApiClient` / Communications
    HTTP routes, and surfaces only sanitized manifest fields: account window
    label, allowlisted relay channel and runtime-event dispatch state. This
    closes the visible UI action gate; end-to-end WebView live smoke remains
    required before public runtime availability or provider-write completion can
    be claimed.
57. `business cloud edge smoke readiness harness` —
    `make whatsapp-business-cloud-edge-readiness` now runs a fast preflight for
    the standalone Business Cloud edge proxy. It verifies the public proxy path,
    protected Hermes runtime-bridge paths, `X-Hermes-Secret` injection contract,
    `X-Hub-Signature-256` forwarding, unsigned-POST rejection before Hermes
    forwarding, Docker Compose profile, Dockerfile target, loopback default env
    and docs/status coverage. With
    `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_PROBE=1`, it can also probe a running
    local proxy `/healthz`, `/manifest` and unsigned webhook rejection; with
    `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_READYZ_PROBE=1`, it can prove `/readyz`
    reaches the protected Hermes proxy manifest. This closes local operator
    readiness for the edge bridge but does not replace real public ingress,
    Meta webhook registration or Business Cloud live smoke.
58. `runtime API smoke probe` — `make whatsapp-live-smoke-readiness` now has an
    optional runtime API probe for a running local Hermes backend. With
    `HERMES_WHATSAPP_RUNTIME_API_PROBE=1`, `HERMES_LOCAL_API_SECRET` and
    `HERMES_WHATSAPP_SMOKE_ACCOUNT_ID`, it calls the protected WhatsApp
    capabilities, account-capabilities, runtime-status and runtime-health
    endpoints through ADR-0056 `X-Hermes-Secret`, verifies account-scoped
    payload contracts and checks that responses do not expose raw
    session/token/cookie/media-ref payload markers. This closes the repeatable
    local API preflight for manual smoke, but it still does not replace
    provider live actions, provider-observed reconciliation or manual smoke
    evidence.
59. `manual smoke evidence validator` — the repository now has
    `make whatsapp-live-smoke-evidence`, a strict validator for the ignored
    local artifact `.local/whatsapp/live-smoke-evidence.json`. The artifact
    records only sanitized evidence references keyed by closure gates. It
    requires a hashed `account_fingerprint`, owner-visible/no-headless
    attestation, all required gate entries in `status = passed` and rejects
    raw session/token/cookie/authorization/media-ref/private account markers.
    This makes the final live-smoke evidence auditable without committing
    secrets or message contents. It still does not close the domain until a real
    local smoke artifact validates and the remaining provider capability gaps
    are resolved.
60. `remaining native SDK command gap verifier` —
    `make whatsapp-native-md-sdk-gap-readiness` now verifies the local
    `wa-rs 0.2.0` source inventory behind the native MD adapter. It checks that
    the SDK methods used by the smoke-gated verified subset still exist
    (`send_message`, `edit_message`, `revoke_message`, `mark_as_read`,
    `groups().leave`, `upload`, `download_from_params`), verifies the
    `forwarded_text_reemit` contract through `ExtendedTextMessage.ContextInfo`,
    and checks that no public safe API has appeared for the current unsupported
    native commands: `publish_status`, `archive`, `unarchive`, `mute`,
    `unmute`, `pin`, `unpin`, `join_group`, `mark_unread`. This keeps the
    `wa_rs_sdk_command_gap` health manifest grounded in current local source
    evidence and will fail after a future SDK upgrade if the blocker can be
    removed. It does not implement those missing writes or remove the manual
    smoke gate.
61. `domain closure audit gate` —
    `make whatsapp-domain-closure-audit` now produces a machine-readable
    closure audit for the whole WhatsApp provider surface. In normal mode it
    passes as an honesty check while reporting `closure_achieved = false` and
    explicit blockers: missing sanitized live-smoke evidence for
    `whatsapp_native_md`, `whatsapp_web_companion` and
    `whatsapp_business_cloud`, remaining native unsupported commands and
    `ADR-0101` not yet Accepted. `make whatsapp-domain-closure-gate` runs the
    same audit in require-closed mode and must fail until those blockers are
    resolved. This prevents accidental overclaiming; it does not replace live
    provider smoke, SDK/API work or public Business Cloud ingress.
62. `ADR-0101 accepted runtime decision` —
    `ADR-0101` is now `Accepted` for the WhatsApp provider-runtime boundary,
    provider-shape model and native fallback selection. The acceptance scope is
    intentionally narrow: it accepts `whatsapp-rust` rejection, `wa-rs 0.2.0`
    as the smoke-gated native fallback behind `WhatsAppProviderRuntime`, and
    separate WebView companion / Business Cloud provider shapes. It does not
    mark any live provider as publicly available; `whatsapp_native_md`,
    `whatsapp_web_companion` and `whatsapp_business_cloud` still require
    sanitized live-smoke evidence, provider-observed reconciliation and closure
    gate success.
63. `native low-level SDK gap evidence` —
    `make whatsapp-native-md-sdk-gap-readiness` now also inspects the local
    `wa-rs-core 0.2.0` and `wa-rs-appstate 0.2.0` source next to `wa-rs`. It
    records that `Client::send_iq` exists only as a low-level custom stanza
    escape hatch, while `wa-rs-appstate` exposes decode/hash/process helpers but
    no public outgoing app-state patch/mutation encoder for archive, mute, pin,
    unread or status writes. It also verifies that the group IQ surface has
    invite-link fetch/reset but no join-by-invite / accept-invite spec. This
    keeps the native unsupported set grounded in SDK source evidence instead of
    relying on naming alone.
64. `native forward text reemit submission` —
    `whatsapp_native_md` can now submit `forward` through the same
    `WhatsAppProviderRuntime` command boundary as other writes. The request
    carries text from the Communications projection plus source provider ids,
    the runtime sends an `ExtendedTextMessage` through `wa-rs::Client::send_message`
    with `ContextInfo.is_forwarded = true` and `forwarding_score = 1`, and
    provider submission metadata records `submission_mode =
    forwarded_text_reemit` without storing message bodies in health/audit-like
    payloads. SDK success remains only `submitted`; completion still requires
    provider-observed Signal Hub evidence and reconciliation. Status/archive/
    mute/pin/join/unread remain unsupported until safe provider APIs and smoke
    evidence exist.
65. `strict live-smoke evidence references` —
    `make whatsapp-live-smoke-evidence` now validates gate-specific
    `evidence_refs` instead of accepting arbitrary notes. Raw/inbound gates must
    cite `raw_record:` plus accepted `event_log:` / `signal_hub:` refs,
    provider writes must cite both `command:` and observed event refs, session
    restore and credential gates must cite `vault_binding:` / `runtime_api:`
    refs, media gates must cite `blob:` / `storage:` refs, Business Cloud edge
    gates must cite `edge_proxy:` refs, and redaction gates must cite the
    inspected API/log/UI/audit surface. Placeholder refs and weak
    reconciliation evidence fail the validator and therefore cannot satisfy the
    domain closure audit.
66. `native Rust/wa-rs upgrade path verifier` —
    `make whatsapp-native-md-sdk-gap-readiness` now also checks the native
    upgrade context: backend MSRV is `rust-version = "1.89"` after the Rust 1.88
    compile spike failed, `backend/Cargo.toml` / `Cargo.lock` still pin
    `wa-rs` / `wa-rs-core` `0.2.0`, and native health keeps the remaining
    unsupported commands blocked on safe provider APIs plus smoke evidence.
    Rust/toolchain upgrade is not treated as sufficient evidence for
    `publish_status`, archive/mute/pin/unread app-state writes or join-by-invite.
    Operators can set `HERMES_WA_RS_CRATES_IO_PROBE=1` to have the same verifier
    run `cargo info` against published `wa-rs`, `wa-rs-core` and
    `wa-rs-appstate` before considering a dependency upgrade.
67. `live-smoke evidence collector` —
    `make whatsapp-live-smoke-collect-evidence` now builds the ignored
    `.local/whatsapp/live-smoke-evidence-<provider_shape>.json` artifact from a
    sanitized `.local/whatsapp/live-smoke-observations.json` input and
    immediately validates it with `make whatsapp-live-smoke-evidence` semantics.
    The collector can render an observations template, can derive only a hashed
    `account_fingerprint` from `HERMES_WHATSAPP_SMOKE_ACCOUNT_ID`, rejects
    secret-like observation content, and leaves gates pending unless the operator
    supplies concrete sanitized `evidence_refs`. It therefore helps produce
    closure evidence from real live smoke without allowing synthetic passed
    gates.

Event-first guard coverage now also has a narrow static layer:

- `backend/tests/communications_architecture_target.rs` confines concrete
  WhatsApp provider-library imports to runtime adapter subtrees and rejects
  direct WhatsApp runtime dependencies from domains/engines/workflows.
- `backend/tests/whatsapp_signal_hub.rs` verifies the Signal Hub event-family
  fixture matrix, sanitized payload contract and provider-observed command
  reconciliation invariant without running DB/Testcontainers-backed suites.

Открытые критические пробелы:

1. `WebView companion live runtime` — bridge contract, runtime-health manifest, owner-visible Tauri WebView shell, typed frontend invoke bridge, runtime-panel `Open Companion` action, safe extractor injection contract and allowlisted runtime-event relay dispatch существуют, но отсутствует live smoke.
2. `native_md runtime` — выделен как отдельный provider shape with an account-scoped actor contract, `wa-rs` compile probe, host-vault backend/store-family manifest, event-family classifier, raw evidence envelope, sanitized inbound DTO, runtime-bridge dispatch contract, application-level Signal Hub sink, smoke-gated runtime manager, vault-aware link startup, transient QR/pair-code response channel, startup restore attempt, reconnect policy, verified-subset provider command execution boundary, deterministic unsupported-command failure preflight, live media upload submission path, sanitized media download ref contract, host-vault media ref materialization, local-blob media download path and health/capability matrix including explicit `wa-rs` SDK command-gap evidence, но без manual live smoke and full outbound command coverage.
3. `business_cloud runtime` — отдельный provider shape, host-vault API-token/app-secret/verify-token setup, smoke-gated `send_text`/`send_template`/`send_media`/`send_voice_note` provider submission with rate-limit retry hints and provider-request-id receipt reconciliation target, local runtime-bridge webhook/status evidence ingestion, local bridge verification/signature hardening, protected edge/proxy manifest, standalone edge proxy binary, Docker Compose edge-proxy profile, behavioral proxy forwarding tests and edge readiness harness существуют, но реальный public ingress/Meta webhook registration, live smoke and production public availability ещё не реализованы.
4. `provider event bridge` из live runtime — typed backend bridge routes для внешнего runtime-процесса уже есть; WebView shell can open the visible companion window, frontend code can invoke it through Tauri and the companion has an allowlisted metadata-only runtime-event relay dispatch, но typed WebView projections, native/business-cloud producer smoke and end-to-end execution всё ещё отсутствуют.
5. `live media transfer + progress` — upload submission path for native `send_media` / `send_voice_note` exists with local blob byte-read, SHA validation, `wa-rs` upload/send, sanitized progress metadata and provider-request-id based observed-completion reconciliation. Inbound media messages now expose sanitized hash-only download refs, materialize raw provider refs into host vault and support smoke-gated `download_media` into local blob storage through runtime-bridge media observation evidence. Manual smoke evidence remains open.
6. `calls runtime` — metadata only; live handling, control и запись/preview remain out of scope.

## Ключевое правило на этом этапе

Документ описывает текущую цель как **полноценное live-provider покрытие**, но в текущей реализации production-safe runtime execution пока не закрыт end-to-end. Все новые/существующие команды через capability/fixture должны быть наблюдаемыми и завершаться через observer reconciliation, чтобы live-переход не ломал трассируемость.

Доп. сводка:

- Текущие WhatsApp integration тесты: 47 (`backend/tests/whatsapp.rs`).
- Fast WhatsApp contract tests: `backend/tests/whatsapp_signal_hub.rs` and the
  WhatsApp slices inside `backend/tests/communications_architecture_target.rs`.
- Базовые контрактные тесты проходят, но часть интеграционных тестов падает в среде из-за инфраструктурного лимита Docker (недостаток сети/адресов для контейнеров).

См. также:

- [`current-audit-2026-06-24.md`](./current-audit-2026-06-24.md)
- [`full-functionality-target.md`](./full-functionality-target.md)
- [`implementation-plan.md`](./implementation-plan.md)
- [`gap-analysis.md`](./gap-analysis.md)
- [`api.md`](./api.md)
