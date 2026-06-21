# Telegram API Reference — Media and Search

See also: [API Index](../api.md), [Conversations](conversations.md), and [Operations and Realtime](operations-realtime.md).

## Ownership

Telegram provider runtime and debug surfaces live under:

```text
/api/v1/integrations/telegram/*
```

Shared Communication attachment and business read-model APIs live under:

```text
/api/v1/communications/*
```

## Media Routes

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/integrations/telegram/provider-media/download` | Download TDLib media and persist local attachment/blob state |
| POST | `/api/v1/integrations/telegram/provider-media/upload` | Queue provider-side media send from a local attachment or blob |
| GET | `/api/v1/integrations/telegram/provider-search/media` | Provider-assisted media search/filter |
| GET | `/api/v1/communications/attachments/{attachment_id}/preview` | Shared safe preview endpoint |
| POST | `/api/v1/communications/attachments/import` | Shared local attachment import |
| GET | `/api/v1/communications/attachments/search` | Shared attachment search |

## Search Routes

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/integrations/telegram/provider-search/messages` | Provider-assisted message search |
| POST | `/api/v1/integrations/telegram/provider-search/provider` | Explicit provider search trigger |
| GET | `/api/v1/communications/conversations/search` | Provider-assisted chat search |
| GET | `/api/v1/communications/search` | Hermes business search over the projected read-model |
| GET | `/api/v1/communications/saved-searches` | Shared saved-search surface |

## Notes

- Provider media upload does not accept raw file bytes; it works from already imported local attachments/blobs.
- Shared Communication attachment APIs remain the canonical safe access path for local preview and import.
- Normal user-facing Communication search should use `/api/v1/communications/search`; provider search routes are runtime/debug/sync-assist surfaces.
