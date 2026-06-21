SUPERGOAL_PHASE_START
Phase: 3 of 10 — Backend: Communication Domain
Task: Restructure domains/mail/ God directory into Communication domain with core facade and mail channel isolation
Mandatory commands: cargo build, cargo test --all, make backend-validate
Acceptance criteria: 8
Evidence required: tree listing of communications/, build output, test output
Depends on phases: Phase 2

## Why

domains/mail/ содержит ~100+ файлов и является God directory. Документация требует Communication как primary ingestion spine ADR-0085. Mail должен стать channel-specific поддиректорией внутри Communication domain.

## Work

1. **Создать Communication domain core:**
   - `backend/src/domains/communications/mod.rs` — domain facade
   - `backend/src/domains/communications/core/` — channel-agnostic модули:
     - `projection.rs` — shared communication projection logic
     - `state_machine.rs` — communication state machine
     - `models.rs` — shared Communication models
     - `errors.rs` — communication domain errors

2. **Mail channel isolation:**
   - `backend/src/domains/communications/mail/` — переместить mail-specific модули:
     - `accounts/`, `messages/`, `sync/`, `rules/`, `signatures/`, `handlers/`, `storage/`, `fixtures/`
   - `backend/src/domains/communications/mail/mod.rs` — mail facade
   - `backend/src/domains/communications/mail/core/` — mail core (core.rs submodules)
   - `backend/src/domains/communications/mail/background_sync/` — background sync
   - `backend/src/domains/communications/mail/handlers/` — route handlers

3. **Telegram channel integration (если уместно):**
   - `backend/src/domains/communications/telegram/mod.rs` — Telegram channel facade
   - Ре-экспорт из `backend/src/integrations/telegram/`

4. **WhatsApp channel integration (если уместно):**
   - `backend/src/domains/communications/whatsapp/mod.rs` — WhatsApp channel facade
   - Ре-экспорт из `backend/src/integrations/whatsapp/`

5. **Compatibility слой:**
   - `backend/src/domains/mail/mod.rs` — оставить как re-export facade
   - Все public типы из `domains::mail::*` продолжают экспортироваться
   - Добавить deprecation notice в документацию

6. **Обновить импорты:**
   - Все `use crate::domains::mail::*` в handlers, workflows, engines
   - `backend/src/app/router.rs` — обновить route registration
   - `backend/src/workflows/email_sync_pipeline.rs` — обновить импорты
   - `backend/src/workflows/email_intelligence.rs` — обновить импорты

7. **Verify:**
   - `cargo build` — все импорты резолвятся
   - `cargo test --all` — все тесты проходят
   - `make backend-validate` — полный validation gate

## Acceptance criteria (all must pass)

- [ ] AC1: `backend/src/domains/communications/` существует с core/ и mail/ channel
- [ ] AC2: `backend/src/domains/mail/` продолжает экспортировать те же public API через re-export
- [ ] AC3: Все существующие импорты из `domains::mail::*` продолжают работать
- [ ] AC4: Communication domain core не содержит mail-specific provider credential logic
- [ ] AC5: `cargo build` passes (exit 0)
- [ ] AC6: `cargo test --all` passes (exit 0)
- [ ] AC7: `make backend-validate` passes (exit 0)
- [ ] AC8: Нет дублирования кода между `communications/mail/` и старым `domains/mail/`

## Mandatory commands (run each, surface last ~10 lines + exit code)

- `cargo build`
- `cargo test --all`
- `make backend-validate`

## Evidence required in transcript

- Tree listing of `backend/src/domains/communications/` — showing directory structure
- Build output — last 10 lines showing success
- Test output — last 10 lines showing all tests pass

## Notes

- Communication domain — facade над mail, Telegram, WhatsApp channels
- Не переименовывать таблицы или routes без ADR
- Не перемещать provider integration код — только domain-level реструктуризация
- `domains/mail/` compatibility слой — только re-export, без новой логики
- ADR-0073 backend module organization — reference для структуры модулей
