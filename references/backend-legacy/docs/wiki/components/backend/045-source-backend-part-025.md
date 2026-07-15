---
chunk_id: 045-source-backend-part-025
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 045-source-backend-part-025 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Необходимо обновить страницу `components/backend.md` русской Obsidian-вики, чтобы отразить текущую структуру backend-доменов по фрагменту исходного кода. В контексте представлены три домена — `graph`, `documents::processing` и `obligations` — с их моделями, хранилищами, бизнес-логикой и связями между ними. Изменения коснутся описания архитектуры domain-слоя, ключевых абстракций каждого домена, их статусной модели и механизмов обработки.

## Предложенные страницы

#### `components/backend.md`

```markdown
# Backend-компоненты

## Обзор

Backend (`backend/src`) представляет собой монолитное Rust-приложение, организованное вокруг слоя **domain** (`backend/src/domains`).
В текущем контексте задокументированы три домена: **Graph**, **Documents (Processing)** и **Obligations**, а также используемые ими общие механизмы платформы `crate::platform` — события (`EventStore`), наблюдения (`ObservationStore`) и графовые примитивы (`GraphNodeKind`).

Видимые домены в `mod.rs`:
- `calendar`, `communications`, `decisions`, `documents`, `graph`, `obligations`, `organizations`, `persons`, `projects`, `relationships`, `review`, `settings`, `signal_hub`, `tasks`.

Полный перечень приведён по факту наличия в файле `domains/mod.rs`; детальная документация остальных доменов не подтверждена данным контекстом.

## Graph (`domains::graph`)

### Структура модуля

Модуль делится на:
- `core` — основная реализация (хранилище, модели, валидация, запросы).
- `ports` — ре-экспорт `GraphStore` как `GraphProjectionPort`.

### Модели (`core::models`)

#### `GraphNodeKind` (переиспользуется из `crate::platform::graph`)

Поддерживаемые виды узлов (парсятся из строкового представления в БД):
- `person`, `email_address`, `message`, `document`, `project`, `organization`, `task`, `event`, `decision`, `obligation`, `knowledge`.

Ошибка: `UnknownNodeKind`, если значение не соответствует ни одному из перечисленных.

#### `RelationshipType`

Перечень отношений:
- `person_has_email_address`
- `person_sent_message`
- `person_received_message`
- `email_address_sent_message`
- `email_address_received_message`
- `project_has_message`
- `project_has_document`
- `project_involves_person`
- `project_involves_email_address`
- `entity_relationship`

Ошибка: `UnknownRelationshipType`.

#### `GraphReviewState`

Состояния проверки:
- `system_accepted`
- `suggested`
- `user_confirmed`
- `user_rejected`

Ошибка: `UnknownReviewState`.

#### `GraphEvidenceSourceKind`

Источники доказательств:
- `contact` / `person` (парсятся в `Person`)
- `message`
- `document`
- `relationship`
- `decision`
- `obligation`
- `observation`

Ошибка: `UnknownEvidenceSourceKind`.

#### Основные структуры

- `GraphNode` — узел графа (node_id, node_kind, stable_key, label, properties, created_at, updated_at).
- `GraphEdge` — ребро (edge_id, source_node_id, target_node_id, relationship_type, confidence, review_state, properties, valid_from, valid_to, created_at, updated_at).
- `GraphEvidenceSummary` — сводка доказательства (edge_id, source_kind, source_id, observation_id, excerpt, metadata).
- `GraphNeighborhood` — окрестность узла (selected_node, nodes, edges, evidence, edge_limit, truncated, evidence_limit, evidence_truncated).
- `GraphSummary` — сводная статистика графа (node_counts, edge_counts, evidence_count, latest_projection_at, is_empty).

#### `NewGraphNode`, `NewGraphEdge`, `NewGraphEvidence`

Строители для создания сущностей графа с валидацией.

- `NewGraphEdge.confidence` должен быть в диапазоне `[0.0, 1.0]` включительно.
- `valid_to` не поддерживается — временные рёбра запрещены (`TemporalEdgesUnsupported`).
- Упсёрт ребра требует хотя бы одно доказательство (`SystemEdgeRequiresEvidence`).
- `NewGraphEvidence` с `source_kind=Message` требует `observation_id` (`MissingObservationEvidence`).
- `NewGraphEvidence` с `source_kind=Observation` требует совпадения `source_id` и `observation_id` (`ObservationSourceMismatch`).

### Генерация идентификаторов (`core::ids`)

- `node_id(kind, stable_key)` — делегирует в `crate::platform::graph::node_id` (внутреннее устройство не раскрыто в данном контексте).
- `edge_id(source_node_id, relationship_type, target_node_id)` — формат: `graph:edge:v1:<len:source>:<source>:<len:type>:<type>:<len:target>:<target>`.
- `evidence_id(edge_id, source_kind, source_id)` — формат: `graph:evidence:v1:<len:edge>:<edge>:<len:kind>:<kind>:<len:source>:<source>`.

### Хранилище (`core::store`)

`GraphStore` — основное хранилище графа, также доступное как `GraphProjectionPort`.

Методы уровня экземпляра:
- `upsert_node(&self, node: &NewGraphNode) -> Result<GraphNode>` — upsert по `(node_kind, stable_key)`, обновляет `label` и `properties`.
- `upsert_edge_with_evidence(&self, edge: &NewGraphEdge, evidence: &[NewGraphEvidence]) -> Result<GraphEdge>` — атомарный upsert ребра с доказательствами в одной транзакции.

Статические методы для использования внутри транзакций:
- `upsert_node_in_transaction(transaction, node) -> Result<GraphNode>`
- `upsert_edge_with_evidence_in_transaction(transaction, edge, evidence) -> Result<GraphEdge>`

Upsert ребра использует частичный уникальный индекс: `ON CONFLICT (source_node_id, target_node_id, relationship_type) WHERE valid_to IS NULL`.
Доказательства вставляются с `ON CONFLICT (edge_id, source_kind, source_id) DO UPDATE`, обновляя `observation_id` (через `COALESCE`), `excerpt` и `metadata`.

### Запросы (`core::queries`)

- `summary() -> GraphSummary` — агрегация количества узлов по `node_kind`, рёбер по `relationship_type`, общего числа доказательств, времени последнего обновления и признака пустоты.
- `search_nodes(query, limit) -> Vec<GraphNode>` — поиск по `label` или `stable_key` через `ILIKE`.
- `list_nodes_for_picker(limit) -> Vec<GraphNode>` — узлы, отсортированные по убыванию степени (количество активных рёбер), затем по `updated_at DESC`, `label`, `node_id`.
- `neighborhood(node_id) -> Option<GraphNeighborhood>` — окрестность узла с уровнем изоляции `REPEATABLE READ READ ONLY`. Возвращает рёбра, соседние узлы и доказательства. Если количество рёбер превышает `GRAPH_NEIGHBORHOOD_EDGE_LIMIT` (100), устанавливается флаг `truncated`. Аналогично для доказательств с `GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT` (100) и флагом `evidence_truncated`.

### Ошибки (`core::errors`)

`GraphStoreError` включает варианты:
- `Sqlx` — ошибки БД.
- `EmptyField`, `InvalidJsonObject` — валидация.
- `MissingObservationEvidence`, `ObservationSourceMismatch` — валидация доказательств.
- `InvalidConfidence`, `SystemEdgeRequiresEvidence`, `TemporalEdgesUnsupported` — валидация рёбер.
- `UnknownNodeKind`, `UnknownRelationshipType`, `UnknownReviewState`, `UnknownEvidenceSourceKind` — ошибки разбора значений из БД.

## Documents Processing (`domains::documents::processing`)

### Обзор

Подсистема обработки документов реализует жизненный цикл задач (jobs) с шагами обработки, атомарным выполнением и повторными попытками.

### Модели (`models.rs`)

#### `DocumentProcessingStep`
- `ExtractText` (`extract_text`)
- `Ocr` (`ocr`)

#### `DocumentProcessingStatus`
- `Queued` (`queued`)
- `Running` (`running`)
- `Succeeded` (`succeeded`)
- `Failed` (`failed`)
- `Skipped` (`skipped`)

#### `DocumentArtifactKind`
- `ExtractedText` (`extracted_text`)
- `OcrText` (`ocr_text`)

#### Основные структуры
- `DocumentProcessingJob` — задача (job_id, document_id, step, status, attempts, max_attempts, last_error_summary, queued_at, started_at, finished_at, created_at, updated_at).
- `DocumentProcessingArtifact` — артефакт обработки (artifact_id, document_id, job_id, artifact_kind, content_sha256, text_content, storage_kind, storage_path, metadata, created_at).
- `DocumentProcessingRecord` — агрегат: документ + его задачи + артефакты.
- `DocumentProcessingRunReport` — отчёт о прогоне (jobs_seen, jobs_queued, jobs_succeeded, jobs_failed, jobs_skipped).
- `DocumentProcessingRetryCommand` — команда повтора (command_id, job_id, actor_id).
- `DocumentProcessingRetryCommandResult` — результат повтора (job_id, status, event_id).

### Хранилище и жизненный цикл задач (`jobs.rs`, `store.rs`, `runner.rs`)

`DocumentProcessingStore` управляет задачами обработки.

**Постановка в очередь:**
- `enqueue_for_document(document_id)` — создаёт две задачи для документа: `ExtractText` и `Ocr`.
- `upsert_job(document_id, step)` — upsert задачи. Для уже завершённых (`succeeded`, `skipped`) сохраняет текущий статус, не сбрасывая попытки и ошибки; для остальных сбрасывает в `queued` с `attempts=0`.

**Выборка и выполнение:**
- `next_jobs(limit)` — выбирает задачи со статусом `queued` и `attempts < max_attempts`, упорядоченные по `queued_at ASC, job_id`.
- `run_queued_jobs(limit)` — итеративно обрабатывает выбранные задачи, для каждой вызывая `run_single_job`. Ведёт учёт в `DocumentProcessingRunReport`.
- `run_single_job` — в отдельной транзакции:
  1. `mark_running` — переводит задачу в `running` (атомарно: `status='queued' AND attempts < max_attempts`), увеличивает `attempts` на 1.
  2. Выполняет шаг:
     - `ExtractText`: работает только для документов с `kind == "markdown"`. Если `extracted_text` пуст — ошибка `MissingSourceText`. Для остальных `kind` — возвращает `Skipped`.
     - `Ocr`: всегда возвращает `Skipped` с сообщением `"ocr backend is not configured"`.
  3. При успехе или пропуске вызывает `finish_job` с соответствующим статусом.
  4. При ошибке вызывает `finish_job` со статусом `Failed` и обрезанным до 240 символов сообщением об ошибке (с удалением управляющих символов, кроме `\n`).

**Завершение:**
- `finish_job(tx, job, status, last_error_summary)` — устанавливает статус, ошибку, `finished_at = now()`. После обновления перечитывает задачу с `FOR UPDATE` и фиксирует observation.

**Повтор неудачных задач:**
- `requeue_failed_job(tx, job_id)` — сбрасывает задачу из `failed` в `queued` с `attempts=0`, очищает ошибку и временные метки. Требует статус `failed` (иначе `RetryRequiresFailedJob`).

**Блокировка:**
- `job_for_update(tx, job_id)` — читает задачу с `FOR UPDATE` для пессимистичной блокировки в транзакции.

### Наблюдения (observations)

Каждое изменение статуса задачи фиксируется через `capture_job_observation`:
- Типы событий: `DOCUMENT_PROCESSING_JOB`, `DOCUMENT_PROCESSING_JOB_STATUS`.
- Связь с сущностью через `link_document_processing_entity_in_transaction` с `entity_kind = "document_processing_job"`.
- В полезную нагрузку observation включаются: job_id, document_id, step, status, attempts, max_attempts, last_error_summary, временные метки.

### Механизм повтора (`retry.rs`)

`DocumentProcessingStore::retry_failed_job` — идемпотентный повтор через event sourcing:
1. Валидация входных параметров (непустые строки).
2. Проверка существующего события повтора по `event_id = RETRY_EVENT_ID_PREFIX + command_id`. Если найдено — возвращает сохранённый результат (идемпотентность).
3. Если задача не в статусе `failed` — ошибка `RetryRequiresFailedJob` (после дополнительной попытки найти событие).
4. Создаёт событие `NewEventEnvelope` с типом `RETRY_EVENT_TYPE`, source `RETRY_SOURCE_KIND` / `RETRY_SOURCE_PROVIDER` и payload, содержащим `job_id`.
5. При конфликте уникальности события — откат транзакции и возврат результата существующего события.
6. Вызывает `requeue_failed_job` в той же транзакции.
7. При наличии `observation_id` связывает наблюдение с задачей через `link_document_processing_entity_in_transaction`.

Константы повтора (точные значения не раскрыты в данном контексте — объявлены в `constants.rs`, который не включён в контекст).

### Сервис команд (`service.rs`)

`DocumentProcessingCommandService` — слой над хранилищем для ручного повтора:
- `retry_failed_job_manual(command)`:
  1. Вызывает `DocumentProcessingStore::retry_failed_job`.
  2. Создаёт observation с `origin_kind = Manual` и типом `DOCUMENT_PROCESSING_JOB_STATUS`.
  3. Повторно вызывает `retry_failed_job_with_observation`, передавая `observation_id`.
- Ошибки агрегируются через `DocumentProcessingCommandServiceError`, объединяющую `DocumentProcessingError` и `ObservationStoreError`.

### Валидация (`validation.rs`)

- `validate_non_empty(field, value)` — поле не должно быть пустым.
- `validate_limit(limit)` — должен быть в диапазоне `[MIN_LIST_LIMIT, MAX_LIST_LIMIT]` (конкретные значения констант не включены в контекст).
- `validate_optional_limit(limit)` — если `None`, подставляет `DEFAULT_LIST_LIMIT`.

## Obligations (`domains::obligations`)

### Структура модуля

Модуль включает: `errors`, `evidence`, `ids`, `models`, `ports`, `row_mapping`, `service`, `store`, `validation`.

Публичный API:
- `ObligationStore` (также доступен как `ObligationReviewPort`).
- `ObligationCommandService`.
- Модели: `NewObligation`, `NewObligationEvidence`, `Obligation`, `ObligationEntityKind`, `ObligationEvidenceSourceKind`, `ObligationReviewState`, `ObligationRiskState`, `ObligationStatus`.
- ID-генераторы: `obligation_id`, `evidence_id`.
- Ошибки: `ObligationStoreError` (также `ObligationReviewPortError`), `ObligationCommandServiceError`.

### Модели (`models.rs`)

Детали моделей (`entity_kind`, `evidence`, `obligation`, `read_model`, `source_kind`, `states`) не раскрыты в данном контексте — доступен только верхнеуровневый ре-экспорт из `models/mod.rs`.

### Идентификаторы (`ids.rs`)

- `obligation_id(obligation: &NewObligation)` — строится из:
  - `obligated_entity_kind` и `obligated_entity_id`
  - `beneficiary_entity_kind` и `beneficiary_entity_id` (оба опциональны; при отсутствии подставляется пустая строка)
  - нормализованного `statement` (приведение к нижнему регистру, схлопывание пробелов)

  Формат: `obligation:v1:<len:obligated_kind>:<obligated_kind>:<len:obligated_id>:<obligated_id>:<len:beneficiary_kind>:<beneficiary_kind>:<len:beneficiary_id>:<beneficiary_id>:<len:statement>:<statement>`.

- `evidence_id(obligation_id, source_kind, source_id)` — формат: `obligation:evidence:v1:<len:obligation>:<obligation>:<len:kind>:<kind>:<len:source>:<source>`.

### Связь с наблюдениями (`evidence.rs`)

- `link_obligation_support_in_transaction` — связывает observation с обязательством через `link_domain_entity_in_transaction` с отношением `"supports"`, передавая `confidence` и `metadata`.
- `link_obligation_review_transition_in_transaction` — материализует переход состояния проверки обязательства через `materialize_review_transition_link_in_transaction`.

### Состояния и проверка

Контекст содержит ре-экспорт `ObligationReviewState`, `ObligationRiskState`, `ObligationStatus` из `models::states`, но их конкретные варианты не включены в данный контекст.

### Ошибки (`errors.rs`)

`ObligationStoreError` включает:
- `Sqlx`, `Observation` — ошибки нижележащих слоёв.
- `EmptyField`, `InvalidJsonObject` — валидация полей.
- `InvalidScore(field, value)` — оценка должна быть в `[0.0, 1.0]`.
- `MissingEvidence` — доказательства обязательны.
- `InvalidObservationEvidenceSource` — несовпадение source_id/observation_id для observation-доказательств.
- `ObservationNotFound` — указанное observation не найдено.
- `ObligationNotFound` — обязательство не найдено.
- `PartialBeneficiary` — beneficiary entity kind и id должны быть указаны вместе.
- `UnknownEntityKind`, `UnknownEvidenceSourceKind`, `UnknownStatus`, `UnknownReviewState`, `UnknownRiskState` — ошибки разбора значений из БД.

## Общие механизмы платформы

Контекст раскрывает следующие платформенные компоненты, используемые доменами:
- `EventStore` (`crate::platform::events`) — добавление и чтение событий, проверка уникальности (`is_unique_violation`).
- `ObservationStore` (`crate::platform::observations`) — создание наблюдений, связывание с сущностями через `link_domain_entity_in_transaction`, материализация переходов review-состояний через `materialize_review_transition_link_in_transaction`.
- `graph::node_id` (`crate::platform::graph`) — генерация идентификаторов узлов графа.

Детали реализации этих компонентов не включены в данный контекст.

## Примечания по контексту

- Конкретные значения констант `DEFAULT_MAX_ATTEMPTS`, `MIN_LIST_LIMIT`, `MAX_LIST_LIMIT`, `DEFAULT_LIST_LIMIT`, `RETRY_EVENT_ID_PREFIX`, `RETRY_EVENT_TYPE`, `RETRY_SOURCE_KIND`, `RETRY_SOURCE_PROVIDER` не подтверждены данным контекстом — соответствующие файлы `constants.rs` для `documents::processing` и `graph` не включены в чанк (за исключением `graph::core::constants`, который содержит только лимиты окрестности).
- Детали моделей `ObligationEntityKind`, `ObligationEvidenceSourceKind`, `ObligationReviewState`, `ObligationRiskState`, `ObligationStatus`, `NewObligation`, `NewObligationEvidence`, `Obligation` не раскрыты — в контексте присутствует только `mod.rs` с ре-экспортами.
- Реализация `ObligationStore` и `ObligationCommandService` не включена в данный контекст.
```

