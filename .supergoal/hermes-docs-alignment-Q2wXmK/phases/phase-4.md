SUPERGOAL_PHASE_START
Phase: 4 of 10 — Backend: Notes Domain
Task: Create backend domain for Notes with models, store, API handlers, and route registration
Mandatory commands: cargo build, cargo test --all, make backend-validate
Acceptance criteria: 8
Evidence required: curl requests, build output, tree listing of notes/
Depends on phases: Phase 2

## Why

docs/domains/notes.md и frontend компоненты NotesList.vue существуют, но backend handler для /api/v1/notes отсутствует. Это один из выявленных gaps GAP-4.

## Work

1. **Создать Notes domain структуру:**
   - `backend/src/domains/notes/mod.rs` — domain facade
   - `backend/src/domains/notes/models.rs` — Note model:
     - `NoteId`, `Note`, `CreateNote`, `UpdateNote`
     - Поля: id, title, content, source, tags, created_at, updated_at
     - Lightweight — без полноценного domain lifecycle
   - `backend/src/domains/notes/store.rs` — Note store:
     - CRUD operations
     - Event-backed или direct storage
     - List with pagination
   - `backend/src/domains/notes/api.rs` — API types
   - `backend/src/domains/notes/handlers/` — route handlers:
     - `list.rs` — GET /api/v1/notes
     - `create.rs` — POST /api/v1/notes
     - `read.rs` — GET /api/v1/notes/:id
     - `update.rs` — PUT /api/v1/notes/:id
     - `delete.rs` — DELETE /api/v1/notes/:id

2. **Route registration:**
   - Добавить notes routes в `backend/src/app/router.rs`
   - Route prefix: `/api/v1/notes`

3. **Integration с frontend API types:**
   - Убедиться, что frontend `frontend/src/domains/notes/api/notes.ts` совместим с новым API
   - Обновить frontend API типы если требуется

4. **Notes как lightweight domain:**
   - Notes — document-like artifacts согласно master-spec.md
   - Не добавлять полный domain lifecycle (events, projections, state machines)
   - Notes могут быть source records, memory items или document extracts

5. **Verify:**
   - `curl /api/v1/notes` — возвращает `{ items: [] }`
   - `curl -X POST /api/v1/notes -d '{"title":"test","content":"hello"}'` — создаёт заметку
   - `curl /api/v1/notes/:id` — возвращает заметку
   - `curl -X PUT /api/v1/notes/:id` — обновляет заметку
   - `curl -X DELETE /api/v1/notes/:id` — удаляет заметку

## Acceptance criteria (all must pass)

- [ ] AC1: `curl /api/v1/notes` возвращает `{ items: [] }` или массив заметок
- [ ] AC2: `curl -X POST /api/v1/notes` создаёт заметку и возвращает её с id
- [ ] AC3: `curl /api/v1/notes/:id` возвращает заметку по ID
- [ ] AC4: `curl -X PUT /api/v1/notes/:id` обновляет заметку
- [ ] AC5: `curl -X DELETE /api/v1/notes/:id` удаляет заметку
- [ ] AC6: Notes — lightweight, без полноценного domain lifecycle (events, projections)
- [ ] AC7: `cargo build` passes (exit 0)
- [ ] AC8: `make backend-validate` passes (exit 0)

## Mandatory commands (run each, surface last ~10 lines + exit code)

- `cargo build`
- `cargo test --all` (если добавлены тесты)
- `make backend-validate`

## Evidence required in transcript

- curl запросы к `/api/v1/notes` endpoints с responses
- Build output — last 10 lines showing success
- Tree listing of `backend/src/domains/notes/` — showing directory structure

## Notes

- Notes — не полноценный domain с event sourcing (пока). Использовать direct storage.
- Следовать `docs/domains/notes.md` как reference для модели данных
- Frontend компоненты NotesList.vue, NotesPage.vue уже существуют — достаточно API контракта
- Если Notes должны быть event-backed — создать ADR, иначе direct store достаточно
- Не требуется schema migration если используется event store или JSONB в существующей таблице
