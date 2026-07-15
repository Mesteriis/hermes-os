---
chunk_id: 063-source-backend-part-043
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 15
generated_by: code-wiki-ru
---

# 063-source-backend-part-043 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `15`

## Резюме

Обновлена страница `components/backend.md` в русской wiki: на основе исходного кода `backend/src/integrations/telegram/runtime/` задокументирован компонент `TelegramRuntimeManager` — его структура, контекстные зависимости, жизненный цикл аккаунтов, модель команд и исполнение, обработка событий чатов и сообщений, загрузка медиа, работа с форумными топиками, а также вспомогательные модули.

## Предложенные страницы

#### `components/backend.md`

```markdown
# Backend компоненты

## Telegram Runtime Manager

### Обзор

`TelegramRuntimeManager` — центральный компонент интеграции Telegram. Он управляет жизненным циклом runtime-акторов (TDLib) для каждого аккаунта, синхронизацией чатов и сообщений, отправкой сообщений и обработкой событий от Telegram.

Определён в `backend/src/integrations/telegram/runtime/manager.rs`.

### Структура

```rust
#[derive(Clone, Default)]
pub struct TelegramRuntimeManager {
    actors: Arc<Mutex<HashMap<String, TelegramRuntimeActorHandle>>>,
}
```

`actors` — потокобезопасный словарь, отображающий идентификатор аккаунта (`String`) на дескриптор актора (`TelegramRuntimeActorHandle`), который содержит состояние актора и опциональный канал отправки команд.

### Публичный API

Менеджер предоставляет методы с суффиксом `_with_deps`, которые принимают внешние зависимости и запрос, и делегируют внутренней реализации через `TelegramRuntimeOperationContext`:

- `sync_chats_with_deps`
- `sync_history_with_deps`
- `send_manual_message_with_deps`
- `send_reply_message_with_deps`
- `send_forward_message_with_deps`
- `search_provider_messages_with_deps`
- `sync_forum_topics_with_deps`

### Контекстные структуры

Для разных операций используются специализированные контексты, объединяющие ссылки на порты и сервисы:

| Контекст | Поля |
|----------|------|
| `TelegramRuntimeOperationDeps` | `provider_account_store`, `provider_secret_binding_store`, `telegram_store`, `secret_store`, `secret_resolver`, `config`, `event_bridge` (опционально) |
| `TelegramRuntimeOperationContext` | Формируется из `Deps`, содержит те же поля |
| `TelegramMediaDownloadContext` | Те же поля, что и `Deps` |
| `TelegramMemberSyncContext` | Те же поля, что и `Deps` |
| `TelegramRuntimeStartContext` | `provider_account_store`, `provider_secret_binding_store`, `telegram_store`, `secret_store`, `secret_resolver`, `config`, `event_bus` (вместо `event_bridge`) |

Используемые порты:
- `ProviderAccountLookupPort` — поиск аккаунтов провайдера.
- `ProviderSecretBindingLookupPort` — поиск привязок секретов.
- `TelegramStore` — хранилище данных Telegram (чаты, сообщения).
- `SecretReferenceStore` — хранилище ссылок на секреты.
- `SecretResolver` — разрешение секретов.
- `AppConfig` — конфигурация приложения.
- `EventBus` / `TelegramRuntimeEventBridgeContext` — шина событий.

Функция `telegram_media_blob_root()` возвращает путь `DEFAULT_MAIL_SYNC_BLOB_ROOT` (из `crate::platform::communications`).

### Жизненный цикл аккаунтов

Методы управления аккаунтами определены в `lifecycle.rs`:

- `status_for_account` — возвращает `TelegramRuntimeStatus`, объединяя состояние аккаунта и состояние актора.
- `start_account`:
  1. Загружает и проверяет активный аккаунт (`load_active_account`).
  2. Опционально получает ключ шифрования сессии TDLib (`optional_telegram_session_key`).
  3. Определяет `runtime_kind` аккаунта (`account_runtime_kind`).
  4. В зависимости от `runtime_kind` создаёт состояние актора:
     - `"fixture"` → `Running` без канала команд.
     - `"tdlib_qr_authorized"` → порождает TDLib актора (`spawn_tdlib_actor`), при успехе получает канал команд (`Sender<TelegramRuntimeCommand>`); также запускает мост событий (`spawn_telegram_runtime_event_bridge`).
     - `"live_blocked"` → `Blocked`.
     - иное → `Error`.
  5. Сохраняет `TelegramRuntimeActorHandle` в `actors`.
  6. Возвращает `TelegramRuntimeStatus`.
- `stop_account_runtime` — останавливает runtime аккаунта.
- `restart_account_runtime` — останавливает и снова запускает.

### Взаимодействие с TDLib актором (команды)

Файл `commands.rs` содержит асинхронные функции `request_actor_*`, которые отправляют команды TDLib актору через `Sender<TelegramRuntimeCommand>`.

Каждая функция:
1. Вызывает `task::spawn_blocking`.
2. Создаёт `mpsc` канал для получения ответа.
3. Отправляет вариант перечисления `TelegramRuntimeCommand` (например, `LoadChats`, `SyncHistory`, `SendText`, `SendMedia`, `EditMessage`, `DeleteMessage`, `SetReaction`, `ReplyMessage`, `ForwardMessage`, `PinMessage` и др.).
4. Ожидает ответ с таймаутом `TDJSON_COMMAND_TIMEOUT` (для `SyncHistory` с режимом `Full` таймаут умножается на 10).
5. При ошибке возвращает `TelegramError::TdlibRuntime`.

Документированные функции (часть показана в усечённом файле, полный список см. `command_executor_dispatch.rs`):
- `request_actor_chats`
- `request_actor_chat_folders`
- `request_actor_history`
- `request_actor_send`
- `request_actor_send_media`
- `request_actor_download_file`
- `request_actor_edit_message`
- `request_actor_delete_message`
- `request_actor_set_reaction`
- `request_actor_reply`
- `request_actor_forward`
- `request_actor_pin_message`
- `request_actor_add_chat_to_folder` и др.

### Исполнитель команд (Command Executor)

Файл `command_executor.rs` содержит точку входа `execute_queued_commands`.

**Константы:**
- `RETRY_BASE_DELAY_SECONDS: 30`
- `RETRY_MAX_DELAY_SECONDS: 900` (15 минут)
- `STALE_EXECUTION_LOCK_SECONDS: 120`

**Алгоритм:**
1. Восстанавливает «зависшие» команды (locked дольше `STALE_EXECUTION_LOCK_SECONDS`) — для них генерируется событие со статусом `stale_recovery`.
2. Получает список активных аккаунтов (`runtime.active_account_ids()`).
3. Для каждого аккаунта:
   - Получает канал команд актора (`runtime.actor_command_tx`).
   - Забирает команды, ожидающие выполнения (`claim_due_commands_for_execution`).
   - Для каждой команды отправляет событие `executing`, а для media-команд дополнительно событие `media_upload_progress`.
   - Вызывает `dispatch_command`.
   - Обрабатывает результат через `handle_dispatch_result`.

**Обработка результата (`handle_dispatch_result`):**
- `Ok(ObservedMessage(snapshot))` → ингестирует snapshot сообщения в хранилище (`ingest_tdlib_message_snapshot`), публикует raw signal, помечает команду как `completed` (reconciled), генерирует события `command.completed` и `command.reconciled`. Для media-команд дополнительно событие `media_upload_completed`.
- `Ok(ObservedTopic(snapshot))` → upsert-ит топик форума (`upsert_topic_snapshot`), помечает команду как `completed`.
- `Ok(AwaitingProvider)` → команда остаётся в статусе `executing` с `reconciliation_status = awaiting_provider`.
- `Err(error)` → логика повторных попыток (retry) — детали не раскрыты в предоставленном контексте.

### Диспетчеризация команд

Файл `command_executor_dispatch.rs` реализует `dispatch_command`, которая по полю `command_kind` из `TelegramProviderWriteCommand` выбирает нужную функцию `request_actor_*` и возвращает `DispatchOutcome`.

Поддерживаемые `command_kind`:

| `command_kind` | Действие | `DispatchOutcome` |
|----------------|----------|-------------------|
| `send_text` | `request_actor_send` | `ObservedMessage` |
| `send_media` | `request_actor_send_media` | `ObservedMessage` |
| `reply` | `request_actor_reply` | `ObservedMessage` |
| `forward` | `request_actor_forward` | `ObservedMessage` |
| `edit` | `request_actor_edit_message` | `AwaitingProvider` |
| `delete` | `request_actor_delete_message` | `AwaitingProvider` |
| `react` / `unreact` | `request_actor_set_reaction` (is_active = true/false) | `AwaitingProvider` |
| `pin` / `unpin` | `request_actor_pin_message` | `AwaitingProvider` |
| `mark_read` / `mark_unread` | `request_actor_toggle_chat_unread` | `AwaitingProvider` |
| `archive` / `unarchive` | `request_actor_toggle_chat_archive` | `AwaitingProvider` |
| `mute` / `unmute` | `request_actor_toggle_chat_mute` | `AwaitingProvider` |
| `folder_add` | `request_actor_add_chat_to_folder` | `AwaitingProvider` |
| `folder_remove` | `request_actor_remove_chat_from_folder` | `AwaitingProvider` |
| `join` | `request_actor_join_chat` | `AwaitingProvider` |
| `leave` | `request_actor_leave_chat` | `AwaitingProvider` |
| `topic_create` | `request_actor_create_forum_topic` | `ObservedTopic` |
| `topic_close` / `topic_reopen` | `request_actor_toggle_forum_topic_closed` | `AwaitingProvider` |

Вспомогательные функции для извлечения данных из `payload`:
- `payload_string` — обязательная строка по ключу.
- `payload_optional_string` — опциональная строка.
- `payload_i64` — целое число `i64`.
- `provider_message_id` — извлекает `provider_message_id` из поля команды.

### Обработка событий чатов

Файлы `chat_event_payloads.rs` и `chat_events.rs` реализуют обработку событий от TDLib, связанных с чатами.

**Типы событий (event_type), используемые в конвертах:**
- `telegram_event_types::CHAT_UPDATED`
- `telegram_event_types::CHAT_MUTED`
- `telegram_event_types::CHAT_ARCHIVED`
- `telegram_event_types::CHAT_PINNED`
- `telegram_event_types::FOLDERS_UPDATED`

**Публикуемые функции:**
- `publish_chat_unread_event` — обновляет счётчики непрочитанных, публикует `CHAT_UPDATED`.
- `publish_chat_marked_as_unread_event` — обновляет состояние "marked as unread", публикует `CHAT_UPDATED`.
- `publish_chat_notification_settings_event` — обновляет настройки уведомлений, публикует `CHAT_UPDATED` и `CHAT_MUTED`.
- `publish_chat_position_event` — обновляет позицию чата (архив/папка/закрепление); публикует `CHAT_UPDATED`, `CHAT_PINNED` (если `list_kind` = `"main"` или `"archive"`) и `CHAT_ARCHIVED`. При `list_kind = "folder"` дополнительно обновляет фильтры групп чатов и публикует `FOLDERS_UPDATED`.
- `publish_chat_removed_from_list_event` — превращает удаление из списка в позиционное обновление.
- `publish_chat_folders_event` — обновляет метки папок чатов (`folder_labels`), публикует `CHAT_UPDATED` для каждого затронутого чата и событие `FOLDERS_UPDATED`.

Каждая функция вызывает внутренний `apply_chat_*_update` (детали выходят за пределы контекста), который проецирует изменения в хранилище, а затем согласовывает ожидающие команды (reconcile). Согласованные команды публикуются через `publish_command_reconciled_events`. События добавляются в `EventStore` и транслируются в `EventBus`.

**Тесты** (`chat_events/tests.rs`) проверяют:
- Порядок событий (например, `CHAT_UPDATED` перед `CHAT_MUTED`).
- Корректную генерацию `FOLDERS_UPDATED` при изменении папки.
- Проекцию `folder_labels` на чаты, включая fallback `"Unknown folder N"`.
- Согласование команд добавления/удаления в папку при позиционных событиях.

### Обработка событий сообщений

Файлы `message_events.rs` и `envelopes.rs` обрабатывают события сообщений.

**Типы событий:**
- `telegram_event_types::MESSAGE_CREATED`
- `telegram_event_types::MESSAGE_DELETED`
- `telegram_event_types::MESSAGE_UPDATED`
- `telegram_event_types::REACTION_CHANGED`

**Функции:**
- `publish_message_created_event` — ingests snapshot сообщения, публикует raw signal, генерирует `MESSAGE_CREATED`.
- `publish_message_deleted_event` — загружает сообщение, записывает tombstone, согласовывает команды удаления, публикует `MESSAGE_DELETED`.
- `publish_message_content_updated_event` — обновляет текст сообщения; если текст изменился, записывает edit observation; согласовывает команды редактирования; публикует `MESSAGE_UPDATED`.
- `publish_message_edited_event` — обновляет метаданные редактирования, публикует `MESSAGE_UPDATED`.
- `publish_message_pinned_event` — обновляет флаг `is_pinned`, согласовывает команды закрепления, публикует `MESSAGE_UPDATED`.

**Конверты событий (`envelopes.rs`):**
- `message_created_event` — конверт с типом `MESSAGE_CREATED`.
- `message_deleted_event` — конверт с типом `MESSAGE_DELETED`, содержит сообщение и `tombstone`.
- `message_updated_event` — конверт с типом `MESSAGE_UPDATED`, позволяет добавлять произвольные поля в `payload`.
- `reaction_changed_event` — конверт с типом `REACTION_CHANGED`, содержит `reaction_summary`.

Функция `append_and_broadcast` добавляет событие в `EventStore` (если передан `PgPool`) и транслирует в `EventBus`.

### Загрузка медиа

Файл `media_download.rs` содержит метод `download_media`.

В зависимости от `runtime_kind`:
- `"fixture"` → ошибка: требуется TDLib актор.
- `"tdlib_qr_authorized"` → активирует актор, вызывает `request_actor_download_file` (приоритет по умолчанию 16), возвращает `TelegramMediaDownloadResponse` с полями статуса (`"downloaded"`, `"downloading"`, `"remote"`), размерами и путём файла.
- `"live_blocked"` → ошибка блокировки.
- иное → ошибка неподдерживаемого runtime.

**События загрузки медиа** (`command_executor_media.rs`):
- `emit_media_upload_event` генерирует события с заданным типом (например, `MEDIA_UPLOAD_PROGRESS`, `MEDIA_UPLOAD_COMPLETED`), включая в `payload` идентификаторы команды, аккаунта, чата, `attachment_id`, `blob_id`, `media_type`, `caption_present`.
- `media_upload_progress_payload` формирует payload с фазой (`progress_phase`) и описанием (`progress_detail`), обогащая `provider_state`.

### Форумные топики

Файл `actor/topics.rs` содержит низкоуровневые функции работы с топиками через TDLib JSON-клиента:

- `actor_get_forum_topics(client, provider_chat_id, limit)` — запрашивает `getForumTopics`, парсит результат через `parse_tdlib_topic_list`.
- `actor_create_forum_topic(client, provider_chat_id, title, command_id)` — создаёт топик через `createForumTopic`, парсит через `parse_tdlib_created_forum_topic`.
- `actor_toggle_forum_topic_closed(client, provider_chat_id, provider_topic_id, is_closed, command_id)` — переключает `toggleForumTopicIsClosed`.

Все используют `TDJSON_COMMAND_TIMEOUT`, ошибки TDLib оборачиваются в `TelegramError::TdlibRuntime`.

В `command_executor_dispatch.rs` команды `topic_create` и `topic_close`/`topic_reopen` перенаправляются в эти функции. Высокоуровневый метод менеджера `sync_forum_topics_with_deps` выполняет синхронизацию топиков.

### Вспомогательные модули

- `account.rs` — `load_active_account`: загружает аккаунт и проверяет, что он активен.
- `actor_states.rs` — `running_actor_state`: создаёт состояние актора `Running`.
```