## Покрытие источников

| Source file | Covered facts |
|---|---|
| `backend/src/domains/graph/core.rs` | Структура модуля `graph::core`, ре-экспортируемые элементы: `GraphStore`, `GraphProjectionPort`, модели, константы, ошибки, функции генерации ID |
| `backend/src/domains/graph/core/constants.rs` | Константы `GRAPH_NEIGHBORHOOD_EDGE_LIMIT=100` и `GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT=100` |
| `backend/src/domains/graph/core/errors.rs` | Перечень вариантов `GraphStoreError`, их сообщения и семантика |
| `backend/src/domains/graph/core/ids.rs` | Форматы `edge_id` и `evidence_id`, делегирование `node_id` в `platform::graph` |
| `backend/src/domains/graph/core/models.rs` | Все типы узлов, отношений, состояний проверки, источников доказательств; структуры `GraphNode`, `GraphEdge`, `GraphNeighborhood`, `GraphSummary`; строители `NewGraphNode`, `NewGraphEdge`, `NewGraphEvidence` с правилами валидации |
| `backend/src/domains/graph/core/queries.rs` | Реализация методов `summary`, `search_nodes`, `list_nodes_for_picker`, `neighborhood` с SQL-запросами, уровнем изоляции и логикой усечения |
| `backend/src/domains/graph/core/row_mapping.rs` | Функции маппинга строк БД в модели, включая парсинг строковых представлений enum-значений с обработкой обратной совместимости (`contact`/`person`) |
| `backend/src/domains/graph/core/store.rs` | Реализация `GraphStore`: upsert узлов и рёбер с доказательствами (включая статические методы для транзакций), SQL с `ON CONFLICT` и частичным уникальным индексом |
| `backend/src/domains/graph/core/validation.rs` | Валидация `NewGraphEdge` и `NewGraphEvidence`, требование наличия доказательств, проверка JSON-объектов |
| `backend/src/domains/graph/mod.rs` | Декларация подмодулей `core` и `ports` |
| `backend/src/domains/graph/ports.rs` | Алиас `GraphProjectionPort = GraphStore` |
| `backend/src/domains/documents/processing/jobs.rs` | Методы `upsert_job`, `next_jobs`, `mark_running`, `job_for_update`, `requeue_failed_job`, `finish_job`; структура `QueuedJob`; фиксация observations при переходах статусов |
| `backend/src/domains/documents/processing/models.rs` | `DocumentProcessingStep`, `DocumentProcessingStatus`, `DocumentArtifactKind`, `DocumentProcessingJob`, `DocumentProcessingArtifact`, `DocumentProcessingRecord`, `DocumentProcessingRunReport`, `DocumentProcessingRetryCommand`, `DocumentProcessingRetryCommandResult` |
| `backend/src/domains/documents/processing/retry.rs` | Идемпотентный механизм повтора через event sourcing: `retry_failed_job`, `retry_failed_job_with_observation`, `RetryCommandEvent`, связывание с observation |
| `backend/src/domains/documents/processing/rows.rs` | Функции `try_row_to_job` и `try_row_to_artifact` — маппинг строк БД в модели обработки документов |
| `backend/src/domains/documents/processing/runner.rs` | `run_queued_jobs`, `run_single_job`, логика шагов `ExtractText` (только markdown) и `Ocr` (заглушка), функция `safe_summary` с тестом |
| `backend/src/domains/documents/processing/service.rs` | `DocumentProcessingCommandService::retry_failed_job_manual` с созданием manual-observation и повторным вызовом с observation_id |
| `backend/src/domains/documents/processing/store.rs` | Публичные методы `DocumentProcessingStore`: `enqueue_for_document`, `list_jobs`, `list_jobs_for_document`, `list_artifacts_for_document`, `document_processing` |
| `backend/src/domains/documents/processing/validation.rs` | Функции `validate_non_empty`, `validate_limit`, `validate_optional_limit` со ссылками на константы |
| `backend/src/domains/obligations/errors.rs` | Перечень вариантов `ObligationStoreError` |
| `backend/src/domains/obligations/evidence.rs` | Функции `link_obligation_support_in_transaction` и `link_obligation_review_transition_in_transaction` |
| `backend/src/domains/obligations/ids.rs` | Формат `obligation_id` с нормализацией statement; формат `evidence_id` |
| `backend/src/domains/obligations/mod.rs` | Публичное API домена obligations: store, service, модели, ID-генераторы, ошибки |
| `backend/src/domains/obligations/models.rs` | Ре-экспорт моделей из подмодулей (детали подмодулей не включены в контекст) |
| `backend/src/domains/mod.rs` | Полный перечень доменов backend |

