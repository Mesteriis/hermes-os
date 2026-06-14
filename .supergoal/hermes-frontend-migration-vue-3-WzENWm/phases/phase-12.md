SUPERGOAL_PHASE_START
Phase: 12 of 15 — Communications/Mail
Task: Port the Communications (mail) domain — the most complex domain — to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 9
Evidence required: build output, communications domain file listing
Depends on phases: 1, 2, 3

## Why

Communications (mail) is the most complex domain with mail list, message viewer with tabs, compose drawer, draft strip, health strip, account wizard, context inspector, and conversation list. It's the core ingestion spine of Hermes.

## Work

1. **Create communications domain** under `frontend/src/domains/communications/`:
   - `types/` — Communication types (ComposeFormModel, MailAccountOption, CommunicationListMessage, RenderedMessageContent, etc.)
   - `api/` — API functions (loadMailList, loadMessage, sendMessage, saveDraft, deleteDraft, loadConversations, loadContext, etc.)
   - `queries/` — TanStack Query hooks:
     - `useMailListQuery(accountId, folder)` — mail list with pagination
     - `useMessageQuery(messageId)` — single message detail
     - `useConversationsQuery(threadId)` — conversation thread
     - `useDraftsQuery()` — draft list
     - `useMailboxHealthQuery()` — mailbox health status
     - `useAccountOptionsQuery()` — mail account options
     - Mutations: useSendMailMutation, useSaveDraftMutation, useDeleteDraftMutation
   - `stores/` — Pinia stores for UI state only:
     - Selected communication, compose form state, drawer open state, active tab, send review state
   - `components/`:
     - MailList.vue — virtualized mail list (TanStack Virtual + TanStack Table)
     - MailListItem.vue — single mail row
     - MailViewer.vue — message detail with tabs
     - MessageBodyTab.vue — rendered HTML body in sandboxed iframe
     - MessageHeadersTab.vue — headers display
     - MessageAttachmentsTab.vue — attachment list
     - MessageRelatedTab.vue — related messages
     - MessageTimelineTab.vue — timeline for this message
     - CommunicationsContextInspector.vue — context analysis panel
     - CommunicationsContextRail.vue — context sidebar
     - CommunicationsConversationList.vue — conversation thread
     - ComposeDrawer.vue — compose/reply/forward drawer with TipTap editor
     - DraftStrip.vue — draft management strip
     - HealthStrip.vue — mailbox health indicator
     - AccountSetupModal.vue — account wizard modal
   - `views/`:
     - CommunicationsPage.vue — main page with widget layout
     - CommunicationsEmptyPage.vue — empty section placeholder
   - `routes/`

2. **Port compose functionality:**
   - Study existing compose in `frontend/src-svelte/lib/services/communications/compose.ts`
   - TipTap editor for rich text body
   - Support compose/reply/forward modes
   - Draft auto-save and management
   - Send with review step

3. **Port message rendering:**
   - Study `frontend/src-svelte/lib/services/communications/rendering.ts`
   - Render HTML email bodies in sandboxed iframe
   - Support text/plain fallback

4. **Register route** for `/communications`

5. **Verify:**
   - Build passes
   - Mail list loads with real data
   - Message viewer renders HTML content
   - Compose drawer opens and can save drafts
   - Draft strip shows managed drafts
   - Health strip shows mailbox status

## Acceptance criteria

- [ ] AC1: Mail list renders with virtual scrolling from API data
- [ ] AC2: Message viewer renders HTML content in sandboxed iframe
- [ ] AC3: Compose drawer supports compose/reply/forward modes with TipTap editor
- [ ] AC4: Draft strip shows and manages drafts (save, delete, open for editing)
- [ ] AC5: Account wizard modal renders provider selection flow
- [ ] AC6: Health strip shows mailbox health status from API
- [ ] AC7: Conversation list renders thread messages
- [ ] AC8: Context inspector shows AI analysis of selected message
- [ ] AC9: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output
- List of communications domain files

## Notes

- Reference `frontend/src-svelte/lib/pages/communications/` and `frontend/src-svelte/lib/services/communications/`
- This is the MOST COMPLEX domain — expect god-file risks. Enforce 500-line limit
- Decompose if any file approaches 500 lines
- Mail list virtualization is critical for performance (TanStack Virtual)
- TipTap is used for the compose editor
- HTML email rendering must use sandboxed iframe (srcdoc approach)
- Server state goes through TanStack Query, UI state through Pinia
- Compose form state is UI state (Pinia), email send is server mutation (TanStack Query)
