## Summary / Резюме

Страница `components/backend.md` должна быть создана (или дополнена) документацией по backend-компонентам, обнаруженным в исходных файлах этого чанка. В частности, требуется описать домены `projects` (core, read‑модели, link‑review‑подсистема) и `relationships` (модели, идентификаторы, ошибки). Каждое утверждение должно опираться исключительно на встроенный исходный код.

## Proposed pages / Предлагаемые страницы

### `components/backend.md`

```markdown
# Backend‑компоненты

## Обзор

Backend системы построен на Rust с использованием `sqlx` и `PgPool` в качестве основного механизма доступа к PostgreSQL. Логика сгруппирована в домены (crate‑модули), каждый из которых содержит модели, хранилище (`Store`), read‑модели и, при необходимости, сервисный слой.

В данном чанке представлены два домена:

- `projects` (ядро и подсистема `link_reviews`)
- `relationships` (модели, идентификаторы, ошибки)

---

## Домен `projects`

### Ядро (`backend/src/domains/projects/core`)

#### `ProjectStore`

Хранилище проектов располагается в файле `store.rs`.

Пул соединений хранится в поле `pool: PgPool`. Конструктор `ProjectStore::new(pool)` принимает готовый пул.

**Методы:**

- `upsert_project(project: &NewProject) -> Result<Project, ProjectStoreError>`  
  Выполняет валидацию через `project.validate()`, затем внутри транзакции выполняет `INSERT … ON CONFLICT … DO UPDATE`, после чего удаляет и пересоздаёт связанные ключевые слова (`project_keywords`). Возвращает вставленную/обновлённую запись через `row_to_project`.

- `list_projects(limit: Option<i64>) -> Result<Vec<ProjectSummary>, ProjectStoreError>`  
  Приводит `limit` к допустимому диапазону через `validate_limit` (по умолчанию — `DEFAULT_PROJECT_LIMIT`, максимальное значение — `MAX_PROJECT_LIMIT`). Запрашивает проекты, сортируя по `updated_at DESC`, затем вычисляет для каждого `ProjectSummary`, включая:
  - `graph_node_id` (получается через `project_graph_node_id`)
  - `stats` (через `self.project_stats`)
  
- `project_detail(project_id: &str) -> Result<Option<ProjectDetail>, ProjectStoreError>`  
  Проверяет непустоту `project_id` через `validate_non_empty`, извлекает проект по идентификатору (`project_by_id`), а затем собирает `ProjectDetail`, содержащий:
  - `graph_node_id`
  - `stats`
  - `timeline` (ограничен `PROJECT_DETAIL_ITEM_LIMIT`)
  - `key_people` (ограничен `PROJECT_DETAIL_ITEM_LIMIT`)
  - `recent_messages` (ограничен `PROJECT_DETAIL_ITEM_LIMIT`)
  - `documents` (ограничен `PROJECT_DETAIL_ITEM_LIMIT`)

Константы лимитов:

- `DEFAULT_PROJECT_LIMIT` — значение по умолчанию для `list_projects`
- `MAX_PROJECT_LIMIT` — максимально допустимый лимит
- `PROJECT_DETAIL_ITEM_LIMIT` — ограничение на количество элементов в деталях проекта (люди, сообщения, документы, timeline)

#### Валидация (`validation.rs`)

- `validate_non_empty(field_name, value)` – обрезает пробелы и возвращает `Err(ProjectStoreError::EmptyField)`, если результат пуст.
- `validate_limit(limit)` – требует `limit > 0`, иначе `Err(ProjectStoreError::InvalidLimit)`, и ограничивает сверху `MAX_PROJECT_LIMIT`.

#### Read‑модели

##### `project_stats` (`stats.rs`)

Метод `ProjectStore::project_stats` вычисляет агрегированную статистику проекта:

1. Получает активные сообщения и документы через `active_project_messages` и `active_project_documents`, затем извлекает идентификаторы через `reviewed_target_ids`.
2. Выполняет четыре запроса:
   - **message_count** – количество сообщений в таблице `communication_messages` с идентификаторами из списка.
   - **document_count** – количество документов в таблице `documents` с идентификаторами из списка.
   - **people_count** – количество уникальных участников переписки (отправитель + получатели, нормализованные через `lower(trim(…))`).
   - **graph_connection_count** – количество рёбер графа (`graph_edges`), где `valid_to IS NULL` и `source_node_id = graph_node_id OR target_node_id = graph_node_id`. `graph_node_id` вычисляется через `project_graph_node_id(project_id)`.
   - **latest_activity_at** – максимальная дата активности из сообщений (`COALESCE(occurred_at, projected_at)`) и документов (`imported_at`).

Возвращает структуру `ProjectStats`.

##### `project_timeline` (`timeline.rs`)

Метод `ProjectStore::project_timeline` строит хронологию проекта:

- Лимит ограничивается через `TimelineEngine::bounded_entity_limit(limit)`.
- Объединяет (через `UNION ALL`) выборки:
  - сообщения: `item_kind = 'message'`, дата — `COALESCE(occurred_at, projected_at)`
  - документы: `item_kind = 'document'`, дата — `imported_at`
- Сортирует по `occurred_at DESC, item_kind, item_id`.
- Результаты маппятся через `row_to_timeline_item` в `Vec<ProjectTimelineItem>`.

#### Маппинг строк (`rows.rs`)

Функции преобразования `PgRow` → доменные структуры:

- `row_to_project` → `Project` (поля: `project_id`, `name`, `kind`, `status`, `description`, `owner_display_name`, `progress_percent`, `start_date`, `target_date`, `created_at`, `updated_at`).
- `row_to_project_message` → `ProjectMessageSummary` (`message_id`, `subject`, `sender`, `occurred_at`).
- `row_to_project_document` → `ProjectDocumentSummary` (`document_id`, `document_kind`, `title`, `observation_id`, `imported_at`).
- `row_to_project_person` → `ProjectPersonSummary` (`display_name`, `email_address`, `interaction_count`, `last_interaction_at`).
- `row_to_timeline_item` → `ProjectTimelineItem` (`item_kind`, `item_id`, `title`, `subtitle`, `occurred_at`).
- `row_to_matched_message` → `ProjectMatchedMessage` (включает десериализацию JSON‑массива `recipients` через `recipients_from_value`).
- `row_to_matched_document` → `ProjectMatchedDocument` (включает `source_fingerprint`).

Для `matched_*` сущностей `review_state` всегда инициализируется как `ProjectLinkReviewState::Suggested`.

---

### Подсистема link‑reviews (`backend/src/domains/projects/link_reviews`)

Реализует ручное подтверждение/отклонение связей сообщений и документов с проектами.

#### Модели (`models.rs`)

- `ProjectLinkTargetKind` — перечисление `Message` | `Document`. Строковые представления: `"message"`, `"document"`.
- `ProjectLinkReviewState` — перечисление `Suggested` | `UserConfirmed` | `UserRejected`. Строковые представления: `"suggested"`, `"user_confirmed"`, `"user_rejected"`.
- `ProjectLinkReviewCommand` — команда на изменение состояния ревью; поля: `command_id`, `project_id`, `target_kind`, `target_id`, `review_state`, `actor_id`.
- `ProjectLinkReviewCommandResult` — результат выполнения команды; поля: `project_id`, `target_kind`, `target_id`, `review_state`, `event_id`.
- `ProjectLinkReview` — полная запись ревью; поля: `project_id`, `target_kind`, `target_id`, `review_state`, `event_id`, `actor_id`, `reviewed_at`, `created_at`, `updated_at`.
- `ProjectReviewedTarget` — идентификатор цели и её текущее `review_state`.
- `ReviewEventApplication` (внутренняя) — данные для применения события ревью в транзакции.

#### Сервис (`service.rs`)

`ProjectLinkReviewService`:

- Хранит `pool: PgPool`.
- Метод `review_manual(command: &ProjectLinkReviewCommand) -> Result<ProjectLinkReviewCommandResult, ProjectLinkReviewServiceError>`:
  1. Захватывает наблюдение (observation) с типом `REVIEW_TRANSITION`, источником `Manual`, через `ObservationStore::capture`.
  2. Делегирует в `ProjectLinkReviewStore::set_review_state_with_observation`, передавая `observation_id` и контекстную метаинформацию.
- Ошибки: `ProjectLinkReviewServiceError` — варианты `Observation` и `ProjectLinkReview`.

#### Хранилище (`store.rs`)

`ProjectLinkReviewStore`:

**Основной метод изменения состояния:**

`set_review_state_with_observation`:
1. Валидирует `command_id`, `project_id`, `target_id`, `actor_id` через `validate_non_empty`.
2. Открывает транзакцию.
3. Проверяет существование проекта (`ensure_project_exists`) и цели (`ensure_target_exists`).
4. Формирует идентификатор события `project_link_review:{command_id}`.
5. Создаёт `NewEventEnvelope` через `command.to_review_event(&event_id)`.
6. Добавляет событие в Event Store через `EventStore::append_in_transaction`.
7. Применяет событие в этой же транзакции через `apply_review_event_in_transaction`.
8. Материализует связь review‑transition через `materialize_review_transition_link_in_transaction`.
9. Коммитит транзакцию.

**Метод `apply_review_event`** — обрабатывает входящий `EventEnvelope`:
- Извлекает `ReviewEvent` из `payload`.
- Проверяет `event_type == "project.link_review_state_changed"`.
- Извлекает `actor_id` из поля `actor` конверта.
- Проверяет существование проекта и цели.
- Применяет событие в транзакции.

**Метод `explicit_review`** — возвращает явную запись ревью по `project_id + target_kind + target_id`, если она существует.

**Запросы активных идентификаторов:**

- `active_message_ids_for_project` / `active_document_ids_for_project`:
  - Выбирает цели через keyword‑matching (ключевые слова проекта из `project_keywords` сравниваются с `subject`/`body_text` для сообщений или `title`/`extracted_text` для документов).
  - Добавляет подтверждённые (`user_confirmed`) связи из `project_link_reviews`.
  - Исключает цели, для которых есть запись с `user_rejected`.
  - Результат — список `ProjectReviewedTarget` с состоянием (`suggested` или `user_confirmed`).

**Адаптер применения событий (`adapters.rs`):**

`apply_review_event_in_transaction`:
- Для состояния `Suggested` — удаляет запись из `project_link_reviews`.
- Для `UserConfirmed` или `UserRejected` — выполняет `INSERT … ON CONFLICT … DO UPDATE`, обновляя `review_state`, `event_id`, `actor_id`, `reviewed_at`.

**Проверки целей (`target_checks.rs`):**

- `ensure_project_exists` — проверяет наличие `project_id` в таблице `projects`.
- `ensure_target_exists` — проверяет существование сообщения (`communication_messages`) или документа (`documents`) в зависимости от `target_kind`.

**Константы (`constants.rs`):**

- `PROJECT_LINK_REVIEW_EVENT_TYPE` = `"project.link_review_state_changed"`
- `PROJECT_LINK_REVIEW_SOURCE_KIND` = `"project_link_review"`
- `PROJECT_LINK_REVIEW_SOURCE_PROVIDER` = `"local_api"`

**Генерация событий (`events.rs`):**

- `ProjectLinkReviewCommand::to_review_event` создаёт `NewEventEnvelope` с типом `project.link_review_state_changed`, source‑информацией и payload, содержащим `project_id`, `target_kind`, `target_id`, `review_state`.
- `ReviewEvent::from_payload` парсит payload, извлекая обязательные строковые поля.

#### Ошибки (`errors.rs`)

`ProjectLinkReviewError` — варианты:

- `ProjectNotFound`, `TargetNotFound`
- `InvalidTargetKind`, `InvalidReviewState`
- `EmptyField`, `MissingPayloadField`, `InvalidPayload`
- `MissingActorId`, `InvalidEventType`
- Прозрачные делегаты: `EventEnvelope`, `Sqlx`, `EventStore`, `Observation`.

---

## Домен `relationships`

### Модели (`models.rs`)

- `RelationshipEntityKind` — перечисление из десяти элементов: `Persona`, `Organization`, `Project`, `Communication`, `Document`, `Task`, `Event`, `Decision`, `Obligation`, `Knowledge`. Каждый вариант имеет строковое представление (snake_case). Парсинг чувствителен к точному совпадению, иначе `RelationshipStoreError::UnknownEntityKind`.
- `RelationshipEvidenceSourceKind` — перечисление: `Observation`, `Communication`, `Document`, `Event`, `Memory`, `Knowledge`, `Decision`, `Obligation`, `Task`, `Project`, `Organization`, `Persona`.
- `RelationshipReviewState` — `Suggested`, `SystemAccepted`, `UserConfirmed`, `UserRejected`. Парсинг ожидает snake_case, иначе `RelationshipStoreError::UnknownReviewState`.
- `NewRelationship` — входная структура для создания связи; поля: `source_entity_kind`, `source_entity_id`, `target_entity_kind`, `target_entity_id`, `relationship_type`, `trust_score`, `strength_score`, `confidence`, `review_state`, `valid_from`, `valid_to`, `metadata`. Предоставляет удобный конструктор `between_personas` и метод `metadata`.
- `NewRelationshipEvidence` — свидетельство связи; поля: `source_kind`, `source_id`, `observation_id`, `excerpt`, `metadata`. Конструкторы: `new`, `observation`. Методы: `excerpt`, `metadata`.
- `Relationship` — полная модель связи, включает все поля `NewRelationship` плюс `relationship_id`, `created_at`, `updated_at`.

### Идентификаторы (`ids.rs`)

- `relationship_id` — детерминированная строка формата `relationship:v1:{len1}:{kind1}:{len2}:{id1}:{len3}:{type}:{len4}:{kind2}:{len5}:{id2}`. Включает длины строк для однозначного разбора.
- `evidence_id` — `relationship:evidence:v1:{len_rel_id}:{rel_id}:{len_src_kind}:{src_kind}:{len_src_id}:{src_id}`.

### Связывание с наблюдениями (`evidence.rs`)

Функция `link_relationship_entity_in_transaction` делегирует в `link_domain_entity_in_transaction` из `platform::observations`, передавая домен `"relationships"`.

### Маппинг строк (`row_mapping.rs`)

`row_to_relationship` преобразует `PgRow` в `Relationship`, вызывая `parse_entity_kind` и `parse_review_state` для текстовых полей.

### Ошибки (`errors.rs`)

`RelationshipStoreError` включает:

- `Sqlx`, `Observation`, `Graph` (как прозрачные делегаты)
- `EmptyField`, `InvalidJsonObject`, `InvalidScore`
- `MissingEvidence`, `InvalidObservationEvidenceSource`, `ObservationNotFound`
- `RelationshipNotFound`, `IdenticalEndpoints`, `InvalidTemporalRange`
- `UnknownEntityKind`, `UnknownEvidenceSourceKind`, `UnknownReviewState`

---

## Порты (`ports`)

В домене `projects`:
- `ProjectCommandPort` — реэкспорт `ProjectStore` из `projects::core`.

В домене `relationships`:
- `RelationshipReviewPort` — реэкспорт `RelationshipStore`.

```

