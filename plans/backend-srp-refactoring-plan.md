# План рефакторинга: Разделение ответственности в backend

Дата: 2026-06-14

## Текущее состояние

Бэкенд содержит ~250+ файлов реализации. Большинство модулей **уже следуют** Single Responsibility Principle (SRP) и разделены на поддиректории: `handlers/`, `core/`, `models/`, `store/`, `errors/`, `validation/`.

## Эталонная архитектура (как должно быть)

Каждый домен/модуль следует структуре:

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

## Файлы, нарушающие SRP (требуют рефакторинга)

### Категория A: Monolithic API files (handlers + DTO + helpers)

| Файл | Строк | Проблема | Целевая структура |
|------|-------|----------|-------------------|
| `domains/decisions/api.rs` | 138 | Axum handlers + DTO (Query, Request, Response) + validation helpers в одном файле | Разделить на `api/handlers.rs` + `api/models.rs` + выделить общие helpers |
| `domains/obligations/api.rs` | 138 | То же самое | Аналогично decisions |
| `domains/relationships/api.rs` | 138 | То же самое | Аналогично decisions |

Эти три файла — **копия друг друга** с разными типами. DRY нарушен.

### Категория B: Monolithic Engine files (business logic + models + errors)

| Файл | Строк | Проблема | Целевая структура |
|------|-------|----------|-------------------|
| `engines/risk.rs` | 153 | RiskEngine + RiskAttentionStatus + модели в одном файле | Разделить на `engines/risk/mod.rs` + `models.rs` + `errors.rs` + `validation.rs` |
| `engines/search.rs` | 217 | SearchIndex + SearchDocument + SearchResult + SearchError в одном файле | Разделить на `engines/search/mod.rs` + `index.rs` + `models.rs` + `errors.rs` |
| `engines/trust.rs` | 120 | TrustEngine + TrustSignalKind + модели сигналов | Разделить на `engines/trust/mod.rs` + `models.rs` + `errors.rs` |
| `engines/enrichment.rs` | 115 | EnrichmentEngine + модели + validation | Разделить на `engines/enrichment/mod.rs` + `models.rs` + `errors.rs` + `validation.rs` |
| `engines/obligation.rs` | 345 | ObligationEngine + детекторы + модели + helpers | Разделить на `engines/obligation/mod.rs` + `engine.rs` + `detection.rs` + `models.rs` + `errors.rs` |
| `engines/decision.rs` | 304 | DecisionEngine + детекторы + модели + helpers | Разделить на `engines/decision/mod.rs` + `engine.rs` + `detection.rs` + `models.rs` + `errors.rs` |

### Категория C: Platform mega-files (routing + handlers + imports из всех доменов)

| Файл | Строк | Проблема | Целевая структура |
|------|-------|----------|-------------------|
| `platform/calls_api.rs` | 208 | Router + handlers + импорты из ВСЕХ доменов | Разделить на `platform/calls/api/mod.rs` + `handlers.rs` + `models.rs` с вынесением импортов |
| `platform/events_api.rs` | 204 | Router + handlers + импорты из ВСЕХ доменов | Разделить на `platform/events/api/mod.rs` + `handlers.rs` + `models.rs` |

Оба файла импортируют практически один и тот же набор доменов — дублирование.

### Категория D: Monolithic platform stores

| Файл | Строк | Проблема | Целевая структура |
|------|-------|----------|-------------------|
| `platform/storage.rs` | 218 | Database struct + StorageError + миграции + настройки | Разделить на `platform/storage/mod.rs` + `database.rs` + `errors.rs` + `migrations.rs` |

## Приоритеты рефакторинга

```mermaid
flowchart TD
    A["Приоритет 1<br/>Decisions, Obligations, Relationships API"] --> B["Простые, изолированные изменения<br/>Copy-paste паттерн<br/>Высокий DRY выигрыш"]
    C["Приоритет 2<br/>Engines: obligation.rs, decision.rs"] --> D["Крупные файлы 300+ строк<br/>Логика разделяется на engine + detection"]
    E["Приоритет 3<br/>Engines: risk, search, trust, enrichment"] --> F["Средние файлы 100-200 строк<br/>Чистое разделение на mod + models + errors"]
    G["Приоритет 4<br/>Platform: calls_api, events_api"] --> H["Затрагивают routing<br/>Требуют осторожности с импортами"]
    I["Приоритет 5<br/>platform/storage.rs"] --> J["Низкий риск<br/>Чистое разделение"]
```

## Подробный план по приоритетам

### Приоритет 1: Decisions / Obligations / Relationships API

**Цель:** Разделить монолитные `api.rs` на `handlers/` + `models/` + выделить общие helpers в shared модуль.

