## Summary / Резюме

Требуется дополнить страницу `components/backend.md` в русской Obsidian‑wiki информацией о компонентах доменов `persons` и `projects`, полученной из встроенных исходных файлов текущего чанка. Описываются структуры данных, хранилища, основные операции, порты и модели, присутствующие в этих доменах, чтобы отразить реальное состояние кодовой базы в документации.

## Proposed pages / Предлагаемые страницы

### `components/backend.md`

```markdown
# Backend

Backend организован по доменам. Каждый домен включает подмодули для хранения, моделей и бизнес‑логики. В данном чанке представлены домены `persons` и `projects`.

## Домен persons

Корневой модуль: `backend/src/domains/persons/mod.rs` объявляет следующие подмодули: `analytics`, `api`, `command_service`, `core`, `enrichment`, `enrichment_engine`, `expertise`, `export`, `health`, `identity`, `intelligence`, `investigator`, `memory`, `ports`, `service`, `trust`.

### Порты (`ports`)

Файл `ports.rs` переэкспортирует типы‑порты:

- `PersonProjectionStore` (из `super::api`) как `PersonProjectionPort`.
- `RelationshipEventStore` (из `super::memory`) как `RelationshipEventPort`.

Полный код портов не встроен в данный контекст, поэтому поведение `PersonProjectionPort` не подтверждено.

### Сервис (`service`)

Файл `service.rs` просто переэкспортирует всё из `super::command_service`. Детали `command_service` в данном контексте не раскрыты.

### Память (`memory`)

Подмодуль `memory` содержит хранилища для событий отношений и снимков персон.

#### `RelationshipEventStore`

Расположен в `backend/src/domains/persons/memory/relationship_events.rs`.

**Модель `RelationshipEvent`:**

```rust
pub struct RelationshipEvent {
    pub id: String,
    pub person_id: String,
    pub event_type: String,
    pub title: String,
    pub description: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub source: String,
    pub related_entity_id: Option<String>,
    pub related_entity_kind: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}
```

**Публичные методы:**

- `timeline(person_id: &str, limit: i64) -> Vec<RelationshipEvent>`  
  Возвращает последние события персоны, отсортированные по `occurred_at` по убыванию. Лимит ограничивается через `TimelineEngine::bounded_entity_limit(limit)`.

- `add(event: &NewRelationshipEvent) -> RelationshipEvent`  
  Создаёт новое событие. Перед вставкой вызывает `TimelineEngine::validate_event` с `entity_kind = "persona"`.

- `add_with_observation(event: &NewRelationshipEvent, observation_id: &str) -> RelationshipEvent`  
  Добавляет событие и связывает его с наблюдением через `link_persons_entity`, передавая в метаданных `person_id` и `event_type`.

- `upsert_email_message_event(...)`  
  Идемпотентно создаёт событие, связанное с email‑сообщением. Использует `source = "email_sync"` и `related_entity_kind = "communication_message"`. Проверяет отсутствие дубликата через `WHERE NOT EXISTS`. При создании связывает событие с наблюдением через `link_persons_entity_in_transaction` с указанием `link_tag = "email_sync_relationship_event"`.

**Модель `NewRelationshipEvent`:**

```rust
pub struct NewRelationshipEvent {
    pub person_id: String,
    pub event_type: String,
    pub title: String,
    pub description: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub source: String,
    pub related_entity_id: Option<String>,
    pub related_entity_kind: Option<String>,
}
```

#### `PersonSnapshotStore`

Расположен в `backend/src/domains/persons/memory/snapshots.rs`.

**Модель `PersonSnapshot`:**

```rust
pub struct PersonSnapshot {
    pub id: String,
    pub person_id: String,
    pub snapshot_date: DateTime<Utc>,
    pub data: Value,
    pub source: String,
    pub created_at: DateTime<Utc>,
}
```

**Публичные методы:**

- `list(person_id: &str) -> Vec<PersonSnapshot>`  
  Возвращает до 20 последних снимков, отсортированных по `snapshot_date` по убыванию.

- `create(person_id: &str, data: Value, source: &str) -> PersonSnapshot`  
  Создаёт новый снимок. Поле `snapshot_date` генерируется на уровне БД.

- `history_diff(person_id: &str, from_date: DateTime<Utc>, to_date: DateTime<Utc>) -> HistoryDiff`  
  Вычисляет разницу между ближайшими снимками на каждый из переданных моментов времени. Возвращает `HistoryDiff` со списком изменённых полей.

**Модели `HistoryDiff` и `FieldChange`:**

```rust
pub struct HistoryDiff {
    pub person_id: String,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub changes: Vec<FieldChange>,
}

