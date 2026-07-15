# Zoom Live Smoke Checklist

Status date: 2026-06-27.

The current repository implements Zoom authorization and integration bridge
foundation plus the authorized recording-sync worker. This checklist validates
blocked-live and authorized-live runtime behavior in the current phase.

## Future blocked-live smoke check

### Preconditions

- Backend is running locally.
- Database migrations are applied.
- Fixture routes are enabled only in local/dev mode.
- No real secret values are placed in JSON metadata.
- `/api/v1/integrations/zoom/*` routes exist.

### Check 1 - Capabilities

```http
GET /api/v1/integrations/zoom/capabilities
```

Expected:

```text
runtime_mode = fixture_plus_authorized_live_workers
accounts.fixture = available
accounts.live_blocked = degraded
auth.oauth_user = available
auth.server_to_server = available
auth.token_refresh = available
bridge.meetings = available
bridge.recordings = available
bridge.transcripts = available
bridge.transcript_files = available
```

### Check 2 - Register blocked live account metadata

```http
POST /api/v1/integrations/zoom/accounts
```

Use secret references only:

```json
{
  "account_id": "zoom_live_owner",
  "display_name": "Owner Zoom",
  "external_account_id": "external-account-id",
  "account_email": "owner@example.com",
  "auth_shape": "oauth_user",
  "client_id": "client-id-or-public-ref",
  "token_secret_ref": "secret_ref_zoom_token",
  "webhook_secret_ref": "secret_ref_zoom_webhook",
  "metadata": {}
}
```

Expected:

```text
lifecycle_state = blocked
runtime_kind = zoom_live_blocked_runtime
runtime_blockers contains zoom_live_authorization_required
```

### Check 3 - Start blocked live runtime

```http
POST /api/v1/integrations/zoom/runtime/start
```

Expected:

```text
status = blocked
healthy = false
live_runtime_available = false
zoom.runtime.status_changed emitted
```

### Check 4 - Complete live authorization

For OAuth user accounts:

```http
POST /api/v1/integrations/zoom/oauth/start
POST /api/v1/integrations/zoom/oauth/complete
```

For Server-to-Server accounts:

```http
POST /api/v1/integrations/zoom/oauth/server-to-server/authorize
```

Expected:

```text
lifecycle_state = authorized
runtime_kind = zoom_live_authorized_runtime
token_secret_ref points to a host_vault secret_reference
raw access_token, refresh_token and client_secret absent from account config
```

### Check 5 - Start authorized live runtime

```http
POST /api/v1/integrations/zoom/runtime/start
```

Expected:

```text
status = running
healthy = true
live_runtime_available = true
runtime_blockers empty unless token rotation or another runtime error is active
zoom.runtime.status_changed emitted
```

### Check 6 - Scheduled recording sync capability

Expected:

```text
provider_sync.recordings.scheduler = available
started authorized runtimes can be polled by the local recording-sync daemon
```

### Check 7 - Refresh authorized token bundle

```http
POST /api/v1/integrations/zoom/oauth/refresh
```

Expected:

```text
refreshed = true or skipped_not_expired
token_secret_ref points to the existing host_vault secret_reference
raw access_token, refresh_token and client_secret absent from response and account config
```

## Fixture bridge smoke check

### Check 8 - Register fixture account

```http
POST /api/v1/integrations/zoom/fixtures/accounts
```

Expected:

```text
auth_shape = fixture
lifecycle_state = fixture_ready
```

### Check 9 - Start fixture runtime

Expected:

```text
status = running
healthy = true
zoom.runtime.status_changed emitted
```

### Check 10 - Ingest meeting observation

Expected:

```text
provider call evidence upserted
zoom.meeting.observed emitted
correlation_id preserved or generated
```

### Check 11 - Ingest recording observation

Expected:

```text
zoom.recording.observed emitted
secret-like fields absent from event payload
```

### Check 12 - Ingest transcript observation

Expected:

```text
call transcript upserted
placeholder call created if transcript arrives first
zoom.transcript.observed emitted
```

## Future live smoke check before enabling runtime

Do not enable live runtime until these checks exist:

- OAuth and Server-to-Server authorization store tokens through HostVault
  secret references only.
- Token refresh never writes raw token to provider account config.
- Webhook signature verification rejects invalid signatures.
- Replay/idempotency guard handles repeated provider events.
- Recording worker writes local blobs through approved storage boundary.
- Transcript import records provenance and source reference.
- Runtime lifecycle actions are audited.
- Consent/no-hidden-recording policy is visible and enforced.

## Rollback check

Runtime removal should mark lifecycle as `removed`, not destructively delete
provider account history.

```http
POST /api/v1/integrations/zoom/runtime/remove
```

Expected:

```text
removed = true
removed_at set
GET /accounts excludes it by default
GET /accounts?include_removed=true includes it
```
