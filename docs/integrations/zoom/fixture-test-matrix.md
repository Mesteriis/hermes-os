# Zoom Fixture Test Matrix

Status date: 2026-06-27.

Fixture tests should validate the integration boundary without depending on
live provider access. The tests below are target coverage for the future Zoom
implementation.

## Account and runtime tests

| Scenario | Route | Fixture input | Expected result |
|---|---|---|---|
| Create fixture account | `POST /fixtures/accounts` | valid account id/display/external id | `provider_kind=zoom_user`, `auth_shape=fixture`, `lifecycle_state=fixture_ready`. |
| List accounts | `GET /accounts` | one fixture account | account returned and sorted by display name. |
| Get runtime status | `GET /runtime/status?account_id=...` | fixture account | `status=stopped`, `healthy=true`, ingest capabilities available. |
| Start fixture runtime | `POST /runtime/start` | fixture account | lifecycle becomes `running`, runtime event emitted. |
| Stop fixture runtime | `POST /runtime/stop` | running fixture account | lifecycle becomes `stopped`, runtime event emitted. |
| Remove fixture runtime | `POST /runtime/remove` | fixture account | lifecycle becomes `removed`, removed response returned. |
| List without removed | `GET /accounts` | removed account | removed account excluded. |
| List with removed | `GET /accounts?include_removed=true` | removed account | removed account included. |
| Register live blocked user | `POST /accounts` | `auth_shape=oauth_user` | `provider_kind=zoom_user`, `lifecycle_state=blocked`. |
| Register live blocked S2S | `POST /accounts` | `auth_shape=server_to_server` | `provider_kind=zoom_server_to_server`, `lifecycle_state=blocked`. |

## Bridge tests

| Scenario | Route | Fixture input | Expected result |
|---|---|---|---|
| Meeting observation | `POST /runtime-bridge/meetings` | account id and meeting id | provider call upserted, `zoom.meeting.observed` appended/broadcast. |
| Meeting with participants | `POST /runtime-bridge/meetings` | participant snapshots | participants preserved as metadata/evidence. |
| Recording observation | `POST /runtime-bridge/recordings` | recording id and meeting id | `zoom.recording.observed` appended/broadcast. |
| Transcript observation | `POST /runtime-bridge/transcripts` | transcript id, meeting id, text | placeholder call ensured, transcript upserted, event appended/broadcast. |
| Transcript without meeting call | `POST /runtime-bridge/transcripts` | transcript arrives first | placeholder provider call created with stable id. |
| VTT transcript file import | `POST /runtime-bridge/transcript-files` | VTT file text | transcript text and timed segments parsed, placeholder call ensured, event appended/broadcast. |
| SRT transcript file import | `POST /runtime-bridge/transcript-files` | SRT file text | transcript text and timed segments parsed. |
| Plain transcript file import | `POST /runtime-bridge/transcript-files` | plain text | transcript text imported with empty segment list. |
| Missing account | any bridge route | unknown account id | invalid request error. |
| Missing meeting id | meeting/recording/transcript bridge | empty meeting id | validation error. |
| Missing transcript text | transcript bridge | empty text | validation error. |
| Empty transcript file text | transcript file bridge | empty file text | validation error. |
| Malformed timed transcript file | transcript file bridge | invalid cue timestamp | validation error. |
| Malformed metadata | bridge route | metadata array/string | validation error. |

## Sanitization tests

Payloads containing these fields at any depth must not be appended or broadcast:

```text
access_token
refresh_token
token
client_secret
webhook_secret
download_token
password
```

Test cases:

| Scenario | Input location | Expected result |
|---|---|---|
| Top-level token | `metadata.access_token` | field removed. |
| Nested token | `metadata.auth.refresh_token` | field removed. |
| Recording download token | `recording.metadata.download_token` | field removed. |
| Participant password field | `participants[].metadata.password` | field removed. |

## Idempotency tests

Stable identifiers should be derived from stable inputs:

```text
call_id: account_id + meeting_id
event_id: kind + account_id + subject_id + optional observation_id
```

Expected behavior:

- repeated same meeting observation upserts same call id;
- same observation id produces same event id;
- changed observation id produces distinct event id for the same subject;
- causation/correlation are preserved when supplied.

## Example fixture payload set

```text
fixtures/zoom/account.fixture.json
fixtures/zoom/meeting.observed.json
fixtures/zoom/recording.observed.json
fixtures/zoom/transcript.observed.json
fixtures/zoom/security.sanitization.json
```

The current codebase does not yet include these fixture files. This matrix
defines the target fixture set.