## Source coverage / Покрытие источников

- `backend/src/domains/projects/core/read_model/stats.rs` — запросы `message_count`, `document_count`, `people_count`, `graph_connection_count`, `latest_activity_at`, использование `reviewed_target_ids` и `project_graph_node_id`.
- `backend/src/domains/projects/core/read_model/timeline.rs` — `project_timeline`, `UNION ALL` сообщений и документов, ограничение через `TimelineEngine::bounded_entity_limit`, сортировка.
- `backend/src/domains/projects/core/rows.rs` — функции `row_to_project`, `row_to_project_message`, `row_to_project_document`, `row_to_project_person`, `row_to_timeline_item`, `row_to_matched_message`, `row_to_matched_document`, десериализация `recipients`.
- `backend/src/domains/projects/core/store.rs` — `upsert_project` (транзакция, keywords), `list_projects` (лимиты, `ProjectSummary`), `project_detail` (сборка `ProjectDetail`), константы `DEFAULT_PROJECT_LIMIT`, `PROJECT_DETAIL_ITEM_LIMIT`.
- `backend/src/domains/projects/core/validation.rs` — `validate_non_empty`, `validate_limit`, `MAX_PROJECT_LIMIT`.
- `backend/src/domains/projects/link_reviews.rs` — публичный интерфейс модуля.
- `backend/src/domains/projects/link_reviews/adapters.rs` — `apply_review_event_in_transaction`: удаление для `Suggested`, upsert для `UserConfirmed`/`UserRejected`.
- `backend/src/domains/projects/link_reviews/constants.rs` — `PROJECT_LINK_REVIEW_EVENT_TYPE`, `PROJECT_LINK_REVIEW_SOURCE_KIND`, `PROJECT_LINK_REVIEW_SOURCE_PROVIDER`.
- `backend/src/domains/projects/link_reviews/errors.rs` — `ProjectLinkReviewError` и его варианты.
- `backend/src/domains/projects/link_reviews/events.rs` — `to_review_event` (создание `NewEventEnvelope` с JSON‑payload), `ReviewEvent::from_payload`.
- `backend/src/domains/projects/link_reviews/models.rs` — `ProjectLinkTargetKind`, `ProjectLinkReviewState`, `ProjectLinkReviewCommand`, `ProjectLinkReviewCommandResult`, `ProjectLinkReview`, `ProjectReviewedTarget`, `ReviewEventApplication`.
- `backend/src/domains/projects/link_reviews/rows.rs` — `row_to_project_link_review`, `row_to_project_reviewed_target`.
- `backend/src/domains/projects/link_reviews/service.rs` — `review_manual`: Observation capture + вызов `set_review_state_with_observation`.
- `backend/src/domains/projects/link_reviews/store.rs` — `set_review_state`, `set_review_state_with_observation`, `apply_review_event`, `explicit_review`, `active_message_ids_for_project` (keyword‑matched + review filter), `active_document_ids_for_project`.
- `backend/src/domains/projects/link_reviews/target_checks.rs` — `ensure_project_exists`, `ensure_target_exists`.
- `backend/src/domains/projects/link_reviews/validation.rs` — `validate_non_empty`.
- `backend/src/domains/projects/mod.rs` — объявление `pub mod core`, `pub mod link_reviews`, `pub mod ports`.
- `backend/src/domains/projects/ports.rs` — `ProjectCommandPort = ProjectStore`.
- `backend/src/domains/relationships/errors.rs` — `RelationshipStoreError` и все варианты.
- `backend/src/domains/relationships/evidence.rs` — `link_relationship_entity_in_transaction`.
- `backend/src/domains/relationships/ids.rs` — `relationship_id`, `evidence_id`.
- `backend/src/domains/relationships/mod.rs` — экспорты модуля.
- `backend/src/domains/relationships/models.rs` — `RelationshipEntityKind`, `RelationshipEvidenceSourceKind`, `RelationshipReviewState`, `NewRelationship` (включая `between_personas`), `NewRelationshipEvidence`, `Relationship`.
- `backend/src/domains/relationships/ports.rs` — `RelationshipReviewPort = RelationshipStore`.
- `backend/src/domains/relationships/row_mapping.rs` — `row_to_relationship`.

## Drift candidates / Кандидаты на drift

Из предоставленного контекста не видны расхождения (drift) между кодом и существующей документацией, поскольку страница `components/backend.md` либо отсутствует, либо её текущее содержимое не встроено в данный context pack. В пределах встроенных исходных файлов противоречий между объявленными структурами, методами и их реализацией не обнаружено.
