# Telegram API Reference — Media and Search

См. также: [API Index](../api.md), [Conversations](conversations.md) и [Operations and Realtime](operations-realtime.md).

## Media

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| POST | `/api/v1/telegram/media/download` | Download TDLib file and persist completed local blob + Communication attachment row |
| POST | `/api/v1/telegram/media/upload` | Queue provider-side media send from a local `attachment_id` or `blob_id` through the durable Telegram command outbox |
| GET | `/api/v1/telegram/search/media?account_id=&provider_chat_id=&kind=&limit=` | Projected Telegram media gallery/filter with attachment download metadata for files-tab parity |

Fixture runtime intentionally fails closed for media downloads.
Media download requests emit `telegram.media.download.started` before runtime
dispatch, `telegram.media.download.failed` when the runtime rejects or fails the
request, and `telegram.media.download.progress` when TDLib returns a
non-completed file snapshot. Completed TDLib downloads also update projected
Telegram message attachment metadata (`download_state`, `local_path`,
`attachment_id`, `tdlib_file_id`) and emit `telegram.media.downloaded` with
sanitized projected `message` and `chat` snapshots for realtime cache patching.
Message ingestion now also derives remote attachment metadata for TDLib
`messageSticker`, `messageAnimation` and `messageVideoNote` payloads. Those rows
reuse the existing projected `attachments` metadata contract and can be listed
by the files tab, downloaded through the existing media route, and previewed
after local download.
The thread Files and Voice tabs derive a current-chat download queue from the
same projected attachment readiness fields (`download_state`, `local_path`,
`tdlib_file_id`, `provider_attachment_id`, size/progress metadata) and expose
retry actions without a separate Telegram polling surface.

Media upload uses the provider command model:

```text
POST /api/v1/communications/attachments/import
  -> communication_mail_blobs + communication_attachment_imports
POST /api/v1/telegram/media/upload
  -> telegram_provider_write_commands(command_kind=send_media)
  -> TDLib sendMessage media builder
```

`/api/v1/telegram/media/upload` does not accept raw file bytes and does not call
TDLib directly from UI. It accepts a local `attachment_id` or `blob_id`, rejects
malicious imported attachments, records provider-write audit metadata and emits
`telegram.media.upload.started` plus command status events. Provider completion
is only recorded after the outbox executor receives a provider message snapshot.

Supported backend media send types:

```text
photo
video
document
audio
voice
sticker
animation / gif
```

The workspace search UI consumes the same media search response, includes media
hits in the result count and projects local-preview/download readiness from
`local_path`, `tdlib_file_id`, `provider_attachment_id` and `download_state`.
When a downloaded Telegram attachment has a projected Communication
`attachment_id`, the Telegram media viewer can call the shared
`GET /api/v1/communications/attachments/{attachment_id}/preview` route for safe
text/image previews. That shared route enforces local blob, scan-status and
size-limit checks; Hermes still does not expose a Telegram-specific preview
bytes route.
TDLib messages with `media_album_id` now also project album grouping metadata
into the canonical message payload:

```text
metadata.media_album_id
metadata.media_album_key = provider_chat_id + media_album_id
```

The Telegram Files tab can group already-loaded selected-chat messages by this
key. Provider-wide album sync and album-aware media upload/send are still
missing.

Media download required fields:

```text
account_id
provider_chat_id
provider_message_id
tdlib_file_id
```

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/media` | Media gallery/search |
| GET | `/api/v1/telegram/media/{media_id}/preview` | Telegram workbench preview |
| GET | `/api/v1/telegram/media/search?q=&type=` | Provider/local media search |
| POST | `/api/v1/telegram/voice/send` | Voice message send |
| POST | `/api/v1/telegram/video-note/send` | Video note send |

## Attachments

Telegram currently reuses shared Communication attachment APIs after media download.
The Telegram Files tab now exposes the shared
`/api/v1/communications/attachments/search` index through a Telegram-scoped
panel limited by the selected chat account id, so downloaded Telegram
attachments can be found by filename/content type/scan status without creating
a Telegram-specific attachment store.

Target: Telegram workbench should expose Telegram-specific attachment views while
storage remains provider-neutral.

## Search

### Текущие возможности

- local chat title filter in UI;
- local loaded thread text filter;
- shared Communication search can include Telegram-projected messages by `channel_kind`.
- workspace dialog search UI backed by `GET /api/v1/telegram/chats/search`;
- workspace message search UI backed by `GET /api/v1/telegram/search/messages`;
- explicit provider search command `POST /api/v1/telegram/search/provider`;
- workspace media search UI plus files tab gallery backed by `GET /api/v1/telegram/search/media`.
- downloaded local Telegram media can open in a read-only viewer using projected
  attachment metadata and local blob paths.
- saved-search create/select/delete UI reuses the shared Communication
  `/api/v1/communications/saved-searches` API with `channel_kind=telegram` and
  selected Telegram account scope. No Telegram-specific saved-search route is
  introduced.

### Недостающие маршруты
| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/telegram/search/provider` | Не требуется: command endpoint реализован как POST для явного provider search. |

Current search contract notes:

- `GET /api/v1/telegram/chats/search` searches projected Telegram chats by
  title and optional `account_id` scope.
- `GET /api/v1/telegram/search/messages` searches projected Telegram messages by
  text and optional `account_id` / `provider_chat_id` scope. Если `account_id` указан, в запрос
  также инициируется TDLib provider search для актуализации projection.
- `POST /api/v1/telegram/search/provider` now exposes provider search as explicit
  capability-aligned command endpoint and returns full projection response after
  provider attempt.
- The workspace search results panel renders a provider/local/fallback search
  source label derived from selected-account runtime status, so provider-backed
  searches are visible without requiring a separate provider-only UI.
- `GET /api/v1/telegram/search/media` is a projection-backed gallery/filter
  endpoint. It currently supports optional free-text `q`, scope and `kind`,
  attempts TDLib provider message search first when both `q` and `account_id`
  are present, and then returns projected attachment download metadata
  (`tdlib_file_id`, `provider_attachment_id`, `local_path`) so files-tab gallery
  rows can keep preview/download parity outside the currently loaded message
  window. The response includes `source`, `provider_search_attempted` and
  `provider_search_error` to distinguish provider-refresh results from
  projection-only fallback.
