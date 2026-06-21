SUPERGOAL_PHASE_START
Phase: 1 of 10 — Foundation & Safety Net
Task: Add test infrastructure Testcontainers in crates/testkit and write characterization tests for existing behavior
Mandatory commands: make backend-validate, cd frontend && pnpm build && pnpm test:unit
Acceptance criteria: 8
Evidence required: testcontainers test output, frontend test output, inventory report
Depends on phases: none

## Why

Без тестовой инфраструктуры и characterization тестов все последующие изменения рискуют сломать существующее поведение. Эта фаза создаёт safety net для всех последующих alignment-изменений.

## Work

1. **Testcontainers infrastructure в `crates/testkit/`:**
   - Проверить существующую структуру `crates/testkit/`
   - Добавить Testcontainers PostgreSQL container helper
   - Добавить container lifecycle management (start, health check, stop)
   - Добавить fixture loading helper
   - Добавить migration runner helper

2. **Characterization tests для Person API:**
   - Тест: Create Person / person переписка
   - Тест: List Persons / pagination
   - Тест: Get Person by ID
   - Тест: Update Person metadata
   - Тест: Person search

3. **Characterization tests для Communication API:**
   - Тест: List communications with filters
   - Тест: Get communication by ID
   - Тест: Communication search

4. **Characterization tests для Search API:**
   - Тест: Basic text search
   - Тест: Filter by source_kind

5. **Frontend test infrastructure:**
   - Расширить `frontend/src/__tests__/placeholder.test.ts` до реального smoke test
   - Добавить Vitest config если отсутствует
   - Добавить тест для ApiClient
   - Добавить тест для i18n composable

6. **Инвентаризация:**
   - Задокументировать все naming conflicts Persons↔Personas
   - Задокументировать все компоненты без Loading/Empty/Error/Skeleton states
   - Задокументировать все cross-domain import зависимости
   - Измерить CommunicationsPage — точная разбивка строк по ответственности
   - Определить Telegram parity gaps относительно Telegram Desktop
   - Определить Mail parity gaps относительно Outlook/Apple Mail/Thunderbird

7. **Verify:**
   - Все Testcontainers тесты проходят
   - Frontend тесты проходят
   - `make backend-validate` passes
   - Инвентаризационный отчёт сохранён

## Acceptance criteria (all must pass)

- [ ] AC1: Testcontainers PostgreSQL контейнер поднимается и проходит health check
- [ ] AC2: Хотя бы 1 characterization test для Person API
- [ ] AC3: Хотя бы 1 characterization test для Communication API
- [ ] AC4: Frontend placeholder test расширен до реального smoke test (хотя бы 2 asserts)
- [ ] AC5: `make backend-validate` passes (exit 0)
- [ ] AC6: `cd frontend && pnpm test:unit` passes (exit 0)
- [ ] AC7: Инвентаризация naming conflicts задокументирована в `docs/refactoring/`
- [ ] AC8: Инвентаризация UI states по всем компонентам выполнена и задокументирована

## Mandatory commands (run each, surface last ~10 lines + exit code)

- `make backend-validate`
- `cd frontend && pnpm build && pnpm test:unit`

## Evidence required in transcript

- Testcontainers test output — last 10 lines showing success
- Frontend test output — last 10 lines showing success
- Инвентаризационный отчёт — файл или summary

## Notes

- Do NOT change existing behavior — only add tests
- Testcontainers тесты должны быть изолированы от dev PostgreSQL
- Использовать `crates/testkit/` для shared test infrastructure
- Frontend тесты использовать Vitest (уже есть в проекте)
- Инвентаризацию сохранить в `docs/refactoring/` для reference
