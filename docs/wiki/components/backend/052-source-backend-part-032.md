---
chunk_id: 052-source-backend-part-032
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 21
generated_by: code-wiki-ru
---

# 052-source-backend-part-032 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `21`

## Резюме

Страница `components/backend.md` должна быть обновлена, чтобы отразить фактическую структуру backend-доменов из исходного кода. В текущем контексте предоставлены файлы из доменов `signal_hub`, `relationships`, `review`, а также пустой модуль `settings`. Предлагается описать каждый домен: его основные сервисы, хранилища, модели данных, политики и обработку ошибок, опираясь исключительно на встроенный код. Также необходимо отметить, что домен `settings` присутствует, но его содержимое не раскрыто в данном контексте.

## Предложенные страницы

#### `components/backend.md`

```markdown
# Компоненты backend

Backend-часть системы реализует несколько независимых доменов, каждый из которых
отвечает за определённую бизнес-логику. Ниже описаны домены, исходный код которых
присутствует в контексте текущего чанка.

## Общий обзор

| Домен | Роль |
|-------|------|
| `signal_hub` | Управление источниками сигналов, соединениями, политиками, профилями, проверками здоровья и операционным контролем |
| `relationships` | Хранение и управление связями между сущностями с поддержкой доказательств (evidence) и ручного ревью |
| `review` | Инбокс элементов для ревью (кандидаты на сущности, связи, проекты и т.д.) с поддержкой переходов статусов и продвижения |
| `settings` | Модуль-заглушка; содержимое не раскрыто в данном контексте |

## Домен `signal_hub`

### Публичный интерфейс

Из модуля `backend/src/domains/signal_hub/mod.rs` экспортируются следующие ключевые
типы и сервисы:

- `SignalHubStore` (алиас `SignalHubPort`) – основное хранилище данных.
- `SignalHubSignalService` – сервис обработки сырых сигналов.
- `SignalProcessingOutcome` – результат обработки сигнала (`Accepted`, `Rejected`, `Muted`, `Paused`).
- `SignalHubControlService` – сервис управления операционным состоянием (disable, mute, pause источников).
- `SignalHubConnectionService` – сервис управления соединениями.
- `SignalHubCapabilityService` – сервис для работы с возможностями источников.
- `SignalHubHealthService` – сервис проверки здоровья источников.
- `SignalHubProfileService` – сервис управления профилями политик.
- `SignalPolicy`, `SignalPolicyMode`, `SignalPolicyScope`, `SignalPolicyEvaluator`, `SignalPolicyDecision` – система политик.
- `SignalSource`, `SignalConnection`, `SignalProfile`, `SignalHealth`, `SignalCapability` – модели данных.
- `SignalHubError` – перечисление ошибок домена.
- Вспомогательные функции для отправки сигналов из AI, mail, telegram, whatsapp.

### Хранилище (`store.rs`)

`SignalHubStore` (исходный файл частично обрезан) предоставляет методы для работы с
источниками (`list_sources`, `get_source`), соединениями (`create_connection`,
`update_connection`, `remove_connection`, `find_connection_by_account`),
профилями (`list_profiles`, `create_profile`, `update_profile`, `profile_by_code`,
`delete_profile`), политиками (`list_active_policies`, `create_policy`,
`expire_matching_policies`, `create_profile_managed_policy`,
`expire_managed_profile_policies`), проверками здоровья (`run_health_check`,
`upsert_health_snapshot`), capabilities (`list_capabilities`,
`replace_source_capabilities`). Полное описание ограничено обрезанием файла.

### Политики (`policies.rs`)

- `SignalPolicy` – структура политики с полями: `scope`, `source_code`, `connection_id`, `event_pattern`, `mode`, `reason`, `expires_at`.
- `SignalPolicyScope` – перечисление: `Global`, `Source`, `Connection`, `EventPattern`, `Profile`.
- `SignalPolicyMode` – перечисление: `Enabled`, `Disabled`, `Muted`, `Paused`, `ReplayOnly`, `FixtureOnly`.
- `SignalPolicyDecision` – результат оценки: `Allow`, `Rejected`, `Paused`, `Muted`.
- `SignalPolicyEvaluator` – вычислитель, который по набору политик и событию принимает решение `decide(&self, source_code, connection_id, event_type, policies)`. Приоритет: Disabled > Paused > Muted > Allow.
- Функция `event_type_matches` поддерживает точное совпадение или префиксный шаблон с суффиксом `.*`.

### Управление источниками (`controls.rs`, частично обрезан)

`SignalHubControlService` реализует методы:

- `disable_source`, `enable_source` – отключение/включение источника на уровне `SignalPolicyScope::Source`.
- `disable_signals`, `enable_signals`, `mute_signals`, `unmute_signals`, `pause_signals`, `resume_signals` – операции над политиками с произвольной областью (`SignalHubControlRequest`).
- Каждая операция приводит к созданию или очистке политик, а также синхронизирует runtime-состояние источника через `reconcile_source_runtime_state`.

Тип `SignalHubControlRequest` содержит:
- `scope: SignalPolicyScope`
- `source_code: Option<String>`
- `connection_id: Option<String>`
- `event_pattern: Option<String>`
- `reason: String`

Тип `SignalHubControlResult` возвращает `source_code`, `connection_id`, `event_pattern`, `policy_id` и `cleared_count`.

### Соединения (`connections.rs`)

`SignalHubConnectionService` управляет соединениями:

- `create_connection`, `update_connection`, `remove_connection` – CRUD с созданием событий `signal.connection.*`.
- `upsert_account_connection`, `remove_account_connection` – поиск по `source_code` и `account_id` для upsert/удаления.
- `reconcile_operator_status` – синхронизирует политики на основе статуса соединения: при статусе `disabled`/`paused`/`muted` применяется соответствующая политика на уровне `Connection`.
- Функция `connection_operator_mode` преобразует строковый статус соединения в `SignalPolicyMode`.

### Профили (`profiles.rs`)

`SignalHubProfileService` работает с профилями, хранящими набор политик:

- `list_profiles` – возвращает список с флагом `is_active`, определяемым по настройке `signal_hub.active_profile` в `ApplicationSettingsStore`.
- `create_profile`, `update_profile`, `remove_profile` – CRUD профилей; при удалении активного профиля сбрасывает настройку на `"production"`.
- `apply_profile` – делает профиль активным: очищает предыдущие managed-политики, создаёт политики из профиля, сохраняет код активного профиля в настройках.

### Возможности источников (`capabilities.rs`)

`SignalHubCapabilityService` при вызове `list_capabilities` пересчитывает и сохраняет возможности для каждого источника:

- Базовые: `signals.observe`, при наличии `supports_connections` – `connections.manage`, при `supports_runtime` – `runtime.health_check` и `runtime.pause`, при `supports_mute` – `runtime.mute`, при `supports_replay` – `runtime.replay`.
- Специфичные: для `browser` – `browser.capture`, `filesystem` – `files.observe`, `voice` – `voice.transcribe`, `fixture` – `fixture.emit`, `ai` – `ai.enrich`.
- Состояние возможности (`state`) вычисляется через `source_capability_control_state`, которая проверяет активные политики: если есть Disabled – `blocked`, Paused или Muted – `degraded`, иначе – `available`. В `reason` добавляется пояснение о текущем статусе.

### Здоровье (`health.rs`)

`SignalHubHealthService` обёртывает вызовы к `SignalHubStore` для `run_health_check` и `upsert_health_snapshot`, после каждого генерирует событие `signal.source.health_changed`.

### AI, Mail, Telegram, Whatsapp отправка

- `dispatch_ai_helper_signal` – строит сырой сигнал типа `signal.raw.ai.<event_kind>.observed`, отправляет через `EventStore`, при разрешении обработки запускает `process_raw_signal`.
- `dispatch_mail_raw_signal` и `dispatch_mail_delivery_event_signal` – аналогично для почтовых сигналов. Используют `StoredRawCommunicationRecord` и `MailDeliverySignalRequest`.
- `dispatch_telegram_raw_signal`, `dispatch_whatsapp_raw_signal` – экспортируются из соответствующих модулей.

### Фикстуры (`fixtures.rs`, `fixture_source.rs`)

- `fixtures.rs` определяет встроенные источники (`system_source_fixtures`), загружаемые из `fixtures/signal_hub/system_sources.toml`. Тест в модуле проверяет коды: `system`, `ai`, `mail`, `telegram`, `whatsapp`, `zoom`, `github`, `browser`, `rss`, `calendar`, `filesystem`, `home_assistant`, `voice`, `fixture`.
- Также определяет системные профили (`system_profile_fixtures`): `production` (без ограничений), `development` (muted `rss` и `browser`), `testing` (muted большинство реальных источников), `maintenance` (paused `mail`, `telegram`, `whatsapp`, `zoom`).
- `fixture_source.rs` – сервис `SignalFixtureSourceService` для эмиссии тестовых сигналов из TOML-каталога `test_signals.toml`. Поддерживает `emit_fixture` (требует существования источника) и `list_fixture_sources`. Валидация: schema_version == 1.

## Домен `relationships`

### Сервис (`service.rs`)

- `RelationshipCommandService` – единственная операция `review_manual`:
  принимает `relationship_id` и `RelationshipReviewState`, захватывает наблюдение
  типа `REVIEW_TRANSITION` с `ObservationOriginKind::Manual` и provenance
  `"captured_by": "relationships_service.review_manual"`.
  Затем вызывает `RelationshipStore::set_review_state_with_observation` с передачей идентификатора наблюдения.
- Ошибки: `RelationshipCommandServiceError` с вариантами `Observation` (из `ObservationStoreError`) и `Relationship` (из `RelationshipStoreError`).

### Хранилище (`store.rs`, частично обрезано)

- `RelationshipStore` предоставляет:
  - `upsert_with_evidence` – upsert связи вместе с доказательствами в транзакции; вызывает `validate_relationship_with_evidence`.
  - `upsert_with_evidence_in_transaction` – вставка в таблицу `relationships` с `ON CONFLICT (relationship_id) DO UPDATE`, вставка доказательств в `relationship_evidence` с `ON CONFLICT (relationship_id, source_kind, source_id) DO UPDATE`. Для каждого evidence с observation вызывается `link_relationship_entity_in_transaction`.
  - `list_for_entity` – выборка связей, где `source_entity_kind` и `source_entity_id` или `target_entity_kind` и `target_entity_id` совпадают; лимит ограничен `clamp(1, 100)`.
  - `list_by_review_state` – фильтр по `review_state`.
  - `set_review_state` / `set_review_state_with_observation` – обновление статуса ревью с материализацией графа и ссылки на переход ревью.
- Внутренние функции: `materialize_relationship_graph_in_transaction`, `materialize_relationship_graph_review_in_transaction` – частично обрезаны, детали неполны.

### Валидация (`validation.rs`)

- `validate_relationship_with_evidence` – проверяет связь и что evidence не пуст.
- `validate_relationship`:
  - `source_entity_id`, `target_entity_id`, `relationship_type` непустые.
  - `trust_score`, `strength_score`, `confidence` в диапазоне [0.0, 1.0].
  - `metadata` – JSON-объект.
  - Конечные точки не совпадают (`source_entity_id == target_entity_id` при равных `source_entity_kind` и `target_entity_kind` вызывает `IdenticalEndpoints`).
  - Если `valid_from` и `valid_to` заданы, то `valid_to >= valid_from`.
- `validate_evidence`:
  - `source_id` непустой.
  - Если `source_kind == Observation`, то `observation_id` должен совпадать с `source_id`.
  - `metadata` – JSON-объект.

## Домен `review`

### Модели (`models.rs`)

- `ReviewItemKind` – перечисление:
  `NewPerson`, `NewOrganization`, `IdentityCandidate`, `ProjectLinkCandidate`, `ContradictionCandidate`,
  `PotentialTask`, `PotentialObligation`, `PotentialDecision`, `PotentialRelationship`, `PotentialProject`, `KnowledgeCandidate`.
  Преобразование в/из строки через `as_str` / `parse`.
- `ReviewItemStatus` – перечисление:
  `New`, `InReview`, `Approved`, `Promoted`, `Dismissed`, `Archived`.
  Аналогично `as_str` / `parse`.
- `ReviewItem` – поля: `review_item_id`, `item_kind`, `title`, `summary`, `status`, `target_domain`, `target_entity_kind`, `target_entity_id`, `confidence`, `metadata`, `created_at`, `updated_at`.
- `NewReviewItem` – конструктор с `item_kind`, `title`, `summary`, `confidence`, `metadata`. Метод `validate` проверяет непустоту `title`/`summary`, диапазон `confidence` [0,1] и JSON-объектность `metadata`.
- `ReviewItemEvidence`, `NewReviewItemEvidence` – evidence с `observation_id`, `evidence_role`, `metadata`. `NewReviewItemEvidence::new` по умолчанию ставит роль `"primary"`. Валидация аналогична.
- `ReviewPromotionTarget` – целевые `target_domain`, `target_entity_kind`, `target_entity_id`. Валидация проверяет непустоту.
- Вспомогательные валидационные функции: `validate_review_item_with_evidence`, `validate_non_empty`, `validate_json_object`, `validate_score`.

### Сервис (`service.rs`)

- `ReviewInboxService` – операция `transition_status_from_manual`:
  создаёт observation `REVIEW_TRANSITION` с ручным происхождением, затем вызывает `ReviewInboxStore::set_status_with_observation`. Параметры: `captured_by` и `endpoint` (оба `&'static str`).
