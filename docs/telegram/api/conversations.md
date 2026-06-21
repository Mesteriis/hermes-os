# Telegram API Reference — Conversations

See also: [API Index](../api.md), [Foundation](foundation.md), and [Media and Search](media-search.md).

## Ownership

Telegram runtime, sync, provider search, provider commands, and setup APIs live under:

```text
/api/v1/integrations/telegram/*
```

Communications business state remains provider-neutral and lives under:

```text
/api/v1/communications/*
```

## Current Telegram Runtime Routes

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/integrations/telegram/conversations?account_id=&limit=` | Projected Telegram chat list |
| GET | `/api/v1/integrations/telegram/conversations/{telegram_chat_id}` | Projected chat detail |
| GET | `/api/v1/integrations/telegram/conversations/{telegram_chat_id}/members` | Projected member list |
| POST | `/api/v1/integrations/telegram/provider-conversations/{telegram_chat_id}/members/sync` | Provider-backed member sync |
| GET | `/api/v1/integrations/telegram/conversations/{telegram_chat_id}/pinned-messages` | Projected pinned messages |
| GET | `/api/v1/integrations/telegram/provider-conversations/search` | Provider-scoped chat search assist |
| GET | `/api/v1/integrations/telegram/conversation-folders` | Provider folder projection |
| POST | `/api/v1/integrations/telegram/provider-conversations/join` | Join command |
| POST | `/api/v1/integrations/telegram/provider-conversations/{telegram_chat_id}/leave` | Leave command |
| POST | `/api/v1/integrations/telegram/provider-conversations/{telegram_chat_id}/pin` | Pin command |
| POST | `/api/v1/integrations/telegram/provider-conversations/{telegram_chat_id}/unpin` | Unpin command |
| POST | `/api/v1/integrations/telegram/provider-conversations/{telegram_chat_id}/archive` | Archive command |
| POST | `/api/v1/integrations/telegram/provider-conversations/{telegram_chat_id}/unarchive` | Unarchive command |
| POST | `/api/v1/integrations/telegram/provider-conversations/{telegram_chat_id}/mute` | Mute command |
| POST | `/api/v1/integrations/telegram/provider-conversations/{telegram_chat_id}/unmute` | Unmute command |
| POST | `/api/v1/integrations/telegram/provider-conversations/{telegram_chat_id}/read` | Provider read reconciliation command |
| POST | `/api/v1/integrations/telegram/provider-conversations/{telegram_chat_id}/unread` | Provider unread reconciliation command |
| POST | `/api/v1/integrations/telegram/provider-conversations/{telegram_chat_id}/folders/{provider_folder_id}` | Add folder assignment |
| POST | `/api/v1/integrations/telegram/provider-conversations/{telegram_chat_id}/folders/{provider_folder_id}/remove` | Remove folder assignment |
| POST | `/api/v1/integrations/telegram/provider-conversations/{telegram_chat_id}/folders/reassign` | Replace folder assignments |
| POST | `/api/v1/integrations/telegram/provider-sync/chats` | Provider chat sync |
| POST | `/api/v1/integrations/telegram/provider-sync/history` | Provider history sync |

## Topics

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/integrations/telegram/conversations/{telegram_chat_id}/topics` | Topic list |
| POST | `/api/v1/integrations/telegram/conversations/{telegram_chat_id}/topics` | Create topic command |
| GET | `/api/v1/integrations/telegram/topics/{topic_id}` | Topic detail |
| POST | `/api/v1/integrations/telegram/topics/{topic_id}/close` | Close or reopen topic |
| GET | `/api/v1/integrations/telegram/topics/{topic_id}/messages` | Topic-scoped message list |
| GET | `/api/v1/integrations/telegram/topics/search` | Topic search |

## Message Runtime Routes

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/integrations/telegram/messages?account_id=&provider_chat_id=&limit=` | Projected Telegram message list |
| POST | `/api/v1/integrations/telegram/messages` | Fixture ingest |
| POST | `/api/v1/integrations/telegram/messages/send` | Manual runtime send |
| POST | `/api/v1/integrations/telegram/messages/{message_id}/reply` | Runtime reply |
| POST | `/api/v1/integrations/telegram/messages/{message_id}/forward` | Runtime forward |
| POST | `/api/v1/integrations/telegram/messages/{message_id}/edit` | Edit command |
| POST | `/api/v1/integrations/telegram/messages/{message_id}/delete` | Delete command |
| POST | `/api/v1/integrations/telegram/messages/{message_id}/restore-visibility` | Restore visibility command |
| POST | `/api/v1/integrations/telegram/messages/{message_id}/pin` | Pin command |
| POST | `/api/v1/integrations/telegram/messages/{message_id}/mark-read` | Message-level read command |
| GET | `/api/v1/integrations/telegram/messages/{message_id}/versions` | Version list |
| GET | `/api/v1/integrations/telegram/messages/{message_id}/tombstones` | Tombstone list |
| GET | `/api/v1/integrations/telegram/messages/{message_id}/raw` | Sanitized raw evidence |
| GET | `/api/v1/integrations/telegram/messages/{message_id}/reply-chain` | Reply chain |
| GET | `/api/v1/integrations/telegram/messages/{message_id}/forward-chain` | Forward chain |
| GET/POST/DELETE | `/api/v1/integrations/telegram/messages/{message_id}/reactions` | Reaction projection and commands |

## Notes

- Provider search and provider sync remain integration-scoped runtime surfaces.
- Communication business UI should consume `/api/v1/communications/*` routes and not call these provider runtime routes directly.
- Provider observations are projected on the application side; integrations do not own communication business mutations.