pub struct FieldChange {
    pub field: String,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
}
```

### Доверие (`trust`)

Подмодуль `trust` управляет обещаниями (`promises`) и рисками (`risks`), а также состоянием здоровья персоны.

Расположен в `backend/src/domains/persons/trust.rs`, который переэкспортирует:

- Ошибки: `PersonTrustError`.
- Модели: `PersonPromise`, `PersonRisk`.
- Константу: `PERSON_PROMISE_CREATED_EVENT_TYPE`.
- Хранилища: `PersonPromiseStore`, `PersonRiskStore`.

#### Ошибки (`trust/errors.rs`)

`PersonTrustError` объединяет:

- `Sqlx` (`sqlx::Error`)
- `RiskEngine` (`crate::engines::risk::RiskEngineError`)
- `Observation` (`crate::platform::observations::ObservationStoreError`)
- `Event` (`EventStoreError`)

#### Модели (`trust/models.rs`)

```rust
pub struct PersonPromise {
    pub id: String,
    pub person_id: String,
    pub description: String,
    pub source_message_id: Option<String>,
    pub promised_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct PersonRisk {
    pub id: String,
    pub person_id: String,
    pub risk_type: String,
    pub description: String,
    pub severity: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
}
```

#### `PersonPromiseStore` (`trust/promises.rs`)

Константа: `PERSON_PROMISE_CREATED_EVENT_TYPE = "person.promise.created"`.

Методы:

- `list(person_id: &str) -> Vec<PersonPromise>` – все обещания персоны, отсортированные по `promised_at` по убыванию.
- `create(person_id, description, due_at) -> PersonPromise` – создаёт обещание в транзакции, затем формирует событие типа `person.promise.created` через `EventStore::append_in_transaction`. Событие использует идентификатор `person_promise_created:{promise.id}`, `aggregate_kind = "person_promise"`, `subject_kind = "persona"`, и payload с полями `promise_id`, `person_id`, `description`, `due_at`. Повторная вставка идемпотентна за счёт обработки `unique_violation` в событийном хранилище.
- `fulfill(id: &str)` – помечает статус `'fulfilled'`, выставляет `fulfilled_at = now()`.
- `mark_broken(id: &str)` – устанавливает статус `'broken'`.

#### `PersonRiskStore` (`trust/risks.rs`)

Методы:

- `list(person_id: &str) -> Vec<PersonRisk>` – все риски персоны, по `created_at` убыванию.
- `report(person_id, risk_type, description, severity, source) -> PersonRisk` – создаёт риск в транзакции, используя `RiskEngine::persona_observation` для валидации. После вставки вызывает `sync_person_health_status_in_transaction` для пересчёта статуса здоровья.
- `resolve(id: &str, resolution: &str)` – разрешает риск (проставляет `resolved_at` и `resolution`), затем пересчитывает статус здоровья персоны.

#### Проекция здоровья (`trust/health_projection.rs`)

Функция `sync_person_health_status_in_transaction`:

- Выбирает все неразрешённые риски персоны (где `resolved_at IS NULL`).
- Преобразует их в `RiskSignal::unresolved` с парсингом `RiskSeverity::parse`.
- Вычисляет `health_status` через `RiskEngine::derive_attention_status` и преобразует в `as_persona_health_status()`.
- Обновляет строку в таблице `persons`, устанавливая `health_status`, `last_health_check = now()`, `updated_at = now()`.

Преобразование строк БД в модели `PersonPromise` и `PersonRisk` выполняют функции `row_to_promise` и `row_to_risk` из `trust/rows.rs`.

## Домен projects

Домен `projects` имеет ядро (`core`), а также подмодуль `link_reviews`, упоминаемый в декларациях ошибок и в методах проекции. Полный код `ProjectLinkReviewStore` в данном контексте отсутствует.

### Ядро (`projects/core`)

Файл `core.rs` экспортирует:

- Модели: `Project`, `NewProject`, `ProjectDetail`, `ProjectDocumentSummary`, `ProjectMessageSummary`, `ProjectPersonSummary`, `ProjectStats`, `ProjectSummary`, `ProjectTimelineItem`, `ProjectListResponse`, а также внутри крейта `ProjectMatchedMessage`, `ProjectMatchedDocument`, `ProjectProjectionSource`.
- Идентификаторы: `project_graph_node_id`.
- Хранилище: `ProjectStore` (оно же `ProjectCommandPort`).
- Ошибки: `ProjectStoreError` (также экспортируется как `ProjectCommandPortError`).

#### Константы (`core/constants.rs`)

- `DEFAULT_PROJECT_LIMIT: i64 = 25`
- `MAX_PROJECT_LIMIT: i64 = 100`
- `PROJECT_DETAIL_ITEM_LIMIT: i64 = 8`

#### Ошибки (`core/errors.rs`)

`ProjectStoreError` содержит варианты:

- `Sqlx` (транзитивное от `sqlx::Error`)
- `EmptyField(&'static str)` – поле не должно быть пустым
- `InvalidProgress(i32)` – progress_percent должен быть 0..100
- `NoKeywords` – проект должен иметь хотя бы одно ключевое слово
- `ProjectLinkReview` – транзитивная ошибка `ProjectLinkReviewError`
- `InvalidLimit`
- `InvalidRecipients`

#### Идентификаторы (`core/ids.rs`)

- `project_graph_node_id(project_id: &str) -> String` возвращает строку `"graph:node:v1:project:{project_id}"`.

#### Модели (`core/models.rs`)

**`NewProject`** – построитель нового проекта. Предоставляет методы:

- `active(project_id, name, kind, description, owner_display_name, keywords)` – устанавливает `status = "active"` и `progress_percent = 0`.
- `progress(percent)` – задаёт процент прогресса.
- `validate() -> Result<ValidatedProject, ProjectStoreError>` – проверяет непустоту обязательных полей, границы `progress_percent`, приводит ключевые слова к строчному регистру, удаляет дубликаты и требует наличие хотя бы одного ключевого слова.

**`ValidatedProject`** – внутренняя модель после валидации, с теми же полями, но гарантированно валидными.

**`Project`** – публичная модель проекта с `created_at`, `updated_at` и без ключевых слов (ключевые слова хранятся отдельно).

**`ProjectStats`** – статистика: `message_count`, `document_count`, `people_count`, `graph_connection_count`, `latest_activity_at`.

**`ProjectSummary`** – объединяет `Project`, `ProjectStats` и `graph_node_id`.

**`ProjectDetail`** – расширенная информация: `Project`, `ProjectStats`, `graph_node_id`, `timeline`, `key_people`, `recent_messages`, `documents`.

**`ProjectTimelineItem`** – элемент временной шкалы: `item_kind`, `item_id`, `title`, `subtitle`, `occurred_at`.

**`ProjectPersonSummary`** – персона в проекте: `display_name`, `email_address`, `interaction_count`, `last_interaction_at`.

**`ProjectMessageSummary`** – сообщение в проекте: `message_id`, `subject`, `sender`, `occurred_at`.

**`ProjectDocumentSummary`** – документ в проекте: `document_id`, `document_kind`, `title`, `observation_id`, `imported_at`.

**`ProjectListResponse`** – список проектов: `items: Vec<ProjectSummary>`.

Внутренние модели (crate‑private):

- `ProjectProjectionSource` – для графовой проекции: `project` и `keywords`.
- `ProjectMatchedMessage` – сообщение с `review_state: ProjectLinkReviewState`.
- `ProjectMatchedDocument` – документ с `review_state: ProjectLinkReviewState`.

#### `ProjectStore`

Хранилище проектов. Экземпляр создаётся с пулом соединений `PgPool`. Большинство методов определены в подмодулях `read_model` и `projection`.

**Методы Read‑модели** (из `core/read_model/`):

- `project_by_id(project_id: &str) -> Option<Project>` – поиск проекта по идентификатору.
- `project_keywords(project_id: &str) -> Vec<String>` – ключевые слова проекта.
- `project_messages(project_id, limit) -> Vec<ProjectMessageSummary>` – сообщения, привязанные к проекту, через список активных идентификаторов сообщений (`active_project_messages`). Использует `COALESCE(occurred_at, projected_at)` для даты.
- `project_documents(project_id, limit) -> Vec<ProjectDocumentSummary>` – документы проекта через активные идентификаторы.
- `project_people(project_id, limit) -> Vec<ProjectPersonSummary>` – участники проекта. Извлекаются из отправителей и получателей сообщений, затем группируются по email, LEFT JOIN с таблицей `persons` для подстановки `display_name`. Пустые email‑адреса исключаются. Сортировка: по числу взаимодействий, затем по дате последнего взаимодействия.
- `active_project_messages(project_id) -> Vec<ProjectReviewedTarget>` – делегирует в `ProjectLinkReviewStore::active_message_ids_for_project`.
- `active_project_documents(project_id) -> Vec<ProjectReviewedTarget>` – делегирует в `ProjectLinkReviewStore::active_document_ids_for_project`.

**Методы проекции** (из `core/projection.rs`):

- `graph_projection_projects() -> Vec<ProjectProjectionSource>` – все проекты с ключевыми словами для графовой проекции.
- `matching_project_messages(project_id) -> Vec<ProjectMatchedMessage>` – сообщения с признаком `review_state` из `ProjectLinkReviewState` (по умолчанию `Suggested`, если запись отсутствует).
- `matching_project_documents(project_id) -> Vec<ProjectMatchedDocument>` – документы с `review_state`.

**Вспомогательные функции** (из `projection.rs`):

- `reviewed_targets_and_map(targets) -> (Vec<String>, HashMap<String, ProjectLinkReviewState>)` – строит словарь `target_id → review_state` и список идентификаторов.
- `reviewed_target_ids(targets) -> Vec<String>` – только идентификаторы целей.

Реализация остальных методов `ProjectStore` (например, создание, обновление) не содержится в данном чанке.

```

## Source coverage / Покрытие источников

- **`backend/src/domains/persons/memory/relationship_events.rs`**  
  Покрыты: структура `RelationshipEvent`, `NewRelationshipEvent`, все публичные методы `RelationshipEventStore` (`timeline`, `add`, `add_with_observation`, `upsert_email_message_event`), использование `TimelineEngine::validate_event` с `entity_kind = "persona"`.

- **`backend/src/domains/persons/memory/snapshots.rs`**  
  Покрыты: `PersonSnapshot`, `PersonSnapshotStore` с методами `list`, `create`, `history_diff`, модели `HistoryDiff` и `FieldChange`, логика сравнения снимков.

- **`backend/src/domains/persons/mod.rs`**  
  Перечислены все объявленные подмодули домена `persons`.

- **`backend/src/domains/persons/ports.rs`**  
  Задокументированы реэкспорты `PersonProjectionPort` и `RelationshipEventPort`.

- **`backend/src/domains/persons/service.rs`**  
  Отмечено, что сервис переэкспортирует `command_service`, без раскрытия содержания.

- **`backend/src/domains/persons/trust.rs`**  
  Перечислены реэкспорты: ошибки, модели, константа, хранилища.

- **`backend/src/domains/persons/trust/errors.rs`**  
  Описаны варианты `PersonTrustError`.

- **`backend/src/domains/persons/trust/health_projection.rs`**  
  Описан алгоритм `sync_person_health_status_in_transaction`: выборка неразрешённых рисков, вычисление статуса через `RiskEngine::derive_attention_status`, обновление таблицы `persons`.

- **`backend/src/domains/persons/trust/models.rs`**  
  Приведены полные определения `PersonPromise` и `PersonRisk`.

- **`backend/src/domains/persons/trust/promises.rs`**  
  Описаны: `PersonPromiseStore` с методами `list`, `create`, `fulfill`, `mark_broken`; константа `PERSON_PROMISE_CREATED_EVENT_TYPE`; логика формирования события `person.promise.created` с использованием `NewEventEnvelope` и `EventStore::append_in_transaction`; идемпотентность через обработку `unique_violation`.

- **`backend/src/domains/persons/trust/risks.rs`**  
  Описаны: `PersonRiskStore` с методами `list`, `report` (с вызовом `RiskEngine::persona_observation` и `sync_person_health_status_in_transaction`), `resolve` (с пересчётом здоровья).

- **`backend/src/domains/persons/trust/rows.rs`**  
  Задокументировано наличие функций `row_to_promise` и `row_to_risk`; детали опущены, но покрыты описанием маппинга.

- **`backend/src/domains/projects/core.rs`**  
  Перечислены публичные экспорты из ядра проектов.

- **`backend/src/domains/projects/core/constants.rs`**  
  Зафиксированы значения `DEFAULT_PROJECT_LIMIT`, `MAX_PROJECT_LIMIT`, `PROJECT_DETAIL_ITEM_LIMIT`.

- **`backend/src/domains/projects/core/errors.rs`**  
  Приведены варианты `ProjectStoreError`.

- **`backend/src/domains/projects/core/ids.rs`**  
  Документирована функция `project_graph_node_id`.

- **`backend/src/domains/projects/core/models.rs`**  
  Описаны `NewProject` (с методом `active`, `progress`, `validate`), `ValidatedProject`, `Project`, `ProjectStats`, `ProjectSummary`, `ProjectDetail`, `ProjectTimelineItem`, `ProjectPersonSummary`, `ProjectMessageSummary`, `ProjectDocumentSummary`, `ProjectListResponse`, а также crate‑private `ProjectProjectionSource`, `ProjectMatchedMessage`, `ProjectMatchedDocument`.

- **`backend/src/domains/projects/core/projection.rs`**  
  Описаны методы `graph_projection_projects`, `matching_project_messages`, `matching_project_documents` и вспомогательные функции.

- **`backend/src/domains/projects/core/read_model.rs`**  
  Задокументированы подмодули read‑модели.

- **`backend/src/domains/projects/core/read_model/documents.rs`**  
  Описан метод `project_documents`.

- **`backend/src/domains/projects/core/read_model/keywords.rs`**  
  Описан метод `project_keywords`.

- **`backend/src/domains/projects/core/read_model/messages.rs`**  
  Описан метод `project_messages`.

- **`backend/src/domains/projects/core/read_model/people.rs`**  
  Описан метод `project_people` с деталями агрегирующего запроса.

- **`backend/src/domains/projects/core/read_model/projects.rs`**  
  Описан метод `project_by_id`.

- **`backend/src/domains/projects/core/read_model/reviewed_targets.rs`**  
  Описаны методы `active_project_messages` и `active_project_documents`, делегирующие в `ProjectLinkReviewStore`.

## Drift candidates / Кандидаты на drift

Из данного контекста не выявлено явных расхождений между кодом и его документированием, поскольку в чанке отсутствуют предшествующие версии wiki‑страниц, ADR или конфигурационные файлы.

Однако следует отметить:

- `PersonProjectionPort` упомянут в `ports.rs` как реэкспорт из `super::api`, но исходный код `api` не встроен в этот контекст. Фактическое поведение этого порта не может быть подтверждено.
- `ProjectStore` экспортируется как `ProjectCommandPort`, но реализация `store.rs` и методы, связанные с командами (создание, обновление), не присутствуют в чанке. Соответственно, полнота покрытия командного интерфейса не гарантирована.
- `ProjectLinkReviewStore` используется в методах `ProjectStore`, но его исходный код также отсутствует; типы `ProjectLinkReviewState` и `ProjectReviewedTarget` видны только в употреблении. Возможна несинхронизация между реальной схемой этих типов и документированным использованием – но из предоставленных фрагментов она не видна.
