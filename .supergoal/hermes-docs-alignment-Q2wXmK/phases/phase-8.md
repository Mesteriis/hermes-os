SUPERGOAL_PHASE_START
Phase: 8 of 10 — Telegram Module Parity
Task: Achieve full parity with Telegram Desktop — accounts, chats, messages, attachments, voice/video, channels, groups, forums, search, drafts, notifications, media gallery
Mandatory commands: cargo build && cargo test --all, cd frontend && pnpm build
Acceptance criteria: 10
Evidence required: build output, Telegram parity checklist, Telegram store line count
Depends on phases: Phase 3

## Why

Telegram модуль GAP-9 имеет частичную реализацию. Требуется паритет с Telegram Desktop для ключевых сценариев, чтобы пользователи могли полностью управлять Telegram через Hermes.

## Work

### Backend

1. **Missing API endpoints:**
   - Media gallery endpoint — list/search all media types photos, videos, documents
   - Drafts persistence — save/load/update drafts per chat
   - Notification preferences — get/set per-chat notifications
   - Message search integration with Tantivy — full-text search over messages

2. **Channel/group/forum support:**
   - Channel metadata expansion
   - Group member list endpoint
   - Forum topic support

3. **Backend models and store updates:**
   - Add media-related types
   - Add draft types
   - Add notification preference types
   - Update store modules if needed

### Frontend

4. **Media gallery component:**
   - `frontend/src/domains/telegram/components/TelegramMediaGallery.vue`
   - Tabs: Photos, Videos, Documents, Links
   - Grid layout with lazy loading
   - Preview on click

5. **Voice message UI:**
   - Play/pause controls
   - Waveform visualization placeholder
   - Download option

6. **Message search UI:**
   - Search input in TelegramPanel header
   - Search results list with highlights
   - Tantivy-backed search API

7. **Chat folders/tabs:**
   - Tab bar: All, Personal, Groups, Channels, Forums
   - Filter chat list by folder

8. **Message threading:**
   - Reply chain visualization
   - Reply indicator in message list
   - Jump to replied message

9. **Online/typing indicators:**
   - Online status dot
   - Typing indicator animation

10. **Telegram store refactor:**
    - Выделить business logic helpers из store в composables
    - Store < 300 строк было 462

## Acceptance criteria (all must pass)

- [ ] AC1: Media gallery отображает photos, videos, documents в grid layout
- [ ] AC2: Voice messages воспроизводятся в UI play/pause
- [ ] AC3: Message search работает — отправляет запрос к backend и отображает результаты
- [ ] AC4: Chat folders/tabs работают — фильтрация чатов по типу
- [ ] AC5: Reply chains отображаются с возможностью jump to replied message
- [ ] AC6: Online/typing indicators отображаются корректно
- [ ] AC7: Telegram store < 300 строк было 462
- [ ] AC8: `cargo build && cargo test --all` passes
- [ ] AC9: `cd frontend && pnpm build` passes
- [ ] AC10: `make backend-validate` passes

## Mandatory commands (run each, surface last ~10 lines + exit code)

- `cargo build && cargo test --all`
- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output — last 10 lines from both backend and frontend
- Telegram parity checklist — each item marked completed
- Telegram store line count: `wc -l frontend/src/domains/telegram/stores/telegram.ts`
- List of new Telegram components

## Notes

- Разбить на must-have и nice-to-have если объём слишком большой
- Must-have: media gallery, search, message threading, chat folders
- Nice-to-have: voice messages, typing indicators
- Backend changes should be minimal — focus on frontend
- Reference `docs/domains/telegram-channel.md` for architecture
- Telegram store refactor: только UI state в store, business logic в composables