## Покрытие источников

| Source file | Covered facts |
|-------------|---------------|
| `backend/src/integrations/telegram/runtime/actor/topics.rs` | Функции `actor_get_forum_topics`, `actor_create_forum_topic`, `actor_toggle_forum_topic_closed`; использование `TDJSON_COMMAND_TIMEOUT`, `tdlib_provider_chat_id`, `receive_tdlib_extra`, парсинг топиков. |
| `backend/src/integrations/telegram/runtime/commands.rs` (truncated) | Синхронные обёртки `request_actor_*` (чаты, папки, история, отправка текста/медиа, загрузка, редактирование, удаление, реакции, ответ, пересылка, закрепление и др.); использование `task::spawn_blocking`, `mpsc`, `TDJSON_COMMAND_TIMEOUT` (10x для полной истории); ошибки `TelegramError::TdlibRuntime`. |
| `backend/src/integrations/telegram/runtime/manager.rs` | Структура `TelegramRuntimeManager` (`actors: Arc<Mutex<HashMap<...>>>`); публичные методы `sync_chats_with_deps`, `sync_history_with_deps`, `send_manual_message_with_deps`, `send_reply_message_with_deps`, `send_forward_message_with_deps`, `search_provider_messages_with_deps`, `sync_forum_topics_with_deps`; контекстные структуры (`TelegramRuntimeOperationDeps`, `TelegramRuntimeOperationContext`, `TelegramMediaDownloadContext`, `TelegramMemberSyncContext`, `TelegramRuntimeStartContext`) и их поля; функция `telegram_media_blob_root`. |
| `backend/src/integrations/telegram/runtime/manager/account.rs` | `load_active_account` — загрузка аккаунта и проверка активности. |
| `backend/src/integrations/telegram/runtime/manager/actor_states.rs` | `running_actor_state` — создание состояния актора `Running`. |
| `backend/src/integrations/telegram/runtime/manager/chat_event_payloads.rs` (truncated) | Сборщики событий: `chat_unread_updated_event`, `chat_marked_as_unread_updated_event`, `chat_notification_settings_updated_event`, `chat_notification_settings_chat_updated_event`, `chat_archived_updated_event`, `chat_position_updated_event`, `chat_folder_labels_updated_event`, `chat_pinned_updated_event`; используемые event types (`CHAT_UPDATED`, `CHAT_MUTED`, `CHAT_ARCHIVED`, `CHAT_PINNED`); структура конвертов. |
| `backend/src/integrations/telegram/runtime/manager/chat_events.rs` (truncated) | Public функции обработки событий чатов: `publish_chat_unread_event`, `publish_chat_marked_as_unread_event`, `publish_chat_notification_settings_event`, `publish_chat_position_event`, `publish_chat_removed_from_list_event`, `publish_chat_folders_event`; паттерн: project, reconcile, publish; вызов `publish_chat_group_filters_event`; использование `EventStore`, `EventBus`. |
| `backend/src/integrations/telegram/runtime/manager/chat_events/tests.rs` (truncated) | Тесты, проверяющие порядок событий, генерацию `FOLDERS_UPDATED`, проекцию `folder_labels`, согласование команд папок. |
| `backend/src/integrations/telegram/runtime/manager/command_executor.rs` (truncated) | `execute_queued_commands` — основной цикл: восстановление stale команд, итерация по аккаунтам, диспетчеризация, обработка результатов; константы `RETRY_BASE_DELAY_SECONDS`, `RETRY_MAX_DELAY_SECONDS`, `STALE_EXECUTION_LOCK_SECONDS`; `handle_dispatch_result` для `ObservedMessage`, `ObservedTopic`, `AwaitingProvider`; генерация командных и медийных событий. |
| `backend/src/integrations/telegram/runtime/manager/command_executor_dispatch.rs` | `dispatch_command` — отображение `command_kind` на действия и результат `DispatchOutcome`; вспомогательные `payload_string`, `payload_optional_string`, `payload_i64`, `provider_message_id`. |
| `backend/src/integrations/telegram/runtime/manager/command_executor_media.rs` | `emit_media_upload_event`, `media_upload_progress_payload`; структура событий загрузки медиа. |
| `backend/src/integrations/telegram/runtime/manager/lifecycle.rs` | Методы `status_for_account`, `start_account`, `stop_account_runtime`, `restart_account_runtime`; логика выбора `runtime_kind` и создания актора/состояния; использование `optional_telegram_session_key`. |
| `backend/src/integrations/telegram/runtime/manager/media_download.rs` | `download_media` — ветвление по `runtime_kind`, вызов `ensure_tdlib_actor`, `request_actor_download_file`; возврат `TelegramMediaDownloadResponse`. |
| `backend/src/integrations/telegram/runtime/manager/message_events.rs` (truncated) | `publish_message_created_event`, `publish_message_deleted_event`, `publish_message_content_updated_event`, `publish_message_edited_event`, `publish_message_pinned_event`; паттерны ingest, project, reconcile. |
| `backend/src/integrations/telegram/runtime/manager/message_events/envelopes.rs` | Конверты событий сообщений: `message_created_event`, `message_deleted_event`, `message_updated_event`, `reaction_changed_event`; `append_and_broadcast`. |

