# WhatsApp API Reference

Статус: целевой API scope и стартовый route audit на 2026-06-17.

This package intentionally treats production WhatsApp Channel implementation as
not yet existing. Routes below are target contracts, not confirmed implemented
routes.

Все будущие protected маршруты должны использовать локальный API guard из
ADR-0056. Browser WebSocket clients передают local secret через `hermes_secret`,
потому что native WebSocket requests не могут выставить `X-Hermes-Secret`.

## Base

Business/read-model routes stay under:

```text
/api/v1/communications/whatsapp
```

Runtime/setup/account-control routes stay under:

```text
/api/v1/integrations/whatsapp
```

Provider kind:

```text
whatsapp_web
```

Account kinds:

```text
whatsapp_personal
whatsapp_business
```

## Capability Contract

### Целевые маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/integrations/whatsapp/capabilities` | Global operation-level capability matrix |
| GET | `/api/v1/integrations/whatsapp/accounts/{account_id}/capabilities` | Account-scoped capability matrix with lifecycle/runtime overrides |

### Целевые capability states

```text
available
degraded
blocked
unsupported
```

### Required operation groups

- account lifecycle;
- WebView session lifecycle;
- runtime;
- sync;
- read/search;
- send;
- reply;
- forward;
- reaction;
- delete;
- media upload;
- media download;
- join group;
- leave group;
- status read;
- status publish;
- voice send;
- calls metadata;
- export;
- secret/session access.

### Minimal operation matrix

| Operation | Action class | Initial status |
|---|---|---|
| `messages.send` | `provider_write` | `blocked` |
| `messages.reply` | `provider_write` | `blocked` |
| `messages.forward` | `provider_write` | `blocked` |
| `messages.react` | `provider_write` | `blocked` |
| `messages.delete` | `destructive` | `blocked` |
| `media.upload` | `provider_write` | `blocked` |
| `media.download` | `read/local_write` | `blocked` |
| `groups.join` | `provider_write` | `blocked` |
| `groups.leave` | `provider_write/destructive` | `blocked` |
| `status.read` | `read` | `blocked` |
| `status.publish` | `provider_write` | `blocked` |
| `voice.send` | `provider_write` | `blocked` |
| `calls.metadata.read` | `read` | `blocked` |
| `calls.control` | `unsupported` | `unsupported` |
| `calls.record` | `unsupported` | `unsupported` |
| `calls.stt` | `unsupported` | `unsupported` |

Capability payloads should include:

```text
operation
category
status
action_class
reason
confirmation_required
closure_gate
account_scope
runtime_kind
provider_kind
```

## Accounts

### Целевые маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/integrations/whatsapp/accounts?include_removed=` | List WhatsApp account metadata |
| POST | `/api/v1/integrations/whatsapp/accounts` | Create account metadata and secret/session references |
| DELETE | `/api/v1/integrations/whatsapp/accounts/{account_id}` | Mark account removed, stop runtime, preserve evidence |
| POST | `/api/v1/integrations/whatsapp/accounts/{account_id}/logout` | End local session, stop runtime, preserve evidence |

Account config stores non-secret metadata only.

Allowed account kinds:

```text
whatsapp_personal
whatsapp_business
```

`whatsapp_business` is still WhatsApp Web companion for WhatsApp Business App.
It is not Meta Business Platform Cloud API.

### Недостающие маршруты

All account routes are missing in this starting production audit.

Future routes may include encrypted session export/import only after a dedicated
ADR and host-vault-backed secret policy.

## WebView Session Lifecycle

### Целевые маршруты

| Method | Path | Описание |
|---|---|---|
| POST | `/api/v1/integrations/whatsapp/sessions/link/start` | Start owner-visible WhatsApp Web link flow |
| GET | `/api/v1/integrations/whatsapp/sessions/link/{setup_id}` | Poll link status |
| DELETE | `/api/v1/integrations/whatsapp/sessions/link/{setup_id}` | Cancel pending link flow |
| GET | `/api/v1/integrations/whatsapp/sessions?account_id=` | List local session metadata |
| POST | `/api/v1/integrations/whatsapp/runtime/start` | Start account-scoped companion runtime |
| POST | `/api/v1/integrations/whatsapp/runtime/stop` | Stop account-scoped companion runtime |
| GET | `/api/v1/integrations/whatsapp/runtime/status?account_id=` | Runtime status, blockers and capability diagnostics |

Target runtime states:

