SUPERGOAL_PHASE_START
Phase: 6 of 10 — Frontend: Missing Stores & States
Task: Create WhatsApp and Organizations Pinia stores, add Loading/Empty/Error/Skeleton states to all components
Mandatory commands: cd frontend && pnpm build, cd frontend && pnpm lint
Acceptance criteria: 9
Evidence required: build output, store files listing, useComponentStates usage grep
Depends on phases: Phase 5

## Why

WhatsApp domain GAP-6 и Organizations domain GAP-6 не имеют Pinia stores для UI state. Большинство frontend компонентов GAP-7 не обрабатывают Loading/Empty/Error/Skeleton состояния. Это ухудшает UX и не соответствует стандартам качества.

## Work

### Part A: Create Missing Stores

1. **Organizations store:**
   - `frontend/src/domains/organizations/stores/organizations.ts`
     - State: filters, selection, search query, view mode
     - Actions: setFilters, setSelection, setSearch, setViewMode
     - Использовать TanStack Query для server state
     - Pinia setup store syntax

2. **WhatsApp store:**
   - `frontend/src/domains/whatsapp/stores/whatsapp.ts`
     - State: session filters, message filter, selected session
     - Actions: setSessionFilter, setMessageFilter, selectSession
     - Использовать TanStack Query для server state
     - Pinia setup store syntax

3. **Update components to use stores:**
   - OrganizationsPage — использовать organizations store
   - OrganizationDetail — использовать organizations store
   - WhatsAppPage — использовать whatsapp store

### Part B: Add UI States to All Components

4. **Create shared composable:**
   - `frontend/src/shared/composables/useComponentStates.ts`
     - Returns: `{ isLoading, isEmpty, hasError, error, skeletonItems }`
     - Standardized pattern for all components
     - Reusable with any query result

5. **Add states to components using inventory from Phase 1:**
   - TelegramChatList — Loading/Skeleton, Empty, Error с retry
   - WhatsAppSessionList — Loading/Skeleton, Empty, Error с retry
   - DocumentsList — Loading/Skeleton, Empty, Error с retry
   - NotesList — Loading/Skeleton, Empty, Error с retry
   - PersonsList — Loading/Skeleton, Empty, Error с retry
   - OrganizationsDetail — Loading/Skeleton, Empty state
   - OrganizationsList — Loading/Skeleton, Empty, Error с retry
   - CommunicationsPanel components — Loading/Skeleton, Empty, Error

6. **Verify:**
   - Визуальная проверка каждого компонента в 4 состояниях
   - Skeleton компонент `shared/ui/Skeleton.vue` используется везде
   - Empty state содержит иконку + сообщение + action CTA
   - Error state содержит сообщение + retry button

## Acceptance criteria (all must pass)

- [ ] AC1: Organizations store создан и экспортируется
- [ ] AC2: WhatsApp store создан и экспортируется
- [ ] AC3: OrganizationsPage использует organizations store для UI state
- [ ] AC4: WhatsAppPage использует whatsapp store для UI state
- [ ] AC5: useComponentStates composable существует и используется в >= 5 компонентах
- [ ] AC6: Каждый проверенный компонент показывает Skeleton при загрузке
- [ ] AC7: Каждый компонент показывает Empty state при пустом списке
- [ ] AC8: Каждый компонент показывает Error state с retry button при ошибке
- [ ] AC9: `cd frontend && pnpm build` passes exit 0

## Mandatory commands (run each, surface last ~10 lines + exit code)

- `cd frontend && pnpm build`
- `cd frontend && pnpm lint`

## Evidence required in transcript

- Build output — last 10 lines showing success
- List of files in `frontend/src/domains/organizations/stores/` and `frontend/src/domains/whatsapp/stores/`
- grep for `useComponentStates` — confirming usage in multiple components
- Visual confirmation of Loading/Empty/Error states screenshots or descriptions

## Notes

- Использовать существующий `shared/ui/Skeleton.vue` — не создавать новый
- Pinia setup store syntax `defineStore name, () => { ... }` — соответствует проекту
- Empty state pattern: иконка + сообщение + кнопка действия
- Error state pattern: сообщение об ошибке + retry button
- Все новые stores должны быть typed — никаких `any`
- Reference persons store `frontend/src/domains/personas/stores/personas.ts` как pattern
