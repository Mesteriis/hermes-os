# WhatsApp Live Smoke Checklist

Status: manual local validation only.
Date: 2026-06-26.

This checklist exists for ADR-0101 acceptance and must be used only for
owner-visible local runtime validation. It is not a CI workflow.

## Preconditions

- local development machine under owner control;
- `HERMES_LOCAL_API_SECRET` configured;
- host vault available;
- no screen-hidden or headless WhatsApp runtime mode;
- explicit owner opt-in for unofficial personal-account runtime risk;
- test account or low-risk account selected for the session.

Before starting live provider actions, run:

```sh
make whatsapp-domain-closure-audit
make whatsapp-live-smoke-readiness
make whatsapp-native-md-sdk-gap-readiness # required for whatsapp_native_md accounts
```

For the strict manual-smoke preflight, set:

```sh
HERMES_LIVE_SMOKE_STRICT_ENV=1
HERMES_LOCAL_API_SECRET=...
HERMES_WHATSAPP_SMOKE_ACCOUNT_ID=...
make whatsapp-live-smoke-readiness
```

When Hermes is already running locally, the same readiness target can also
probe the protected runtime API without performing provider actions:

```sh
HERMES_WHATSAPP_RUNTIME_API_PROBE=1
HERMES_LOCAL_API_SECRET=...
HERMES_WHATSAPP_SMOKE_ACCOUNT_ID=...
HERMES_WHATSAPP_SMOKE_PROVIDER_SHAPE=whatsapp_web_companion # optional
make whatsapp-live-smoke-readiness
```

The runtime API probe calls only capabilities, account-capabilities,
runtime-status and runtime-health endpoints through `X-Hermes-Secret`. It
checks account scoping, provider-shape contract when requested and the absence
of raw session/token/cookie/media-ref payload markers.

This readiness check is static/preflight evidence only. It does not replace the
manual live smoke run below.

## Evidence artifact

The manual smoke run must produce a sanitized local evidence artifact before the
domain can be treated as closed. The default path is ignored by git:

```text
.local/whatsapp/live-smoke-evidence.json
```

Create the template locally after the preflight:

```sh
mkdir -p .local/whatsapp
node scripts/whatsapp-live-smoke-evidence.mjs --template > .local/whatsapp/live-smoke-evidence.json
```

Fill it with sanitized evidence references only, then validate it:

```sh
node scripts/whatsapp-live-smoke-collect-evidence.mjs --observations-template \
  --provider-shape whatsapp_native_md > .local/whatsapp/live-smoke-observations.json
make whatsapp-live-smoke-collect-evidence
make whatsapp-live-smoke-evidence
make whatsapp-domain-closure-gate
```

`make whatsapp-live-smoke-collect-evidence` is a normalizer, not a bypass. It
reads `.local/whatsapp/live-smoke-observations.json`, writes an ignored
`.local/whatsapp/live-smoke-evidence-<provider_shape>.json` artifact and then
runs the strict validator. Gates without operator-provided sanitized
`evidence_refs` remain pending, and the command fails until every required gate
has real evidence.

The artifact must use `account_fingerprint = sha256:<64 hex chars>` rather
than raw account ids, phone numbers or JIDs. It must not contain message
bodies, provider payloads, QR/pair codes, cookies, authorization headers,
session material, media keys, direct paths, static URLs, access token values,
app secret values or verify token values. Required gates stay failed until
their evidence entry is `status = passed`; unsupported or skipped entries do
not close the domain.

Each passed gate must also include concrete sanitized `evidence_refs`, not a
free-form note. The validator requires refs with prefixes that match the gate:
`raw_record:` for raw evidence, `event_log:` / `signal_hub:` for accepted
events, `command:` plus observed event refs for provider writes,
`vault_binding:` for session/credential binding, `blob:` / `storage:` for
media bytes, `runtime_api:` for protected API probes, `edge_proxy:` for
Business Cloud ingress and `log_scan:` / `ui:` / `audit:` for redaction checks.
Placeholder refs such as `replace-with-*`, `pending`, `todo`, `example` or
`dummy` are rejected.

## Runtime boundary checks

1. Start Hermes with the intended WhatsApp runtime shape enabled.
2. Verify the runtime is surfaced through:
   - `GET /api/v1/integrations/whatsapp/capabilities`
   - `GET /api/v1/integrations/whatsapp/accounts/{account_id}/capabilities`
   - `GET /api/v1/integrations/whatsapp/runtime/status?account_id=...`
3. Confirm the reported provider shape is correct:
   - `whatsapp_web_companion`, or
   - `whatsapp_native_md`, or
   - `whatsapp_business_cloud`.
4. Confirm blocked capabilities stay blocked if the runtime is not fully ready.

## WebView companion checks

Run these only for `whatsapp_web_companion` accounts.

1. From frontend integration code, call `getWhatsappWebCompanionManifest`, which
   invokes Tauri command `whatsapp_web_companion_manifest` for the account, and
   verify it reports:
   - provider shape `whatsapp_web_companion`;
   - target URL `https://web.whatsapp.com/`;
   - protected `/runtime-bridge/*` event routes;
   - authorized-session path
     `/api/v1/integrations/whatsapp/runtime-bridge/sessions/authorized`;
   - durable outbox claim/failure paths;
   - extractor state `contract_injected_relay_dispatch_available`;
   - relay channel `tauri_allowlisted_companion_runtime_bridge_dispatch`;
   - runtime bridge dispatch `runtime_events_bridge_wired_smoke_pending`;
   - no cookie/session/profile/message/media fields.