- Ошибки: `ReviewInboxServiceError` с вариантами:
  `StatusObservationCapture` (обёртка над `ObservationStoreError`),
  `PromotionObservationCapture` (аналогично),
  `ReviewInbox` (из `ReviewInboxError`).

### Хранилище (`store.rs`, частично обрезано)

- `ReviewInboxStore` предоставляет:
  - `create_with_evidence` – валидация (`validate_review_item_with_evidence`), транзакционное создание через `create_with_evidence_in_transaction`.
  - `list_by_status`, `list_open` (статусы `new`, `in_review`), `list_all` – запросы с лимитом `clamp(1, 100)`.
  - `set_status` / `set_status_with_observation` – изменение статуса с материализацией ссылки перехода.
  - `promote` / `promote_with_observation` – перевод в статус `promoted` с установкой `target_domain`, `target_entity_kind`, `target_entity_id`.
  - `get` – получение одного элемента.
  - `list_evidence` – получение evidence для элемента.
  - Внутренние транзакционные методы: `attach_evidence_in_transaction`, `transition_status_in_transaction`, `promote_in_transaction`, `find_latest_by_kind_and_metadata_in_transaction`.
- Генерация событий: при вставке нового элемента вызываются `append_candidate_detected_event` и `append_review_available_event`; при смене статуса/продвижении – `append_review_status_event`. Детали этих методов обрезаны.

