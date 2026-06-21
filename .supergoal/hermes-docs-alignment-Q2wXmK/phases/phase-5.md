SUPERGOAL_PHASE_START
Phase: 5 of 10 — Frontend: God Component Refactoring
Task: Break CommunicationsPage 891 into components under 500 lines, extract query logic into composables, eliminate raw fetch, fix cross-domain imports in review store
Mandatory commands: cd frontend && pnpm build, cd frontend && pnpm lint
Acceptance criteria: 10
Evidence required: build output, CommunicationsPage line count, review store imports verification, composables list
Depends on phases: Phase 3

## Why

CommunicationsPage.vue 891 строк превышает порог 500 строк. Дополнительно GAP-11 cross-domain imports в review store нарушает domain isolation, а GAP-12 raw fetch смешан с TanStack Query. Эта фаза устраняет все три проблемы.

## Work

### Part A: CommunicationsPage Refactoring

1. **Extract TanStack Query logic into composables:**
   - `frontend/src/domains/communications/composables/useCommunicationsQueries.ts`
     - `useCommunicationsList` — list query with filters
     - `useCommunicationDetail` — single item query
     - `useCommunicationMutations` — create/update/delete mutations
   - `frontend/src/domains/communications/composables/useMailQuery.ts`
     - `useMailList`, `useMailMessage`, `useMailSyncStatus`
   - `frontend/src/domains/communications/composables/useTelegramQuery.ts`
     - `useTelegramChats`, `useTelegramMessages`
   - `frontend/src/domains/communications/composables/useWhatsAppQuery.ts`
     - `useWhatsAppSessions`, `useWhatsAppMessages`

2. **Eliminate raw fetch:**
   - Найти все `fetch()` вызовы в CommunicationsPage
   - Заменить на TanStack Query hooks или ApiClient напрямую
   - Проверить: `fetchMailMessage`, `fetchMailSyncStatus` и другие

3. **Split CommunicationsPage into panel components:**
   - `frontend/src/domains/communications/components/MailPanel.vue`
   - `frontend/src/domains/communications/components/TelegramPanel.vue`
   - `frontend/src/domains/communications/components/WhatsAppPanel.vue`
   - CommunicationsPage.vue становится роутером/контейнером между панелями
   - Каждый panel component < 300 строк

4. **Verify CommunicationsPage < 500 строк:**
   - `wc -l CommunicationsPage.vue` < 500

### Part B: Review Store Cross-Domain Isolation

5. **Create shared types:**
   - `frontend/src/domains/review/types/shared.ts`
     - `ReviewRelationship` interface
     - `ReviewDecision` interface
     - `ReviewObligation` interface
     - `ReviewContradiction` interface
     - Minimal fields only — не дублировать domain model

6. **Refactor review store:**
   - `frontend/src/domains/review/stores/review.ts`
     - Убрать прямые импорты из `../../personas/api/personas`
     - Убрать прямые импорты из `../../tasks/api/tasks`
     - Убрать прямые импорты из `../../knowledge/api/knowledge`
     - Использовать shared types из `../types/shared`
     - API вызовы через инверсию зависимостей strategy pattern

7. **Verify:**
   - `grep "from '../../personas\|from '../../tasks\|from '../../knowledge" frontend/src/domains/review/stores/review.ts` — 0 matches
   - `cd frontend && pnpm build` — build pass
   - `cd frontend && pnpm lint` — lint pass

## Acceptance criteria (all must pass)

- [ ] AC1: CommunicationsPage.vue < 500 строк было 891
- [ ] AC2: Query-логика вынесена в composables useCommunicationsQueries, useMailQuery, useTelegramQuery, useWhatsAppQuery
- [ ] AC3: Нет raw fetch в CommunicationsPage — все запросы через TanStack Query или ApiClient
- [ ] AC4: CommunicationsPage разбит на MailPanel, TelegramPanel, WhatsAppPanel — каждый < 300 строк
- [ ] AC5: Review store не импортирует напрямую из personas, tasks, knowledge
- [ ] AC6: Review store использует shared types из review/types/shared.ts
- [ ] AC7: review/types/shared.ts содержит minimal interfaces для ReviewRelationship, ReviewDecision, ReviewObligation, ReviewContradiction
- [ ] AC8: cd frontend && pnpm build passes exit 0
- [ ] AC9: cd frontend && pnpm lint passes exit 0
- [ ] AC10: CommunicationsPage корректно рендерит активную панель

## Mandatory commands (run each, surface last ~10 lines + exit code)

- `cd frontend && pnpm build`
- `cd frontend && pnpm lint`

## Evidence required in transcript

- Build output — last 10 lines showing success
- CommunicationsPage line count: `wc -l frontend/src/domains/communications/views/CommunicationsPage.vue`
- Review store imports verification: `grep -n "from '\.\./\." frontend/src/domains/review/stores/review.ts`
- List of new composables in `frontend/src/domains/communications/composables/`
- List of new panel components

## Notes

- Не менять API контракты backend — только frontend рефакторинг
- CommunicationsPage становится layout-контейнером с active tab state
- Review store fixes — использовать shared types, не дублировать domain logic
- All new composables must have proper TypeScript types — no `any`
- Reference существующие store patterns в других domains
