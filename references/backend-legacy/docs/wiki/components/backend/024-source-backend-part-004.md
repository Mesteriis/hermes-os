---
chunk_id: 024-source-backend-part-004
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 23
generated_by: code-wiki-ru
---

# 024-source-backend-part-004 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `23`

## Резюме

Обновить страницу `components/backend.md`, добавив описание модуля `backend/src/app/api_support` — слоя поддержки API, предоставляющего парсинг запросов, типовые структуры запросов/ответов для команд ревью, фабрики хранилищ, маршрутизацию AI-моделей и каталоги возможностей Telegram/WhatsApp. Текущее содержимое страницы не встроено, поэтому заменяем её целиком на основе предоставленных исходников.

## Предложенные страницы

### `components/backend.md`

```markdown
# Слой поддержки API (app/api_support)

Модуль `backend/src/app/api_support` содержит общие вспомогательные типы, функции и фабрики, используемые обработчиками API. Он не содержит конечных точек, а предоставляет строительные блоки: парсинг query-параметров, структуры команд ревью, инициализацию хранилищ и ответы с информацией о возможностях провайдеров.

## Парсинг запросов (query_parsing)

Набор функций и структур для разбора URL‑параметров, специфичных для доменных сущностей. Все парсеры используют `url::form_urlencoded`.

### Коммуникации (communication)

- **`CommunicationMessagesQuery`** — параметры фильтрации сообщений: `account_id`, `workflow_state`, `channel_kind`, `conversation_id`, `q` (поисковая строка), `local_state`, `cursor`, `limit`.
- Поле `q` дополнительно разбирается через `parse_communication_message_search_query`, которая заполняет `match_mode` (`MessageSearchMatchMode`) и `search` (`MessageSearchQuery`).
- Функция `parse_communication_messages_query(raw_query)` возвращает заполненную структуру или `ApiError`.
- Пустые значения игнорируются вспомогательной `non_empty_query_value`.

### Документы (documents)

- **`DocumentProcessingJobsQuery`** — только `limit` (целое число).
- `parse_document_processing_jobs_query` накладывает ограничение `clamp(1, 100)`.
- **Валидаторы**: `validate_non_empty_document_id`, `validate_non_empty_document_processing_field` — проверяют на непустую строку и возвращают понятные сообщения об ошибках.

### Граф (graph)

- Три запроса:
  - **`GraphNeighborhoodQuery`** (`node_id`, `depth` — только значение `1` разрешено для `u8`).
  - **`GraphNodesQuery`** (`limit`).
  - **`GraphSearchQuery`** (`q`, `limit`).
- Соответствующие функции `parse_graph_neighborhood_query`, `parse_graph_nodes_query`, `parse_graph_search_query`.

### Персоны (persons)

- **`PersonIdentityCandidatesQuery`** — `limit` с `clamp(1, 100)`.
- **`parse_person_identity_review_state`** — разбирает `"suggested"`, `"user_confirmed"`, `"user_rejected"` в `PersonIdentityReviewState`.
- **`validate_non_empty_person_identity_field`** — для полей `command_id`, `identity_candidate_id`, `person_id`.

### Проекты (projects)

- **`ProjectLinkCandidatesQuery`** — `limit` (тип `usize`, `clamp(1, 100)`).
- **`ProjectsQuery`** — `limit` (`i64`).
- **`parse_project_link_target_kind`** — `"message"` или `"document"`.
- **`parse_project_link_review_state`** — те же три состояния, что и для персон.
- **`validate_non_empty_project_link_field`** — для `command_id`, `project_id`, `target_id`.

### Задачи (tasks)

- **`TaskCandidatesQuery`** — `limit` с `clamp(1, 100)`.
- **`parse_task_candidate_review_state`** — `"suggested"`, `"user_confirmed"`, `"user_rejected"`.
- **`validate_non_empty_task_candidate_field`** — для `command_id`, `review_state`, `task_candidate_id`.

## Команды ревью (review_commands)

Типы запросов/ответов для операций, изменяющих состояние проверяемых сущностей. Каждый запрос реализует метод `into_command`, преобразующий его в доменную команду с валидацией полей.

### Персоны (person identity)

- **`PersonIdentityCandidateListResponse`** — список `PersonIdentityCandidate`.
- **`PersonIdentityReviewApiRequest`** — поля `command_id`, `review_state`. Конвертируется в `PersonIdentityReviewCommand`.
- **`PersonIdentityReviewApiResponse`** — `identity_candidate_id`, `review_state`, `event_id`. Реализовано `From<PersonIdentityReviewCommandResult>`.

### Документы (document processing)

- **`DocumentProcessingJobsResponse`** — список `DocumentProcessingJob`.
- **`DocumentProcessingRetryApiRequest`** — `command_id`. Конвертируется в `DocumentProcessingRetryCommand`.
- **`DocumentProcessingRetryApiResponse`** — `job_id`, `status`, `event_id`. Реализовано `From<DocumentProcessingRetryCommandResult>`.

### Задачи (task candidate)

- **`TaskCandidateReviewApiRequest`** — `command_id`, `review_state`. Конвертируется в `TaskCandidateReviewCommand`.
- **`TaskCandidateReviewApiResponse`** — `task_candidate_id`, `review_state`, `event_id`. Реализовано `From<TaskCandidateReviewCommandResult>`.

### Проекты (project link review)

- **`ProjectLinkReviewApiRequest`** — `command_id`, `target_kind`, `target_id`, `review_state`. Конвертируется в `ProjectLinkReviewCommand`.
- **`ProjectLinkReviewApiResponse`** — `project_id`, `target_kind`, `target_id`, `review_state`, `event_id`. Реализовано `From<ProjectLinkReviewCommandResult>`.

## Списки для ревью (review_lists)

Структуры для ответов API, возвращающих списки кандидатов.

- **`ProjectLinkCandidate`** — поля `project_id`, `target_kind`, `target_id`, `graph_node_id`, `title`, `subtitle`, `source_label`, `occurred_at`, `review_state`, `evidence_excerpt`.
- **`ProjectLinkCandidateListResponse`** — обёртка `Vec<ProjectLinkCandidate>`.
- **`TaskCandidateListResponse`** — обёртка `Vec<TaskCandidate>`.
- **`AiRunsQuery`** — `limit` (опционально).
- **`AiRunListResponse`** — обёртка `Vec<AiAgentRun>`.

## Хранилища (stores)

Модуль предоставляет функции‑фабрики для создания хранилищ и сервисов, принимая `&AppState`. Основная база данных извлекается через `database_pool`, которая возвращает `PgPool` или `ApiError::DatabaseNotConfigured`.

### Доменные хранилища (domain_stores)

Определён типаж **`AppStoreFactory`** с методом `from_pool`, реализованный через макрос `impl_app_store_factory!` для десятков хранилищ (события календаря, сообщения, организации, персоны, задачи, граф, документы и т.д.).

Специализированные конструкторы:
- `event_store`, `graph_store`, `message_store`, `observation_store`
- `project_store`, `project_link_review_store`
- `task_candidate_store`, `document_processing_store`
- `person_identity_store`
- `communication_blob_store` — создаёт `LocalCommunicationBlobStore` с корнем `DEFAULT_MAIL_SYNC_BLOB_ROOT`
- `api_audit_log`

### Интеграционные хранилища (integration_stores)

Функции для построения сервисов, специфичных для провайдеров:

- **Telegram**: `telegram_provider_runtime_service`, `telegram_secret_reference_store`, `telegram_runtime_use_case_context`, `telegram_message_write_service`, `telegram_fixture_ingest_service`.
- **WhatsApp**: `whatsapp_provider_runtime_service`, `whatsapp_secret_reference_store`, `whatsapp_fixture_ingest_service`.
- **Zoom**: `zoom_provider_runtime_service`, `zoom_secret_reference_store`.
- **Yandex Telemost**: `yandex_telemost_secret_reference_store`, `yandex_telemost_provider_runtime_store`, `yandex_telemost_provider_runtime_service`.
- **Email**: `account_setup_service` (с host‑vault и хранилищами аккаунтов/секретов).
- **Прочее**: `automation_store`, `call_intelligence_store`.

### Настройки и Vault (settings_vault)

- `settings_store` — создаёт `ApplicationSettingsStore`.
- `database_encrypted_vault` — строит `DatabaseEncryptedSecretVault` при наличии ключа из конфигурации (`config.secret_vault_key()`).

### AI‑маршрутизация и рантайм (ai_routing, ai_runtime)

- **`ai_model_routing`** — разрешает маршрутизацию моделей через `AiControlCenterStore`; при ошибке или отсутствии пула БД возвращает fallback на `settings.chat_model` / `settings.embedding_model`. Поддерживает слоты: `default_chat`, `reasoning`, `summarization`, `mail_intelligence`, `reply_draft`, `extraction`, `embeddings`, `meeting_prep`.
- Функция `resolve_ai_slot_model` проверяет готовность модели для приватного контекста и соответствие провайдера (`built_in/ollama` для `Ollama`, `api/omniroute` для `OmniRoute`).
- **`ai_runtime_client`** — создаёт `AiRuntimeClient` (Ollama или OmniRoute) на основе настроек и ключа API (из `config.omniroute_api_key()`).
- **`ai_requests_allowed`** — проверяет, разрешена ли обработка AI‑запросов, через `runtime_allows_processing` с идентификатором `"ai_request_runtime"`.
- **`ai_service`** — собирает полноценный `AiService` с маршрутизацией и портом атрибуции персон.
- **`email_multilingual_service`**, **`email_ai_reply_service`** — создают сервисы с опциональным AI‑портом.

## Каталог возможностей Telegram (telegram_capabilities)

Модель состояний и каталог операций Telegram согласно ADR‑0091 и ADR‑0052.

### Состояния и классы действий

- **`TelegramCapabilityState`**: `Available`, `Blocked`, `Degraded`, `Planned`, `Unsupported`.
- **`TelegramActionClass`**: `Read`, `LocalWrite`, `ProviderWrite`, `Destructive`, `Export`, `SecretAccess`, `Automation`.

### Запись операции

**`TelegramOperationCapability`** — поля: `operation`, `category`, `status`, `action_class`, `reason`, `confirmation_required`, `closure_gate`. Фабричный метод `new(...)`.

### Ответ API

**`TelegramCapabilitiesResponse`** содержит:
- `version` (строка `"2.1"`), `runtime_mode`, `account_scope` (опционально), флаги конфигурации (наличие `telegram_api_id` / `telegram_api_hash`, доступность `tdjson`, готовность QR‑логина, доступность Bot API).
- `capabilities` — вектор `TelegramOperationCapability`, собирается из трёх каталогов: foundation, messages, extended.
- `planned_features`, `unsupported_features` — статические списки.
- Логика `apply_account_scope_overrides` корректирует статусы и причины для конкретного аккаунта в зависимости от `provider_kind`, `lifecycle_state`, `runtime_kind` (например, боты получают `Unsupported` для операций QR‑авторизации и TDLib‑рантайма).

### Каталог foundation

Включает секции:
- **account**: создание/удаление/логаут, список.
- **runtime**: fixture, tdlib_live, bot_live, статус, стоп/рестарт, здоровье.
- **authorization**: QR‑логин (start/status/password/cancel), зависимость от `qr_ready`.
- **session**: импорт/экспорт (Planned), прокси (Planned).
- **sync**: синхронизация чатов и истории (latest/older/full), статус зависит от `qr_ready`.
- **dialogs**: список, pin, archive, mute, unread_counters, mark_read/unread, управление папками.

### Каталог messages

Группы операций:
- **messages:read**: list, get_versions, get_raw_evidence (всегда Available).
- **messages:write**: send_text, send_media, edit, delete, restore_visibility, mark_read — статус зависит от `qr_ready`.
- **replies_forwards**: reply, forward, pin.
- **reactions**: add, remove (зависят от `qr_ready`), sync (Available).
- **participants**: sync, join, leave.
- **topics**: list, create, close.

Статусы по умолчанию: `Available` при `qr_ready==true`, иначе `Unsupported`/`Blocked`.

### Каталог extended

- **dialogs**: mark_unread (аналогично mark_read).
- **media**: download, upload_send, gallery, preview.
- **voice/calls**: playback, record_send, record, send, video.record, calls.metadata (Available), calls.live_control (Planned), calls.transcription_live (Blocked).
- **search**: local_messages, local_dialogs (Available), provider и media (зависят от `qr_ready`).
- **realtime**: события message_created/updated/deleted, reaction_changed, sync_progress (Available).
- **automation**: dry_run (Available), live_send (Blocked).
- **ai**: summary, translation, bilingual_reply, review_flows (Planned).
- **export**: chat, markdown (Unsupported).

### Тесты

Модуль `telegram_capability_catalog_tests` проверяет:
- Запланированные инициативы имеют статус `"planned"`.
- При `qr_ready == true` диалоговые операции имеют `available` и `provider_write`.
- Поиск и медиа‑операции имеют `available` и `read`.

## Каталог возможностей WhatsApp (whatsapp_capabilities)

Аналогичная Telegram модель для WhatsApp, с учётом форм‑факторов провайдера (provider shape): `WebCompanion`, `NativeMultiDevice`, `BusinessCloud`.

- **`WhatsAppCapabilityState`**: `Available`, `Blocked`, `Degraded`, `Planned`, `Unsupported`.
- **`WhatsAppActionClass`**: `Read`, `LocalWrite`, `ProviderWrite`, `Destructive`, `SecretAccess`.
- **`WhatsappProviderShapeStatus`** — статус конкретного форм‑фактора.
- **`WhatsappCapabilityAccountScope`** — информация о текущем аккаунте (флаги `live_runtime_available`, `live_send_available`, `media_download_available`, `media_upload_available`).
- **`WhatsappCapabilitiesResponse`** — версия `"2.0"`, `provider_shapes` (сводка по каждому), `capabilities` (из `whatsapp_capability_rows`), `planned_features`, `unsupported_features`. Применяет `apply_account_scope_overrides` для корректировки статусов в зависимости от `provider_shape` и `lifecycle_state` (например, Business Cloud исключает личные чаты и QR‑линковку).

*Детальный перечень операций каталога WhatsApp не раскрыт в предоставленных исходниках (файл `whatsapp_capabilities.rs` обрезан), однако видно наличие вспомогательных функций `is_whatsapp_business_cloud_personal_capability`, `is_whatsapp_provider_write_capability` и др.*

## Примечание о контексте

Ряд файлов был обрезан при встраивании (лимит 12 000 символов), поэтому некоторые детали могут быть не задокументированы:
- `telegram_capabilities.rs` — окончание логики `apply_account_scope_overrides` не видно.
- `telegram_capability_catalog_foundation.rs` — обрезан до окончания `push_dialog_capabilities`.
- `whatsapp_capabilities.rs` — обрезан, полный список операций недоступен.
Документированы только те факты, которые присутствуют в предоставленном фрагменте.
```