### Ошибки (`errors.rs`)

- `ReviewInboxError` – перечисление вариантов:
  `Sqlx`, `Json`, `EventStore`, `EventEnvelope`, `ObservationStore`,
  `EmptyField(&'static str)`,
  `InvalidJsonObject(&'static str)`,
  `InvalidMetadataFilter`,
  `InvalidScore(&'static str, f64)`,
  `MissingEvidence`,
  `ObservationNotFound(String)`,
  `ReviewItemNotFound(String)`,
  `UnknownItemKind(String)`,
  `UnknownStatus(String)`.

### Evidence переходов (`evidence.rs`)

Реэкспортирует `materialize_review_transition_link` и `materialize_review_transition_link_in_transaction` из `crate::platform::observations`.

## Домен `settings`

Модуль `backend/src/domains/settings/mod.rs` существует, но в контексте содержит только
один символ (пустой файл). Подробности о назначении и содержимом домена не подтверждены.

---

> **Примечание:** Для доменов `signal_hub` и `review` часть исходных файлов была обрезана
> после 12000 символов. Описание основано исключительно на доступных частях.
> Полное поведение некоторых методов может быть не отражено.
```

## Покрытие источников

- **`backend/src/domains/signal_hub/mod.rs`**: перечень публичных экспортов, структура модулей signal_hub.
- **`backend/src/domains/signal_hub/ai.rs`**: функция `dispatch_ai_helper_signal`, построение события `signal.raw.ai.*.observed`.
- **`backend/src/domains/signal_hub/capabilities.rs`**: сервис `SignalHubCapabilityService`, `list_capabilities`, `refresh_source_capabilities`, базовые и специфичные возможности, логика `source_capability_control_state`.
- **`backend/src/domains/signal_hub/connections.rs`**: сервис `SignalHubConnectionService`, `create_connection`, `update_connection`, `remove_connection`, `upsert_account_connection`, `remove_account_connection`, `reconcile_operator_status`, `connection_operator_mode`.
- **`backend/src/domains/signal_hub/controls.rs`** (частично): `SignalHubControlService`, `SignalHubControlRequest`, `SignalHubControlResult`, операции `disable_source`, `enable_source`, `disable_signals`, `enable_signals`, `mute_signals`, `unmute_signals`, `pause_signals`, `resume_signals`, внутренние `create_scoped_policy`, `clear_scoped_policy`, `reconcile_source_runtime_state`.
- **`backend/src/domains/signal_hub/fixture_source.rs`**: `SignalFixtureSourceService`, `emit_fixture`, `list_fixture_sources`, загрузка из `test_signals.toml`, валидация `schema_version == 1`.
- **`backend/src/domains/signal_hub/fixtures.rs`**: встроенные источники `system_source_fixtures` из `system_sources.toml`, системные профили `development`, `testing`, `maintenance`, `production` с предопределёнными политиками.
- **`backend/src/domains/signal_hub/health.rs`**: `SignalHubHealthService`, `run_health_check`, `upsert_health_snapshot`, события `signal.source.health_changed`.
- **`backend/src/domains/signal_hub/mail.rs`**: `dispatch_mail_raw_signal`, `dispatch_mail_delivery_event_signal`, `MailDeliverySignalRequest`, построение событий с хешированием ID.
- **`backend/src/domains/signal_hub/policies.rs`**: `SignalPolicy`, `SignalPolicyScope`, `SignalPolicyMode`, `SignalPolicyDecision`, `SignalPolicyEvaluator` с `decide`, приоритеты, `event_type_matches`.
- **`backend/src/domains/signal_hub/profiles.rs`**: `SignalHubProfileService`, `list_profiles`, `create_profile`, `update_profile`, `remove_profile`, `apply_profile`, настройка `signal_hub.active_profile`.
- **`backend/src/domains/relationships/service.rs`**: `RelationshipCommandService`, `review_manual`, взаимодействие с `ObservationStore` и `RelationshipStore`.
- **`backend/src/domains/relationships/store.rs`** (частично): `RelationshipStore`, `upsert_with_evidence`, транзакционная вставка в `relationships` и `relationship_evidence`, `list_for_entity`, `list_by_review_state`, `set_review_state_with_observation`.
- **`backend/src/domains/relationships/validation.rs`**: `validate_relationship_with_evidence`, `validate_relationship`, `validate_evidence`, `validate_score`, проверки полей.
- **`backend/src/domains/review/mod.rs`**: публичные экспорты домена `review`.
- **`backend/src/domains/review/models.rs`**: `ReviewItemKind`, `ReviewItemStatus`, `ReviewItem`, `NewReviewItem`, `ReviewItemEvidence`, `NewReviewItemEvidence`, `ReviewPromotionTarget`, валидационные функции.
- **`backend/src/domains/review/service.rs`**: `ReviewInboxService`, `transition_status_from_manual`.
- **`backend/src/domains/review/store.rs`** (частично): `ReviewInboxStore`, `create_with_evidence`, `list_by_status`, `list_open`, `list_all`, `set_status_with_observation`, `promote_with_observation`, `get`, `list_evidence`.
- **`backend/src/domains/review/errors.rs`**: перечисление `ReviewInboxError`.
- **`backend/src/domains/review/evidence.rs`**: реэкспорт функций материализации ссылок переходов.
- **`backend/src/domains/settings/mod.rs`**: факт наличия пустого модуля.

## Исходные файлы

- [`backend/src/domains/relationships/service.rs`](../../../../backend/src/domains/relationships/service.rs)
- [`backend/src/domains/relationships/store.rs`](../../../../backend/src/domains/relationships/store.rs)
- [`backend/src/domains/relationships/validation.rs`](../../../../backend/src/domains/relationships/validation.rs)
- [`backend/src/domains/review/errors.rs`](../../../../backend/src/domains/review/errors.rs)
- [`backend/src/domains/review/evidence.rs`](../../../../backend/src/domains/review/evidence.rs)
- [`backend/src/domains/review/mod.rs`](../../../../backend/src/domains/review/mod.rs)
- [`backend/src/domains/review/models.rs`](../../../../backend/src/domains/review/models.rs)
- [`backend/src/domains/review/service.rs`](../../../../backend/src/domains/review/service.rs)
- [`backend/src/domains/review/store.rs`](../../../../backend/src/domains/review/store.rs)
- [`backend/src/domains/settings/mod.rs`](../../../../backend/src/domains/settings/mod.rs)
- [`backend/src/domains/signal_hub/ai.rs`](../../../../backend/src/domains/signal_hub/ai.rs)
- [`backend/src/domains/signal_hub/capabilities.rs`](../../../../backend/src/domains/signal_hub/capabilities.rs)
- [`backend/src/domains/signal_hub/connections.rs`](../../../../backend/src/domains/signal_hub/connections.rs)
- [`backend/src/domains/signal_hub/controls.rs`](../../../../backend/src/domains/signal_hub/controls.rs)
- [`backend/src/domains/signal_hub/fixture_source.rs`](../../../../backend/src/domains/signal_hub/fixture_source.rs)
- [`backend/src/domains/signal_hub/fixtures.rs`](../../../../backend/src/domains/signal_hub/fixtures.rs)
- [`backend/src/domains/signal_hub/health.rs`](../../../../backend/src/domains/signal_hub/health.rs)
- [`backend/src/domains/signal_hub/mail.rs`](../../../../backend/src/domains/signal_hub/mail.rs)
- [`backend/src/domains/signal_hub/mod.rs`](../../../../backend/src/domains/signal_hub/mod.rs)
- [`backend/src/domains/signal_hub/policies.rs`](../../../../backend/src/domains/signal_hub/policies.rs)
- [`backend/src/domains/signal_hub/profiles.rs`](../../../../backend/src/domains/signal_hub/profiles.rs)

## Кандидаты на drift

В предоставленном контексте видимых расхождений между кодом и документацией нет, так как:
- Существующее состояние wiki-страницы `components/backend.md` неизвестно (отсутствует в чанке).
- В исходных файлах нет упоминаний о какой-либо документации или ADR.
- Все утверждения в предложенной странице прямо подкреплены кодом.
- Нет противоречий между файлами в рамках одного домена.

Возможный скрытый дрейф: домен `settings` присутствует как модуль, но не имеет содержимого; если он должен быть документирован, это расхождение, но без дополнительного контекста нельзя утверждать.
