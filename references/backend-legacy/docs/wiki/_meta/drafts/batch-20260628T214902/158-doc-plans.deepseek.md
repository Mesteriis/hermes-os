### Summary / Резюме

Добавить в `operations/documentation-map.md` документирование текущего плана SRP-рефакторинга backend, основанного на встроенном исходном файле `plans/backend-srp-refactoring-plan.md`. Текущее содержимое страницы `operations/documentation-map.md` не встроено в этот context pack, поэтому предлагаемое содержимое может потребовать слияния с существующими записями карты документации. Предлагаемая страница на русском языке содержит описание плана, его статус, эталонную архитектуру, категории файлов-нарушителей и приоритеты рефакторинга, полностью опираясь на факты из исходного файла.

### Proposed pages / Предлагаемые страницы

```markdown
# Карта документации: планы и операции

## План SRP-рефакторинга backend

- **Файл плана:** `plans/backend-srp-refactoring-plan.md`
- **Дата:** 2026-06-14
- **Статус:** не подтверждён в данном контексте (ожидает выполнения)

### Текущее состояние

Бэкенд содержит ~250+ файлов реализации. Большинство модулей уже следуют Single Responsibility Principle (SRP) и разделены на поддиректории: `handlers/`, `core/`, `models/`, `store/`, `errors/`, `validation/`.

### Эталонная архитектура

Каждый домен/модуль должен следовать структуре:

```
domain/
  mod.rs          # re-exports
  api.rs          # Axum handler functions (thin)
  handlers/       # route registration (if separated from api.rs)
    mod.rs
    ...
  models.rs       # DTO, request/response types
  store.rs        # data access layer
  errors.rs       # domain-specific errors
  validation.rs   # input validation
  ids.rs          # ID generation
  constants.rs    # magic constants
  row_mapping.rs  # DB row <-> domain model mapping
  graph_projection.rs  # graph projection logic
```

### Файлы, нарушающие SRP (категории)

#### Категория A: Monolithic API files

| Файл | Проблема |
|------|----------|
| `domains/decisions/api.rs` | Axum handlers + DTO + validation helpers в одном файле |
| `domains/obligations/api.rs` | То же самое |
| `domains/relationships/api.rs` | То же самое |

Эти три файла — копии друг друга с разными типами (DRY нарушен).

#### Категория B: Monolithic Engine files

| Файл | Проблема |
|------|----------|
| `engines/risk.rs` | RiskEngine + RiskAttentionStatus + модели вместе |
| `engines/search.rs` | SearchIndex + SearchDocument + SearchResult + SearchError вместе |
| `engines/trust.rs` | TrustEngine + TrustSignalKind + модели сигналов |
| `engines/enrichment.rs` | EnrichmentEngine + модели + validation |
| `engines/obligation.rs` | ObligationEngine + детекторы + модели + helpers |
| `engines/decision.rs` | DecisionEngine + детекторы + модели + helpers |

#### Категория C: Platform mega-files

| Файл | Проблема |
|------|----------|
| `platform/calls_api.rs` | Router + handlers + импорты из всех доменов |
| `platform/events_api.rs` | Router + handlers + импорты из всех доменов |

Оба файла импортируют практически один и тот же набор доменов — дублирование.

#### Категория D: Monolithic platform stores

| Файл | Проблема |
|------|----------|
| `platform/storage.rs` | Database struct + StorageError + миграции + настройки |

### Приоритеты рефакторинга

1. **Приоритет 1:** `domains/decisions`, `obligations`, `relationships` — API разделить на `handlers/` + `models/` + общие helpers.
2. **Приоритет 2:** `engines/obligation.rs`, `decision.rs` (крупные, 300+ строк) — разделить на `engine.rs` + `detection.rs` + `models.rs` + `errors.rs`.
3. **Приоритет 3:** `engines/risk.rs`, `search.rs`, `trust.rs`, `enrichment.rs` (100–200 строк) — чистое разделение на `mod.rs` + `models.rs` + `errors.rs`.
4. **Приоритет 4:** `platform/calls_api.rs`, `events_api.rs` — требуют осторожности с импортами, затронут routing.
5. **Приоритет 5:** `platform/storage.rs` — низкий риск, чистое разделение.

### Риски и зависимости

- Тесты: каждый рефакторинг требует запуска `make validate` или `make backend-validate`.
- Импорты: после разделения файлов нужно обновить все `use` импорты в зависимых модулях.
- `app/router/routes/`: некоторые route modules импортируют из `calls_api` и `events_api`.
- `app/error/conversions/`: содержат `From` impl для errors.
- Миграции: не затрагиваются.
```

### Source coverage / Покрытие источников

- **`plans/backend-srp-refactoring-plan.md`**:
  - Текущее состояние (250+ файлов, существующее разделение)
  - Эталонная архитектура и структура доменов
  - Категории A (monolithic API files) — `domains/decisions/api.rs`, `domains/obligations/api.rs`, `domains/relationships/api.rs`
  - Категория B (monolithic engines) — `engines/risk.rs`, `engines/search.rs`, `engines/trust.rs`, `engines/enrichment.rs`, `engines/obligation.rs`, `engines/decision.rs` и их содержимое
  - Категория C (platform mega-files) — `platform/calls_api.rs`, `platform/events_api.rs`
  - Категория D — `platform/storage.rs`
  - Приоритеты рефакторинга (1–5) с целевыми структурами
  - Риски: тесты (`make validate`), импорты, `app/router/routes/`, `app/error/conversions/`, миграции не затрагиваются

### Drift candidates / Кандидаты на drift

Из предоставленного контекста не видно расхождений между кодом, документацией и ADR, так как встроен только один исходный файл плана, а иных источников (кода, других документов) не предоставлено.