```text
not_configured
link_pending
linked
starting
running
degraded
blocked
stopped
expired
failed
logged_out
removed
```

Session payloads must not include cookies, local profile secrets, pairing codes
or message bodies.

## Dialogs / Chats

### Целевые маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/communications/whatsapp/chats?account_id=&kind=&limit=&cursor=` | Projected WhatsApp dialogs ordered by last activity |
| GET | `/api/v1/communications/whatsapp/chats/{chat_id}` | Read projected chat/detail metadata |
| POST | `/api/v1/communications/whatsapp/sync/chats` | Sync dialogs through validated runtime |
| POST | `/api/v1/communications/whatsapp/chats/join` | Queue group/community join command when supported |
| POST | `/api/v1/communications/whatsapp/chats/{chat_id}/leave` | Queue leave command |

Supported dialog kinds:

```text
private
group
community
broadcast
status
```

Join/leave routes are provider-write commands. They must write command rows and
must not mutate projected membership until provider-observed evidence confirms
the state.

## Participants

### Целевые маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/communications/whatsapp/chats/{chat_id}/participants?query=&role=&limit=&cursor=` | Read participant evidence |
| POST | `/api/v1/communications/whatsapp/chats/{chat_id}/participants/sync` | Refresh participant projection when provider/runtime allows |

Participant evidence may include:

- phone number;
- display name;
- `wa_id`;
- group member state;
- admin state;
- community member state.

These routes expose identity traces. They do not create Persona records or merge
identity candidates.

## Messages

### Целевые маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/communications/whatsapp/messages?account_id=&chat_id=&limit=&cursor=` | Read projected messages |
| GET | `/api/v1/communications/whatsapp/messages/{message_id}` | Read a projected message |
| GET | `/api/v1/communications/whatsapp/messages/{message_id}/raw` | Read sanitized append-only raw provider evidence |
| POST | `/api/v1/communications/whatsapp/messages/send` | Queue text send command |
| POST | `/api/v1/communications/whatsapp/messages/{message_id}/reply` | Queue reply command |
| POST | `/api/v1/communications/whatsapp/messages/{message_id}/forward` | Queue forward command |
| POST | `/api/v1/communications/whatsapp/messages/{message_id}/delete` | Queue delete/tombstone command |

Supported message classes:

```text
text
reply
forward
reaction
delete
edit
```

`edit` is target-only and remains blocked unless provider/runtime support is
verified.

## Reactions

### Целевые маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/communications/whatsapp/messages/{message_id}/reactions` | Read projected reaction evidence |
| POST | `/api/v1/communications/whatsapp/messages/{message_id}/reactions` | Queue add/change reaction command |
| DELETE | `/api/v1/communications/whatsapp/messages/{message_id}/reactions/{reaction_id}` | Queue remove reaction command |

Reaction writes require capability checks, command rows, redacted audit and
provider-observed reconciliation.

## Media

### Целевые маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/communications/whatsapp/media?account_id=&chat_id=&kind=&limit=&cursor=` | Search/list projected media |
| POST | `/api/v1/communications/whatsapp/media/download` | Queue or start provider media download |
| POST | `/api/v1/communications/whatsapp/media/upload` | Queue media upload/send command from local Communication attachment/blob |
| GET | `/api/v1/communications/whatsapp/media/{media_id}` | Read media metadata and local availability |

Supported media classes:

```text
photo
video
document
audio
voice_note
contact
location
sticker
gif
```

Media upload must accept local `attachment_id` or `blob_id`. UI must not upload
directly to the WebView/provider runtime.

Download progress is reported through:

```text
whatsapp.media.download.started
whatsapp.media.download.progress
whatsapp.media.download.completed
whatsapp.media.download.failed
```

## Attachments

WhatsApp media must use shared Communication attachment/blob APIs where
possible. Provider-specific routes should only orchestrate WhatsApp runtime
transfer and projection reconciliation.

Attachment metadata must preserve:

- provider attachment id;
- provider message id;
- media kind;
- filename when observed;
- MIME type when observed;
- size when observed;
- hash after local persistence;
- scanner state;
- local blob reference.

## WhatsApp Statuses

### Целевые маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/communications/whatsapp/statuses?account_id=&limit=&cursor=` | Read projected status evidence |
| GET | `/api/v1/communications/whatsapp/statuses/{status_id}` | Read a status evidence record |
| POST | `/api/v1/communications/whatsapp/statuses/publish` | Queue status publish command |
| POST | `/api/v1/communications/whatsapp/statuses/{status_id}/reply` | Queue status reply when supported |

