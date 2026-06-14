SUPERGOAL_PHASE_START
Phase: 2 of 10 — Backend: Persona Naming
Task: Rename Persons to Personas in backend — API routes, models, structs, SemanticSourceKind. Keep compatibility aliases.
Mandatory commands: cargo build, cargo test --all, make backend-validate
Acceptance criteria: 8
Evidence required: build output, test output, curl responses
Depends on phases: Phase 1

## Why

Документация определяет Persona как каноническую модель ADR-0084, но код использует Person struct и /api/v1/persons API. Необходимо синхронизировать backend без breaking changes для существующих клиентов.

## Work

1. **Добавить /api/v1/personas routes:**
   - Создать новые route handlers в `backend/src/domains/persons/handlers/personas.rs`
   - Добавить routes в `backend/src/app/router.rs` как алиасы к persons handlers
   - Оставить `/api/v1/persons` как compatibility redirect

2. **Person struct compatibility:**
   - Добавить type alias `pub type Persona = Person;` в `backend/src/domains/persons/api/types.rs`
   - Убедиться, что `Persona` и `Person` — один и тот же тип на уровне компиляции

3. **PersonaType enum:**
   - Синхронизировать с ADR-0084: `human`, `ai_agent`, `organization_proxy`, `system`
   - Добавить serialization/deserialization тесты

4. **SemanticSourceKind rename:**
   - `SemanticSourceKind::Person` → serialize по умолчанию в `"person"`
   - Добавить compatibility deserialization для `"contact"` → `SemanticSourceKind::Person`
   - Обновить все внутренние reference

5. **Handler facade updates:**
   - Persons handlers facade (`backend/src/domains/persons/handlers/mod.rs`) — добавить re-export для новых persona handlers
   - API projection facade (`backend/src/domains/persons/api.rs`) — обновить комментарии

6. **Update documentation:**
   - ADR review: если rename требует ADR — создать новый ADR или обновить существующий ADR-0084
   - Обновить `docs/persons/` ссылки с persons на personas где уместно

7. **Verify:**
   - `curl /api/v1/personas` возвращает данные
   - `curl /api/v1/persons` продолжает работать
   - `curl -X POST /api/v1/personas` creates persona
   - Search с `source_kind=person` работает
   - Search с `source_kind=contact` тоже работает compatibility

## Acceptance criteria (all must pass)

- [ ] AC1: `curl /api/v1/personas` возвращает те же данные, что `/api/v1/persons`
- [ ] AC2: `curl /api/v1/persons` продолжает работать без изменений
- [ ] AC3: `SemanticSourceKind::Person` сериализуется в `"person"` по умолчанию
- [ ] AC4: `SemanticSourceKind::Person` десериализует `"contact"` как compatibility без ошибки
- [ ] AC5: `cargo build` passes (exit 0)
- [ ] AC6: `cargo test --all` passes (exit 0)
- [ ] AC7: `make backend-validate` passes (exit 0)
- [ ] AC8: Нет breaking изменений в существующих API контрактах

## Mandatory commands (run each, surface last ~10 lines + exit code)

- `cargo build`
- `cargo test --all`
- `make backend-validate`

## Evidence required in transcript

- Build output — last 10 lines showing success
- Test output — last 10 lines showing all tests pass
- curl response from `/api/v1/personas` — first persona object
- curl response from `/api/v1/persons` — confirming compatibility

## Notes

- Do NOT rename database tables — only API/frontend level
- `SemanticSourceKind::Person` → `"contact"` is a legacy DB value; keep deserialization compatibility
- Add deprecation notice for `"contact"` in API docs but do not remove support
- ADR-0084 already defines Persona model — no new ADR needed unless schema change
- Check `docs/adr/ADR-0090` if exists for compatible storage decisions