## Исходные файлы

- [`backend/src/integrations/telegram/runtime/actor/topics.rs`](../../../../backend/src/integrations/telegram/runtime/actor/topics.rs)
- [`backend/src/integrations/telegram/runtime/commands.rs`](../../../../backend/src/integrations/telegram/runtime/commands.rs)
- [`backend/src/integrations/telegram/runtime/manager.rs`](../../../../backend/src/integrations/telegram/runtime/manager.rs)
- [`backend/src/integrations/telegram/runtime/manager/account.rs`](../../../../backend/src/integrations/telegram/runtime/manager/account.rs)
- [`backend/src/integrations/telegram/runtime/manager/actor_states.rs`](../../../../backend/src/integrations/telegram/runtime/manager/actor_states.rs)
- [`backend/src/integrations/telegram/runtime/manager/chat_event_payloads.rs`](../../../../backend/src/integrations/telegram/runtime/manager/chat_event_payloads.rs)
- [`backend/src/integrations/telegram/runtime/manager/chat_events.rs`](../../../../backend/src/integrations/telegram/runtime/manager/chat_events.rs)
- [`backend/src/integrations/telegram/runtime/manager/chat_events/tests.rs`](../../../../backend/src/integrations/telegram/runtime/manager/chat_events/tests.rs)
- [`backend/src/integrations/telegram/runtime/manager/command_executor.rs`](../../../../backend/src/integrations/telegram/runtime/manager/command_executor.rs)
- [`backend/src/integrations/telegram/runtime/manager/command_executor_dispatch.rs`](../../../../backend/src/integrations/telegram/runtime/manager/command_executor_dispatch.rs)
- [`backend/src/integrations/telegram/runtime/manager/command_executor_media.rs`](../../../../backend/src/integrations/telegram/runtime/manager/command_executor_media.rs)
- [`backend/src/integrations/telegram/runtime/manager/lifecycle.rs`](../../../../backend/src/integrations/telegram/runtime/manager/lifecycle.rs)
- [`backend/src/integrations/telegram/runtime/manager/media_download.rs`](../../../../backend/src/integrations/telegram/runtime/manager/media_download.rs)
- [`backend/src/integrations/telegram/runtime/manager/message_events.rs`](../../../../backend/src/integrations/telegram/runtime/manager/message_events.rs)
- [`backend/src/integrations/telegram/runtime/manager/message_events/envelopes.rs`](../../../../backend/src/integrations/telegram/runtime/manager/message_events/envelopes.rs)

## Кандидаты на drift

Из представленного контекста расхождений между кодом, документацией и ADR не обнаружено.