## Исходные файлы

- [`backend/src/domains/documents/processing/jobs.rs`](../../../../backend/src/domains/documents/processing/jobs.rs)
- [`backend/src/domains/documents/processing/models.rs`](../../../../backend/src/domains/documents/processing/models.rs)
- [`backend/src/domains/documents/processing/retry.rs`](../../../../backend/src/domains/documents/processing/retry.rs)
- [`backend/src/domains/documents/processing/rows.rs`](../../../../backend/src/domains/documents/processing/rows.rs)
- [`backend/src/domains/documents/processing/runner.rs`](../../../../backend/src/domains/documents/processing/runner.rs)
- [`backend/src/domains/documents/processing/service.rs`](../../../../backend/src/domains/documents/processing/service.rs)
- [`backend/src/domains/documents/processing/store.rs`](../../../../backend/src/domains/documents/processing/store.rs)
- [`backend/src/domains/documents/processing/validation.rs`](../../../../backend/src/domains/documents/processing/validation.rs)
- [`backend/src/domains/graph/core.rs`](../../../../backend/src/domains/graph/core.rs)
- [`backend/src/domains/graph/core/constants.rs`](../../../../backend/src/domains/graph/core/constants.rs)
- [`backend/src/domains/graph/core/errors.rs`](../../../../backend/src/domains/graph/core/errors.rs)
- [`backend/src/domains/graph/core/ids.rs`](../../../../backend/src/domains/graph/core/ids.rs)
- [`backend/src/domains/graph/core/models.rs`](../../../../backend/src/domains/graph/core/models.rs)
- [`backend/src/domains/graph/core/queries.rs`](../../../../backend/src/domains/graph/core/queries.rs)
- [`backend/src/domains/graph/core/row_mapping.rs`](../../../../backend/src/domains/graph/core/row_mapping.rs)
- [`backend/src/domains/graph/core/store.rs`](../../../../backend/src/domains/graph/core/store.rs)
- [`backend/src/domains/graph/core/validation.rs`](../../../../backend/src/domains/graph/core/validation.rs)
- [`backend/src/domains/graph/mod.rs`](../../../../backend/src/domains/graph/mod.rs)
- [`backend/src/domains/graph/ports.rs`](../../../../backend/src/domains/graph/ports.rs)
- [`backend/src/domains/mod.rs`](../../../../backend/src/domains/mod.rs)
- [`backend/src/domains/obligations/errors.rs`](../../../../backend/src/domains/obligations/errors.rs)
- [`backend/src/domains/obligations/evidence.rs`](../../../../backend/src/domains/obligations/evidence.rs)
- [`backend/src/domains/obligations/ids.rs`](../../../../backend/src/domains/obligations/ids.rs)
- [`backend/src/domains/obligations/mod.rs`](../../../../backend/src/domains/obligations/mod.rs)
- [`backend/src/domains/obligations/models.rs`](../../../../backend/src/domains/obligations/models.rs)

