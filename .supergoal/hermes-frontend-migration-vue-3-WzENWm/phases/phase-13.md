SUPERGOAL_PHASE_START
Phase: 13 of 15 — Telegram & WhatsApp
Task: Port Telegram and WhatsApp messaging domain pages to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 7
Evidence required: build output, telegram and whatsapp domain file listings
Depends on phases: 1, 2, 3

## Why

Telegram and WhatsApp are messaging sub-sections of the Communications view. They share message thread UI patterns and validate real-time-first approach for chat interfaces.

## Work

1. **Create telegram domain** under `frontend/src/domains/telegram/`:
   - Types, API, queries (TanStack Query hooks for chat list, messages, status)
   - Stores (Pinia for UI state: selected chat, active thread)
   - Components:
     - TelegramChatList.vue — virtualized chat list (TanStack Virtual)
     - TelegramMessageThread.vue — message thread display
     - TelegramRail.vue — chat details rail
     - TelegramCommandHeader.vue — command input header
     - TelegramActionRail.vue — action buttons rail
     - TelegramStatusMessages.vue — system status messages
   - Views/TelegramPage.vue — main page
   - Routes

2. **Create whatsapp domain** under `frontend/src/domains/whatsapp/`:
   - Types, API, queries, stores
   - Components:
     - WhatsAppSessionList.vue — session list with virtualization
     - WhatsAppMessageThread.vue — message thread
     - WhatsAppRail.vue — session details rail
   - Views/WhatsAppPage.vue — main page
   - Routes

3. **Update communications navigation:**
   - The Communications route handles sub-navigation for mail/telegram/whatsapp sections
   - These domains render as sub-views of the communications workspace

4. **Register routes** for `/communications/telegram` and `/communications/whatsapp`

5. **Verify:**
   - Build passes
   - Telegram chat list renders with virtual scrolling
   - WhatsApp session list renders

## Acceptance criteria

- [ ] AC1: Telegram chat list renders with virtual scrolling
- [ ] AC2: Telegram message thread renders messages with correct formatting
- [ ] AC3: Telegram rail shows chat details and metadata
- [ ] AC4: WhatsApp session list renders with virtualization
- [ ] AC5: WhatsApp message thread renders messages
- [ ] AC6: Both domains load data from API via TanStack Query
- [ ] AC7: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output

## Notes

- Reference `frontend/src-svelte/lib/pages/telegram/` and `frontend/src-svelte/lib/pages/whatsapp/`
- Use TanStack Virtual for chat/session list virtualization
- These are communication sub-sections accessed via the Communications view
- Telegram has more complex UI (chat list + thread + rail + command header + action rail + status messages)
- WhatsApp is simpler (session list + thread + rail)
