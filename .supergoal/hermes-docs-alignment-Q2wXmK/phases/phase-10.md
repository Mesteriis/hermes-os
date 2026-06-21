SUPERGOAL_PHASE_START
Phase: 10 of 10 — Polish & Harden
Task: Final audit, edge cases, security review, performance, documentation update, full validation gate
Mandatory commands: make validate, cd frontend && pnpm build && pnpm test:unit
Acceptance criteria: 8
Evidence required: final build/test output, updated IMPLEMENTATION_STATUS.md, security review notes
Depends on phases: all phases P1-P9

## Why

Финальная фаза — catch what earlier phases missed because they were focused on shipping alignment. Full validation gate ensures no regressions and all documentation is up to date.

## Sub-passes (each must produce evidence)

### 1. Tests
- [ ] **Backend tests:** Добавить Rust integration tests для изменённых API endpoints из Phases 2, 3, 4
- [ ] **Frontend tests:** Добавить Vitest unit tests для stores Phase 6, composables Phase 5, renamed components Phase 7
- [ ] **Coverage check:** `grep -r "#\[test\]" backend/src/domains/communications/ | wc -l` — хотя бы несколько тестов
- [ ] **Frontend coverage:** `grep -r "describe\(" frontend/src/domains/*/__tests__/ | wc -l` — хотя бы несколько

### 2. Security Review
- [ ] **Input validation:** Проверить все новые API endpoints на валидацию входных данных
- [ ] **X-Hermes-Secret:** Убедиться, что все frontend API calls используют централизованный ApiClient с X-Hermes-Secret
- [ ] **No secrets in bundle:** Проверить frontend bundle на наличие API keys, tokens
- [ ] **No credential exposure:** Проверить audit logs на отсутствие credential payloads

### 3. Performance
- [ ] **Virtual scrolling:** Все списки >20 items используют @tanstack/vue-virtual
- [ ] **Lazy loading:** Все route components lazy-loaded через Vue Router
- [ ] **Bundle size:** `cd frontend && du -sh dist/` — зафиксировать baseline
- [ ] **No N+1:** Проверить GraphQL/SQL queries на N+1 проблемы backend

### 4. Edge Cases
- [ ] **Empty states:** Все возможные пустые состояния работают корректно
- [ ] **Network errors:** Все компоненты корректно обрабатывают offline/error состояния
- [ ] **Concurrent updates:** Проверить race conditions при параллельных записях
- [ ] **Boundary conditions:** Проверить на null/undefined/empty string inputs

### 5. Diff Review
- [ ] **No stray debug logs:** `grep -r "console.log\|println!\|dbg!" --include="*.rs" --include="*.ts" --include="*.vue"` — review each
- [ ] **No TODOs from this run:** Проверить, что все TODO/FIXME из Phases 2-9 закрыты
- [ ] **No unused imports:** `cargo clippy` и `vue-tsc` должны проходить без warnings

### 6. Documentation
- [ ] **IMPLEMENTATION_STATUS.md:** Обновить статус по всем выполненным фазам
- [ ] **docs/refactoring/implementation-alignment-plan.md:** Обновить с результатами alignment
- [ ] **ADR review:** Если были созданы новые ADR — убедиться в их корректности

### 7. Full Validation Gate
- [ ] `make validate` — from repo root, exit 0
- [ ] `cd frontend && pnpm build` — exit 0
- [ ] `cd frontend && pnpm lint` — exit 0
- [ ] `cd frontend && pnpm test:unit` — exit 0
- [ ] `cargo fmt --check` — pass
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` — pass
- [ ] `cargo test --all` — exit 0

### 8. Regression Sweep
- [ ] Сравнить список API endpoints до и после alignment
- [ ] Проверить основные user flows: login, view persons, view communications, search
- [ ] Проверить что compatibility слои работают persons redirect, contact source_kind

## Acceptance criteria (all must pass)

- [ ] AC1: Все sub-passes выполнены с документальным evidence
- [ ] AC2: `make validate` from repo root — exit 0
- [ ] AC3: `cd frontend && pnpm build && pnpm test:unit` — exit 0
- [ ] AC4: `cargo clippy --all-targets --all-features -- -D warnings` — pass
- [ ] AC5: IMPLEMENTATION_STATUS.md обновлён с актуальным статусом
- [ ] AC6: Нет незакрытых TODO/FIXME из Phases 2-9
- [ ] AC7: Нет console.log/dbg! в новом production коде
- [ ] AC8: Compatibility слои persons redirect, contact source_kind работают

## Mandatory commands (run each, surface last ~10 lines + exit code)

- `make validate`
- `cd frontend && pnpm build && pnpm test:unit`

## Evidence required in transcript

- Final build output — last 10 lines
- Test output — last 10 lines showing all tests pass
- `make validate` output — last 20 lines showing all gates pass
- Updated IMPLEMENTATION_STATUS.md — summary section
- Bundle size analysis
- Security review notes — one paragraph per check

## Notes

- Эта фаза — самая важная для качества; не пропускать sub-passes
- Если sub-pass не применим например нет security issues — задокументировать почему
- `make validate` — полный validation gate, не пропускать
- После завершения этой фазы можно создать git commit если пользователь запросит
- Обновить STATE.md до COMPLETE статуса после завершения