## Кандидаты на drift

- **Константы `documents::processing` не раскрыты.** Файл `processing/constants.rs` не включён в контекст. Значения `DEFAULT_MAX_ATTEMPTS`, `MIN_LIST_LIMIT`, `MAX_LIST_LIMIT`, `DEFAULT_LIST_LIMIT`, а также константы `RETRY_EVENT_ID_PREFIX`, `RETRY_EVENT_TYPE`, `RETRY_SOURCE_KIND`, `RETRY_SOURCE_PROVIDER` не подтверждены. При расхождении с документацией в других источниках это может быть drift.
- **Модели `obligations::models` не раскрыты.** Только `models/mod.rs` с ре-экспортами. Конкретные варианты `ObligationEntityKind`, `ObligationEvidenceSourceKind`, `ObligationReviewState`, `ObligationRiskState`, `ObligationStatus`, а также структуры `NewObligation`, `Obligation`, `NewObligationEvidence` не подтверждены контекстом. Если другие wiki-страницы описывают эти модели, возможен drift.
- **Реализация `ObligationStore` и `ObligationCommandService` не включена.** Файлы `store.rs` и `service.rs` не попали в чанк. Их поведение не подтверждено. При наличии документации этих методов в других страницах вики возможно расхождение.
- **Платформенные компоненты не раскрыты.** `EventStore`, `ObservationStore`, `graph::node_id` используются доменами, но их внутренняя реализация не включена в контекст. Если вики содержит детали их поведения, они не могут быть верифицированы из данного чанка.
- **Другие домены (`domains/mod.rs`).** Перечислены `calendar`, `communications`, `decisions`, `organizations`, `persons`, `projects`, `relationships`, `review`, `settings`, `signal_hub`, `tasks` — их исходный код не включён. Если вики содержит страницы для этих доменов, расхождение не может быть обнаружено из данного контекста.