2. In the WhatsApp Runtime panel, use `Open Companion`. This calls
   `openWhatsappWebCompanion`, which invokes Tauri command
   `open_whatsapp_web_companion`, and verify an owner-visible WhatsApp Web
   window opens or focuses with an account-scoped `whatsapp-companion-*` label.
3. Verify the companion window is not running in hidden/headless mode and is
   granted only the allowlisted metadata relay dispatch command, not
   domain-mutating Tauri IPC or `core:default`.
4. Verify the companion initialization contract is origin-guarded to
   `https://web.whatsapp.com` and does not read cookies, Web Storage, IndexedDB,
   message bodies or media bytes.
5. Verify the allowlisted relay dispatch accepts only sanitized metadata, posts
   a `NewWhatsappWebRuntimeEvent` to `/runtime-bridge/runtime-events` and
   returns `provider_observed_event_reconciliation_required`.
6. Until live smoke proves WebView observations, accepted Hub events and
   projections end-to-end, keep public runtime availability blocked and do not
   mark provider writes completed from WebView UI actions alone.

## Authorization checks

1. Start QR link flow.
2. Verify no QR secret/session payload is written to PostgreSQL, events, logs or API responses.
3. Complete authorization.
4. Verify authorized session material is stored through host vault binding:
   - secret purpose `whatsapp_web_session_key`;
   - account-scoped binding only.
5. Stop Hermes.
6. Start Hermes again.
7. Verify session restore works without re-pairing.
8. Start pair-code flow if the runtime shape supports it.
9. Verify pair-code lifecycle surfaces sanitized state only.
10. Revoke and relink once; verify lifecycle state transitions remain correct.

## Read path checks

1. Receive a private message.
2. Receive a group message.
3. Receive a reply or quoted message.
4. Receive a forward.
5. Receive a reaction update.
6. Receive a media message.
7. Receive a status update.
8. Verify each item reaches:
   - raw evidence;
   - accepted signal;
   - canonical Communications projection;
   - provider-neutral search/timeline surfaces where applicable.

## Write path checks

1. Send text message.
2. Send reply.
3. Send forward.
4. Edit message if runtime/provider supports it.
5. Delete message if runtime/provider supports it.
6. Add reaction.
7. Remove reaction.
8. Upload media.
9. Download media.
10. Send voice note if runtime/provider supports it.
11. Archive/unarchive chat.
12. Mute/unmute chat.
13. Pin/unpin chat.
14. Mark read/unread.
15. Join/leave group if supported.
16. Publish status if supported.

For each write, verify:

- durable command row exists;
- canonical `communication_provider_commands` mirror exists;
- provider-observed completion or failure is recorded;
- no secret/session material appears in command payloads, audit metadata or emitted events.

## Redaction checks

Verify all of the following are redacted or absent:

- session blobs;
- session keys;
- access tokens;
- refresh tokens;
- cookies;
- raw authorization headers;
- vault payload bytes.

Inspect:

- API responses;
- raw-evidence endpoint;
- audit rows;
- event payloads;
- logs.

## Business Cloud edge proxy checks

Run these only for `whatsapp_business_cloud` accounts.

1. Start Hermes locally with ADR-0056 `HERMES_LOCAL_API_SECRET` configured.
2. Run the static edge proxy readiness preflight:
   `make whatsapp-business-cloud-edge-readiness`.
   If the proxy is already running locally, the same target can also probe the
   public proxy surface without touching Meta:
   `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_PROBE=1 make whatsapp-business-cloud-edge-readiness`.
   Add `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_READYZ_PROBE=1` only when Hermes is
   running and `/readyz` should reach the protected local proxy manifest.
3. Validate the edge profile:
   `make whatsapp-business-cloud-edge-config`.
4. Start `hermes-whatsapp-business-cloud-edge-proxy` with:
   `make whatsapp-business-cloud-edge-up`.
   The Compose profile reads:
   - `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_BASE_URL`;
   - `HERMES_LOCAL_API_SECRET`;
   - optional `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_ACCOUNT_ID`.
   `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_SECRET` may be used instead of
   the shared local API secret when running the binary outside Compose.
5. Verify `GET /readyz` succeeds only when the protected Hermes proxy manifest
   is reachable with local auth.
6. Expose only the proxy path `/webhooks/whatsapp/business-cloud` through the
   chosen public ingress; do not expose Hermes `/api/v1` directly.
7. Verify Meta challenge `GET` succeeds through the proxy and reaches the
   protected Hermes runtime-bridge route.
8. Verify signed webhook `POST` forwards the exact raw body and
   `X-Hub-Signature-256`; Hermes performs app-secret verification and Signal Hub
   ingestion.
9. Verify proxy failures are sanitized and do not return upstream bodies,
   access tokens, app secrets, verify tokens or raw provider payloads.

## Failure and recovery checks

1. Stop runtime during pending command execution.
2. Verify command is retried or dead-lettered through durable policy.
3. Restore runtime.
4. Verify canonical state is not corrupted by partial execution.
5. Verify capability degradation is surfaced instead of silent failure.

## Exit criteria

The smoke run passes only if:

- session restore works from vault-bound state;
- raw evidence remains source-backed;
- writes reconcile through observed provider evidence;
- all inspected surfaces remain redacted for secrets;
- no hidden or headless runtime behavior was required.