## Покрытие источников

- **`backend/src/app/api_support/query_parsing.rs`** — структура подмодулей.
- **`backend/src/app/api_support/query_parsing/communication.rs`** — `CommunicationMessagesQuery`, парсинг, `MessageSearchMatchMode`.
- **`backend/src/app/api_support/query_parsing/documents.rs`** — `DocumentProcessingJobsQuery`, валидаторы.
- **`backend/src/app/api_support/query_parsing/graph.rs`** — `GraphNeighborhoodQuery`, `GraphNodesQuery`, `GraphSearchQuery`.
- **`backend/src/app/api_support/query_parsing/persons.rs`** — `PersonIdentityCandidatesQuery`, `parse_person_identity_review_state`, валидатор.
- **`backend/src/app/api_support/query_parsing/projects.rs`** — `ProjectLinkCandidatesQuery`, `ProjectsQuery`, парсеры состояний и видов целей.
- **`backend/src/app/api_support/query_parsing/tasks.rs`** — `TaskCandidatesQuery`, `parse_task_candidate_review_state`, валидатор.
- **`backend/src/app/api_support/review_commands.rs`** — структуры запросов/ответов для ревью персон, документов, задач, проектов; реализация `into_command` и `From`.
- **`backend/src/app/api_support/review_lists.rs`** — `ProjectLinkCandidate`, `TaskCandidateListResponse`, `AiRunListResponse`.
- **`backend/src/app/api_support/stores.rs`** — состав подмодулей.
- **`backend/src/app/api_support/stores/ai_routing.rs`** — `ai_model_routing`, разрешение слотов моделей.
- **`backend/src/app/api_support/stores/ai_runtime.rs`** — функции `ai_service`, `ai_runtime_client`, `ai_requests_allowed`, `email_multilingual_service` и др.
- **`backend/src/app/api_support/stores/database.rs`** — `database_pool`.
- **`backend/src/app/api_support/stores/domain_stores.rs`** — `AppStoreFactory`, макрос, конструкторы хранилищ.
- **`backend/src/app/api_support/stores/integration_stores.rs`** — фабрики Telegram, WhatsApp, Zoom, Yandex Telemost, email, automation, call intelligence.
- **`backend/src/app/api_support/stores/settings_vault.rs`** — `settings_store`, `database_encrypted_vault`.
- **`backend/src/app/api_support/telegram_capabilities.rs`** — модель состояний, `TelegramCapabilitiesResponse`, переопределения для аккаунтов.
- **`backend/src/app/api_support/telegram_capability_catalog.rs`** — сборка каталога из трёх частей.
- **`backend/src/app/api_support/telegram_capability_catalog_extended.rs`** — расширенные возможности Telegram.
- **`backend/src/app/api_support/telegram_capability_catalog_foundation.rs`** — базовые возможности Telegram.
- **`backend/src/app/api_support/telegram_capability_catalog_messages.rs`** — возможности, связанные с сообщениями.
- **`backend/src/app/api_support/telegram_capability_catalog_tests.rs`** — тесты состояний возможностей.
- **`backend/src/app/api_support/whatsapp_capabilities.rs`** — модель состояний WhatsApp, `WhatsappCapabilitiesResponse` (частично обрезана).