WhatsApp Status is:

```text
source evidence
timeline evidence
identity signal
```

It is not a separate domain. Status publish is a provider-write command and
must use the outbox.

## Voice Notes

### Целевые маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/communications/whatsapp/voice?account_id=&chat_id=&limit=&cursor=` | Read voice-note attachment metadata |
| POST | `/api/v1/communications/whatsapp/voice/download` | Download voice note attachment |
| POST | `/api/v1/communications/whatsapp/voice/send` | Queue voice send command from local blob |

Supported:

- metadata;
- attachment;
- playback through local blob;
- future transcript integration point.

Not supported in the first version:

- STT;
- hidden recording;
- automatic transcription;
- voice capture.

## Calls

### Целевые маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/communications/whatsapp/calls?account_id=&chat_id=&limit=&cursor=` | Read call metadata evidence |
| GET | `/api/v1/communications/whatsapp/calls/{call_id}` | Read single call evidence record |

Supported in first version:

- call metadata;
- call evidence;
- call timeline entries.

Out of scope:

- audio capture;
- video capture;
- call control;
- recording;
- live call handling;
- STT.

No call control route should be added before a future ADR.

## Search

### Целевые маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/communications/whatsapp/search/messages?q=&account_id=&chat_id=&limit=&cursor=` | Search local projected messages |
| GET | `/api/v1/communications/whatsapp/search/media?q=&account_id=&kind=&limit=&cursor=` | Search projected media |
| GET | `/api/v1/communications/whatsapp/search/participants?q=&account_id=&limit=&cursor=` | Search identity traces and participant evidence |
| POST | `/api/v1/communications/whatsapp/search/provider` | Provider search attempt when runtime capability is available |

Provider search results that are not projected locally must be marked as
evidence candidates until raw source records are preserved.

## Provider Command Outbox

### Целевые маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/integrations/whatsapp/commands?account_id=&chat_id=&status=&limit=&cursor=` | List provider-write command rows |
| GET | `/api/v1/integrations/whatsapp/commands/{command_id}` | Read command state and redacted result metadata |
| POST | `/api/v1/integrations/whatsapp/commands/{command_id}/retry` | Retry failed or dead-letter command when policy allows |

Required command states:

```text
queued
executing
retrying
completed
failed
dead_letter
```

Command rows should cover:

- send;
- reply;
- forward;
- reaction;
- delete;
- media upload;
- media download when modeled as command;
- join group;
- leave group;
- status publish;
- voice send.

## Realtime / Events

### Generic routes

| Method | Path | Описание |
|---|---|---|
| GET | `/api/events/ws?after_position=&hermes_secret=` | Protected WebSocket event stream with replay/heartbeat |
| GET | `/api/events/stream?after_position=` | Protected SSE stream |
| GET | `/api/v1/events?after_position=&limit=&wait_seconds=` | Protected JSON replay/long-poll fallback |
| POST | `/api/v1/events` | Local event API command boundary |
| GET | `/api/v1/events/{event_id}` | Read single event |

### WhatsApp realtime contracts

```text
whatsapp.message.created
whatsapp.message.updated
whatsapp.message.deleted

whatsapp.chat.updated

whatsapp.reaction.changed

whatsapp.media.download.started
whatsapp.media.download.progress
whatsapp.media.download.completed
whatsapp.media.download.failed

whatsapp.command.status_changed
whatsapp.command.reconciled
```

Event payloads must include enough identifiers for cache patching and replay:

- account_id;
- provider_chat_id when available;
- projected chat id when available;
- provider_message_id when available;
- communication_message_id when available;
- command_id when relevant;
- status/reason fields when relevant;
- redacted projected snapshots when safe.

Event payloads must not include:

- message bodies;
- media bytes;
- secrets;
- pairing codes;
- cookies;
- raw provider payloads.

## Frontend API Client

Target frontend client modules:

```text
frontend/src/integrations/whatsapp/api/
frontend/src/integrations/whatsapp/queries/
frontend/src/integrations/whatsapp/stores/
```

Covered target areas:

- capabilities;
- accounts;
- sessions/runtime;
- dialogs;
- messages;
- participants;
- reactions;
- media;
- statuses;
- voice notes;
- call metadata;
- provider command outbox.

Not covered in this starting audit because implementation is not counted:

- all production routes;
- WebView runtime;
- provider command execution;
- provider-observed reconciliation;
- realtime cache patching.