Для каждого из трёх доменов:
1. Создать `api/mod.rs` — re-exports
2. Создать `api/handlers.rs` — только Axum handler функции
3. Создать `api/models.rs` — DTO (запросы/ответы)
4. Выделить общие helpers (`validate_limit`, `validate_required_query_value`, `parse_review_state`, `api_audit_log`, `*_store`) в общий утилитарный модуль (например, `domains/api_helpers.rs`)
5. Обновить `mod.rs` — `pub mod api;`

**Файлы для изменений:**
- `domains/decisions/api.rs` → `domains/decisions/api/mod.rs` + `handlers.rs` + `models.rs`
- `domains/obligations/api.rs` → `domains/obligations/api/mod.rs` + `handlers.rs` + `models.rs`
- `domains/relationships/api.rs` → `domains/relationships/api/mod.rs` + `handlers.rs` + `models.rs`
- `domains/api_helpers.rs` — новый файл с общими helpers
- `app/router/routes/review.rs` — проверить импорты

### Приоритет 2: Engine monolithic files (obligation.rs, decision.rs)

**Цель:** Разделить на модульную структуру, как уже сделано в `engines/timeline/` и `engines/memory/`.

**`engines/obligation.rs` (345 строк):**
- `engines/obligation/mod.rs` — re-exports + ObligationEngine facade
- `engines/obligation/engine.rs` — ObligationEngine impl
- `engines/obligation/detection.rs` — detect_commitment, sentences и др.
- `engines/obligation/models.rs` — ObligationExtractionInput, ObligationTaskCandidate и др.
- `engines/obligation/errors.rs` — ObligationEngineError
- `engines/obligation/validation.rs` — validation functions

**`engines/decision.rs` (304 строк):**
- `engines/decision/mod.rs` — re-exports + DecisionEngine facade
- `engines/decision/engine.rs` — DecisionEngine impl
- `engines/decision/detection.rs` — detect_decision, sentences и др.
- `engines/decision/models.rs` — DecisionExtractionInput и др.
- `engines/decision/errors.rs` — DecisionEngineError
- `engines/decision/validation.rs` — validation functions

### Приоритет 3: Engine files (risk, search, trust, enrichment)

**`engines/risk.rs` (153 строк):**
- `engines/risk/mod.rs` — re-exports + RiskEngine facade
- `engines/risk/models.rs` — RiskAttentionStatus, RiskSignal, RiskSeverity, RiskObservationDraft
- `engines/risk/errors.rs` — RiskEngineError
- `engines/risk/validation.rs` — validate_* functions

**`engines/search.rs` (217 строк):**
- `engines/search/mod.rs` — re-exports
- `engines/search/index.rs` — SearchIndex
- `engines/search/models.rs` — SearchDocument, SearchResult, SearchFields
- `engines/search/errors.rs` — SearchError

**`engines/trust.rs` (120 строк):**
- `engines/trust/mod.rs` — re-exports + TrustEngine facade
- `engines/trust/models.rs` — TrustSignalKind, TrustRelationshipSignal, TrustSourceReliabilitySignal
- `engines/trust/errors.rs` — TrustEngineError
- `engines/trust/validation.rs` — validate_* functions

**`engines/enrichment.rs` (115 строк):**
- `engines/enrichment/mod.rs` — re-exports + EnrichmentEngine facade
- `engines/enrichment/models.rs` — PreferenceDraft, EnrichmentCandidateDraft
- `engines/enrichment/errors.rs` — EnrichmentEngineError
- `engines/enrichment/validation.rs` — validate_* functions

### Приоритет 4: Platform mega-files (calls_api.rs, events_api.rs)

**`platform/calls_api.rs` (208 строк):**
- `platform/calls/api/mod.rs` — re-exports + router registration
- `platform/calls/api/handlers.rs` — handler functions
- `platform/calls/api/models.rs` — request/response types

**`platform/events_api.rs` (204 строк):**
- `platform/events/api/mod.rs` — re-exports + router registration
- `platform/events/api/handlers.rs` — handler functions
- `platform/events/api/models.rs` — request/response types

### Приоритет 5: platform/storage.rs (218 строк)

**`platform/storage.rs`:**
- `platform/storage/mod.rs` — re-exports
- `platform/storage/database.rs` — Database struct
- `platform/storage/errors.rs` — StorageError

## Риски и зависимости

1. **Тесты:** Каждый рефакторинг требует запуска `make validate` или `make backend-validate` после изменений
2. **Импорты:** После разделения файлов нужно обновить все `use` импорты в зависимых модулях
3. **app/router/routes/:** Некоторые route modules импортируют из `calls_api` и `events_api` — нужно проверить
4. **app/error/conversions/:** Содержат `From` impl для errors — нужно убедиться, что error типы не сломаются
5. **Миграции:** Не затрагиваются — чистое разделение кода