## Исходные файлы

- [`backend/src/app/api_support/query_parsing.rs`](../../../../backend/src/app/api_support/query_parsing.rs)
- [`backend/src/app/api_support/query_parsing/communication.rs`](../../../../backend/src/app/api_support/query_parsing/communication.rs)
- [`backend/src/app/api_support/query_parsing/documents.rs`](../../../../backend/src/app/api_support/query_parsing/documents.rs)
- [`backend/src/app/api_support/query_parsing/graph.rs`](../../../../backend/src/app/api_support/query_parsing/graph.rs)
- [`backend/src/app/api_support/query_parsing/persons.rs`](../../../../backend/src/app/api_support/query_parsing/persons.rs)
- [`backend/src/app/api_support/query_parsing/projects.rs`](../../../../backend/src/app/api_support/query_parsing/projects.rs)
- [`backend/src/app/api_support/query_parsing/tasks.rs`](../../../../backend/src/app/api_support/query_parsing/tasks.rs)
- [`backend/src/app/api_support/review_commands.rs`](../../../../backend/src/app/api_support/review_commands.rs)
- [`backend/src/app/api_support/review_lists.rs`](../../../../backend/src/app/api_support/review_lists.rs)
- [`backend/src/app/api_support/stores.rs`](../../../../backend/src/app/api_support/stores.rs)
- [`backend/src/app/api_support/stores/ai_routing.rs`](../../../../backend/src/app/api_support/stores/ai_routing.rs)
- [`backend/src/app/api_support/stores/ai_runtime.rs`](../../../../backend/src/app/api_support/stores/ai_runtime.rs)
- [`backend/src/app/api_support/stores/database.rs`](../../../../backend/src/app/api_support/stores/database.rs)
- [`backend/src/app/api_support/stores/domain_stores.rs`](../../../../backend/src/app/api_support/stores/domain_stores.rs)
- [`backend/src/app/api_support/stores/integration_stores.rs`](../../../../backend/src/app/api_support/stores/integration_stores.rs)
- [`backend/src/app/api_support/stores/settings_vault.rs`](../../../../backend/src/app/api_support/stores/settings_vault.rs)
- [`backend/src/app/api_support/telegram_capabilities.rs`](../../../../backend/src/app/api_support/telegram_capabilities.rs)
- [`backend/src/app/api_support/telegram_capability_catalog.rs`](../../../../backend/src/app/api_support/telegram_capability_catalog.rs)
- [`backend/src/app/api_support/telegram_capability_catalog_extended.rs`](../../../../backend/src/app/api_support/telegram_capability_catalog_extended.rs)
- [`backend/src/app/api_support/telegram_capability_catalog_foundation.rs`](../../../../backend/src/app/api_support/telegram_capability_catalog_foundation.rs)
- [`backend/src/app/api_support/telegram_capability_catalog_messages.rs`](../../../../backend/src/app/api_support/telegram_capability_catalog_messages.rs)
- [`backend/src/app/api_support/telegram_capability_catalog_tests.rs`](../../../../backend/src/app/api_support/telegram_capability_catalog_tests.rs)
- [`backend/src/app/api_support/whatsapp_capabilities.rs`](../../../../backend/src/app/api_support/whatsapp_capabilities.rs)

## Кандидаты на drift

Из предоставленного контекста расхождений между кодом и документацией не видно. Часть файлов обрезана, но это не является дрифтом — полное покрытие невозможно без доступа к полным исходникам. Явных противоречий между встроенными фрагментами и существующей wiki (содержимое которой не предоставлено) не обнаружено, так как текущая wiki не входит в контекст.
