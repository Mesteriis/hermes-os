### Summary / Резюме

Страница `operations/documentation-map.md` обновляется до актуального состояния после завершения канонического рефакторинга.  
Карта документации фиксирует ключевые артефакты, подтверждающие полное выполнение всех архитектурных требований, включая ADR, реализацию, миграции, guard-скрипты и тесты, и отражает завершённый финальный validation gate.

### Proposed pages / Предлагаемые страницы

`operations/documentation-map.md`

```markdown
# Карта документации

## Канонический рефакторинг — финальный отчёт

Финальный канонический отчёт: `canonical-evidence-final-report.md` (корень проекта).

Текущая оценка готовности: **100% выполнено, 0% осталось**.  
Все ключевые артефакты документации и кода подтверждены audit-матрицей и финальным validation-проходом (`make validate` – зелёный).

### Completion Audit Matrix

| Требование | Доказательство | Статус |
|---|---|---|
| `External Systems -> Integrations -> Vault -> Observation Platform -> Ingestion -> Domains -> Knowledge -> Review -> Actions` зафиксировано как целевая архитектура | `docs/adr/ADR-0096-canonical-evidence-review-and-context-packs.md`, `docs/architecture/architecture-overview.md`, `docs/foundation/domain-map.md` | Доказано |
| Observation Platform = canonical evidence store, не часть Vault | `backend/src/platform/observations/**`, миграция `backend/migrations/0094_create_canonical_evidence_review_context.sql`, guard `scripts/check-architecture.mjs` запрещает `backend/src/vault/observations` | Доказано |
| Observation = evidence, not truth | `ADR-0096`, `docs/foundation/world-model.md`, owner flows идут через observation capture + review/domain promotion | Доказано |
| Observations append-only; provider deletion создает новую observation | live test `backend/tests/observations.rs::observations_are_append_only_and_survive_provider_deletion_against_postgres` | Доказано |
| Manual note / voice memo / browser capture создают observations без Vault | live tests в `backend/tests/observations.rs` для manual note, voice recording, browser capture | Доказано |
| Review существует как отдельный inbox domain | `backend/src/domains/review/**`, review routes в `backend/src/app/router/routes/review.rs` | Доказано |
| Review lifecycle `new/in_review/approved/promoted/dismissed/archived` | schema в миграции `0094...`, live tests `backend/tests/review_inbox.rs` | Доказано |
| Review promotion покрывает Personas, Organizations, Tasks, Decisions, Obligations, Relationships, Projects, Knowledge/Documents | `backend/tests/review_inbox.rs::review_can_materialize_promotions_for_core_target_domains_against_postgres` | Доказано |
| Tasks обязаны иметь provenance | guards в `backend/migrations/0104_add_task_provenance_reference_guard.sql`, checks/tests в `backend/tests/tasks.rs` | Доказано |
| Context Packs живут в `engines/context_packs/` как derived/rebuildable engine output | `backend/src/engines/context_packs/**`, runtime guard на legacy table access, active docs updated | Доказано |
| `engines/identity_resolution/` и `engines/relationships/` выделены отдельно | `backend/src/engines/identity_resolution/**`, `backend/src/engines/relationships/**`, `backend/src/engines/mod.rs` | Доказано |
| Запрещены `domains/signals`, `domains/events`, `domains/attention`, `domains/evidence`, Vault-owned observations | `scripts/check-architecture.mjs` + green run `node scripts/check-architecture.mjs` | Доказано |
| Ключевые workflow actions reuse observation evidence и materialize projection links | live suite `backend/tests/v1_workflow_actions.rs` | Доказано |
| Полный validation gate зеленый | `make validate` passed; внутри: backend tests green, `vitest` 156 files / 492 tests passed, `vue-tsc --noEmit && vite build` passed | Доказано |

### Финальная валидация

- `make backend-validate` – passed
- `make validate` – passed
- Frontend:
  - `vitest run` – 156 test files passed, 492 tests passed
  - `vue-tsc --noEmit && vite build` – passed

Документация по ключевым компонентам приведена в колонке «Доказательство» audit-матрицы выше.  
Для операционной навигации по документации ориентируйтесь на перечисленные paths как на канонические источники истины в текущем периоде.
```

### Source coverage / Покрытие источников

- `canonical-evidence-final-report.md` (включено 12000 символов, файл обрезан):
  - Факт завершения рефакторинга на 100% (оценка `100%` / `0%`).
  - Completion Audit Verdict: таблица из 14 строк, каждая строка приведена в карте документации.
  - Финальный validation gate: результаты `make validate`, `vitest`, `vue-tsc`.
  - Перечень ключевых артефактов (пути к ADR, architecture, foundation, migrations, tests, guard scripts, engines, platform).
  - Исторические ledger slices не включались, так как карта документации опирается только на итоговый verdict и validation.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста (единственный встроенный файл `canonical-evidence-final-report.md`, обрезанный после 12000 символов) расхождений между кодом, документацией и ADR не видно.
