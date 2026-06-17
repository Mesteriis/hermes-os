# Telegram Implementation Status

Статус на 2026-06-17.

Проценты ниже описывают coverage только для Telegram Channel. Это не оценка
готовности Communications, Memory, Knowledge, Obligations, Decisions, Personas,
Organizations или Timeline.

## Summary Table

| § | Раздел | Статус | % |
|---|---|---|---:|
| 1 | Channel framing and ADR alignment | ✓ | 100 |
| 2 | Provider/account model | ◐ | 74 |
| 3 | Secret references and host-vault boundary | ✓ | 90 |
| 4 | Capability contract | ◐ | 83 |
| 5 | Fixture runtime | ✓ | 100 |
| 6 | TDLib QR user runtime | ◐ | 65 |
| 7 | Bot runtime | ✗ | 10 |
| 8 | Dialog/chat list | ◐ | 68 |
| 9 | Private chats | ◐ | 60 |
| 10 | Groups/supergroups/channels | ◐ | 45 |
| 11 | Topics/forums | ⚠ | 63 |
| 12 | Message ingestion/projection | ✓ | 88 |
| 13 | Message lifecycle commands | ◐ | 62 |
| 14 | Replies/forwards/pins | ◐ | 58 |
| 15 | Reactions | ◐ | 62 |
| 16 | Message history/versioning | ◐ | 55 |
| 17 | Media metadata/download/upload | ◐ | 68 |
| 18 | Attachments preview/search/dedup | ◐ | 43 |
| 19 | Voice/video messages | ◐ | 20 |
| 20 | Calls/transcripts | ◐ | 43 |
| 21 | Search | ◐ | 72 |
| 22 | Realtime | ◐ | 89 |
| 23 | AI assistance | ◐ | 20 |
| 24 | Frontend workbench | ◐ | 77 |
| 25 | Timeline/shared engine integration points | ◐ | 35 |
| 26 | Offline/outbox/export/proxy/session bundles | ◐ | 30 |
| 27 | Dialog management: unread/pinned/archive/mute/folders | ◐ | 68 |
| 28 | Provider-write command model | ◐ | 68 |
| 29 | Privacy/security/capability UX | ◐ | 50 |
| 30 | Documentation and audit set | ✓ | 100 |

## Navigation

- [Pass Log](status/pass-log.md)
- [Details Core (§2-§16)](status/details-core.md)
- [Details Extended (§17-§30)](status/details-extended.md)

## Recommended Next Implementation Order

1. Detailed capability matrix.
2. Telegram realtime event contracts.
3. Message lifecycle schema: versions, tombstones, replies, forwards, reactions.
4. Provider command/outbox model beyond send.
5. Dialog management: unread, pinned, archive, mute, folders.
6. Media gallery, search and preview parity.
7. Telegram-specific AI review surfaces.
8. Voice/calls with explicit native permission boundary.
