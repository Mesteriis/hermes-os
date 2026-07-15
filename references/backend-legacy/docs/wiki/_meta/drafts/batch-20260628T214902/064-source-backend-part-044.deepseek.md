"# Summary / Резюме

Данный чанк добавляет в русскую wiki подробное описание компонента backend — менеджера runtime для Telegram-интеграции. Страница `components/backend.md` пополняется фактами, извлечёнными непосредственно из исходного кода модуля `manager` (и связанных файлов): архитектура реально-временной обработки событий, проекции сообщений, синхронизация участников, отправка команд, поиск, синхронизация чатов и истории, управление актором TDLib, а также модели запросов/ответов. Все утверждения подтверждены встроенными исходниками; внешние предположения не добавляются.

# Proposed pages / Предлагаемые страницы

## `components/backend.md`

```markdown
# Менеджер Telegram Runtime (backend)

Назначение: управление жизненным циклом Telegram-аккаунтов, обработка реально‑временных событий от TDLib, проекция данных сообщений и участников, выполнение команд отправки, синхронизации чатов/истории и поиска.

## Архитектура

- **TelegramRuntimeManager** — центральный компонент, предоставляющий методы для синхронизации, отправки, поиска и управления акторами.
- **TelegramRuntimeEventBridgeContext** — контекст, содержащий опциональную ссылку на `TelegramStore` и `EventBus`. Используется для публикации событий внутри runtime-моста.
- **Очередь runtime-событий** — канал `UnboundedReceiver<TelegramRuntimeEvent>`, через который actor (TDLib) передаёт события в мост.
- **Привязка к аккаунту** — каждый runtime-актор идентифицируется по `account_id`, акторы хранятся в `Mutex<HashMap<String, TelegramRuntimeActorHandle>>`.

## Компоненты

### Реестр акторов (`registry`)

`TelegramRuntimeManager` предоставляет:
- `actor_state` — получение состояния актора по `account_id`.
- `stop_account` — остановка актора (удаление из реестра).
- `set_actor_handle` — сохранение дескриптора актора.
- `actor_command_tx` — получение отправителя команд для актора.
- `active_account_ids` — список идентификаторов активных аккаунтов, у которых есть канал команд.

Все методы блокируют мьютекс `actors`.

### Мост реально‑временных событий (`realtime_events`)

- `spawn_telegram_runtime_event_bridge` запускает асинхронную задачу tokio, читающую события из `UnboundedReceiver<TelegramRuntimeEvent>` и вызывающую соответствующие обработчики.
- Обрабатываются следующие варианты события:
  - `MessageCreated` → `publish_message_created_event` (тест подтверждает, что публикуется `signal.raw.telegram.message.observed`, а не устаревший `telegram.message.created`)
  - `MessageContentUpdated` → `publish_message_content_updated_event`
  - `MessageEdited` → `publish_message_edited_event`
  - `MessagePinnedUpdated` → `publish_message_pinned_event`
  - `MessageDeleted` → `publish_message_deleted_event`
  - `MessageInteractionInfoUpdated` → `publish_reaction_changed_event`
  - `TypingChanged` → `publish_typing_event`
  - `TopicUpdated` → `publish_topic_event`
  - `ChatUnreadUpdated` → `publish_chat_unread_event`
  - `ChatMarkedAsUnreadUpdated` → `publish_chat_marked_as_unread_event`
  - `ChatNotificationSettingsUpdated` → `publish_chat_notification_settings_event`
  - `ChatPositionUpdated` → `publish_chat_position_event`
  - `ChatRemovedFromList` → `publish_chat_removed_from_list_event`
  - `ChatFoldersUpdated` → `publish_chat_folders_event`
- Перед обработкой проверяется разрешение через `telegram_runtime_event_bridge_allows_processing`, которая вызывает `runtime_allows_processing` с меткой `"telegram"` и runtime `"telegram_runtime_event_bridge"`.

#### Публикация событий сверки команд (`publish_command_reconciled_events`)

- Функция создаёт два события для команды:
  - `telegram.command.status_changed` (тип `COMMAND_STATUS_CHANGED`)
  - `telegram.command.reconciled` (тип `COMMAND_RECONCILED`)
- События включают payload с полями: `command_id`, `account_id`, `command_kind`, `provider_chat_id`, `status`, `reconciliation_status`, `provider_observed_at`, `reconciled_at`, `provider_state` и др.
- Функция `command_event_payload` формирует полезную нагрузку, объединяя статичные поля команды с дополнительным payload.

### Проекция сообщений (`projection`)

- `update_message_reaction_summary` — извлекает из сырого JSON (`raw`) сводку реакций через `derive_tdlib_reaction_summary_metadata` и записывает её в `metadata.reaction_summary` (либо удаляет ключ, если сводка отсутствует). Изменения сохраняются через `append_message_metadata_observation`.
- `project_provider_message_content_observation` — записывает в метаданные `text`, `tdlib_content` и `last_provider_content_update_source` из `TelegramTdlibMessageContentSnapshot`. Вызывает `append_message_content_observation`.
- `project_provider_message_edit_observation` — записывает в метаданные `provider_edit_timestamp`, `last_provider_edit_source` и опционально `tdlib_reply_markup`. Сохраняет через `append_message_metadata_observation`.
- `observed_edit_timestamp` — читает `provider_edit_timestamp` из метаданных, парсит как RFC 3339 и возвращает в UTC; при отсутствии возвращает переданный `fallback`.

### Синхронизация участников (`participants`)

- `sync_chat_members` — главная точка входа для синхронизации участников чата.
- Константа `TELEGRAM_MEMBER_SYNC_TARGET_LIMIT` = 500 задаёт лимит запрашиваемых участников.
- Логика различает типы чатов:
  - **Приватный чат** (`sync_private_chat_members`): участники извлекаются из метаданных чата (`tdlib_private_user_id`). Для «Избранного» (`is_saved_messages == true`) используется `telegram_self_provider_member_id`; иначе строится участник `user:{id}`. События публикуются с источником `"tdlib.chat.metadata"`.
  - **Базовые группы** (`tdlib_basic_group_id` не None): вызывается `request_actor_get_basic_group_members`, затем `sync_provider_roster_snapshots` с меткой `"tdlib.getBasicGroupFullInfo"`.
  - **Супергруппы** (`tdlib_supergroup_id` не None): сначала запрашиваются участники (`request_actor_get_supergroup_members`), затем администраторы (`request_actor_get_supergroup_administrators`). Снимки сливаются (`merge_supergroup_member_snapshots`) и передаются в `sync_provider_roster_snapshots`.
- `sync_provider_roster_snapshots` для каждого снимка выполняет `upsert_chat_participant` и публикует `publish_participant_updated_event`. Если реестр исчерпывающий (`roster_is_exhaustive = true`), отсутствующие участники помечаются через `mark_absent_members_from_exhaustive_roster`, для них также публикуются события с суффиксом `".exhaustive_absence"`. Затем вызывается `reconcile_self_membership_from_provider_roster` для сверки команд join/leave.
- Идентификатор участника (`telegram_participant_id`) вычисляется как SHA-256 от `telegram_chat_id\0provider_member_id` с префиксом `telegram_participant:v1:`.
- Приватный участник строится с `role = "member"`, `status = "member"`, `is_admin = false`, `is_owner = false`; в `permissions` записывается мета-информация о типе чата.

#### События участников (`participant_events`)

- `publish_participant_updated_event` публикует событие `telegram.participant.updated` через `EventBus` и сохраняет в `EventStore`, если контекст содержит `telegram_store`.
- Событие содержит subject `telegram_chat_participant`, payload с полным объектом `TelegramChatMember`, а provenance с полями `provider`, `runtime`, `tdlib_event` (очищенный префикс `tdlib.`).

### Отправка сообщений (`send`)

- `send_manual_message` — ручная отправка. Поддерживает runtime-ы: `"fixture"` (вызывает `manual_send_message` напрямую через хранилище), `"tdlib_qr_authorized"` (через `request_actor_send`), `"live_blocked"` (ошибка). Результат ингестируется через `ingest_tdlib_message_snapshot`, вычисляется `rendered_preview_hash` по тексту.
- `send_reply_message` — ответ на сообщение. Работает только через TDLib (`request_actor_reply`); fixture-режим возвращает ошибку. Ингестирует результат аналогично.
- `send_forward_message` — пересылка. Только TDLib (`request_actor_forward`); fixture-режим возвращает ошибку.
- Во всех случаях возвращается `TelegramManualSendResponse` с полями `raw`, `raw_record_id`, `message_id`, `delivery_state`, `status: "sent"`, `runtime_kind`, `rendered_preview_hash`.

### Синхронизация чатов (`sync_chats`)

- `sync_chats` принимает `TelegramChatSyncRequest` (account_id, limit) и возвращает `TelegramChatSyncResponse`.
- При `runtime_kind == "fixture"` просто возвращает список чатов из хранилища.
- При `"tdlib_qr_authorized"`:
  1. Запрашивает снимки чатов через `request_actor_chats`.
  2. Ингестирует каждый снимок через `ingest_tdlib_chat_snapshot`.
  3. Извлекает идентификаторы папок чатов из `positions` в raw-снимке (тип `"chatListFolder"`), запрашивает папки через `request_actor_chat_folders` и применяет через `apply_provider_chat_folders`.
  4. Возвращает список чатов из хранилища.
- `"live_blocked"` вызывает ошибку; неподдерживаемые runtime-ы — ошибку.

### Синхронизация истории (`sync_history`, `sync_history_tdlib`)

- `sync_history` диспетчеризует:
  - `fixture` → `sync_fixture_history` (возвращает `recent_messages`).
  - `tdlib_qr_authorized` → `sync_tdlib_history` через `TdlibHistorySyncContext`.
- `sync_tdlib_history`:
  1. При режиме `Full` проверяет, что чат — приватный (`ensure_private_chat_for_full_sync`); для групп/каналов полная синхронизация запрещена с сообщением об ошибке.
  2. Запрашивает историю через `request_actor_history` с параметрами `provider_chat_id`, `from_message_id`, `limit`, `mode`.
  3. Для каждого снимка выполняет `ingest_tdlib_message_snapshot`, публикует сигнал через `publish_observed_message_raw_signal`, сверяет команды участия (через `tdlib_self_membership_lifecycle` и `reconcile_participant_commands_from_message_evidence`) и команды реакций (через `derive_tdlib_chosen_reaction_emojis` и `reconcile_reaction_commands_from_provider_reactions`).
  4. Возвращает `TelegramHistorySyncResponse` с флагами `has_more` (истина, если режим не Full и есть `next_from_message_id`) и `next_from_message_id` (определяется через `oldest_tdlib_message_id`).

### События топиков (`topic_events`)

- `publish_topic_event` вызывается при обновлении топика форума (событие `TopicUpdated`):
  1. Вызывает `upsert_topic_snapshot`, которая сохраняет топик через `upsert_topic`.
  2. Выполняет сверку команд топика через `reconcile_topic_commands_from_provider_state` для `topic_close`/`topic_reopen`.
  3. Публикует событие `telegram.topic.updated` с subject `telegram_topic` и payload, содержащим объект `TelegramTopic`.
- `topic_unread_count` обрезает значение до `[0, i32::MAX]`.

### Синхронизация топиков (`topics`)

- `sync_forum_topics` запрашивает топики форума через `request_actor_get_forum_topics` (лимит 100) и upsert-ит каждый в хранилище через `upsert_topic`.
- Если аккаунт не `tdlib_qr_authorized` или нет активного актора, возвращает 0 без ошибки.
- `telegram_topic_id` = SHA-256 от `telegram_chat_id\0provider_topic_id` с префиксом `telegram_topic:v1:`.

### Поиск сообщений (`search`)

- `search_provider_messages` принимает `TelegramProviderSearchRequest` (account_id, опциональный provider_chat_id, query, limit).
- Если запрос пуст, возвращает `[]`.
- При `runtime_kind == "tdlib_qr_authorized"`:
  - При наличии `provider_chat_id` вызывает `request_actor_search_chat_messages`.
  - Иначе вызывает `request_actor_search_messages`.
  - Каждый снимок ингестируется через `ingest_tdlib_message_snapshot` и публикуется raw-сигнал через `publish_observed_message_raw_signal`.
- Для fixture-режима возвращает `[]`.

### Управление актором TDLib (`tdlib_actor`)

- `ensure_tdlib_actor` проверяет наличие живого актора; если нет — создаёт новый:
  1. Извлекает ключ сессии через `optional_telegram_session_key`.
  2. Вызывает `spawn_tdlib_actor`, получает `command_tx` и (опционально) `runtime_event_tx`.
  3. При наличии `event_bridge` запускает `spawn_telegram_runtime_event_bridge`.
  4. Сохраняет дескриптор актора с состоянием `running_actor_state`.

## Модели запросов/ответов (`models`)

- **TelegramRuntimeStartRequest**, **TelegramRuntimeStopRequest**, **TelegramRuntimeRestartRequest** — требуют непустой `account_id`.
- **TelegramChatSyncRequest** — `account_id` обязателен, `limit` (опционально) валидируется через `validate_limit`.
- **TelegramHistorySyncRequest**:
  - `account_id` и `provider_chat_id` обязательны.
  - `from_message_id` должно быть положительным целым (TDLib message id).
  - Режим `"older"` требует наличия `from_message_id`.
  - `mode` по умолчанию `Latest`, допустимые значения: `latest`, `older`, `full` (сериализация `snake_case`).
- **TelegramHistorySyncResponse** — поля `has_more`, `next_from_message_id`.
- **TelegramMediaDownloadRequest**:
  - `tdlib_file_id` должно быть > 0.
  - `priority` — в диапазоне 1–32.
  - `provider_attachment_id` по умолчанию `tdlib-file:{file_id}`, `content_type` по умолчанию `application/octet-stream`.
- **TelegramMediaSendRequest** — содержит `media_type` (фото, видео, документ, аудио, голосовое, стикер, анимация). Преобразование из строки: `"voice"`/`"voice_note"` → Voice, `"animation"`/`"gif"` → Animation; неизвестные типы → ошибка.
- **TelegramRuntimeStatus** — отражает состояние runtime: `tdjson_path`, `tdjson_runtime_available`, `telegram_api_id_configured`, `live_send_available`, `runtime_blockers` и др.
```

# Source coverage / Покрытие источников

- `backend/src/integrations/telegram/runtime/manager/message_events/projection.rs`:
  - Функции проекции `update_message_reaction_summary`, `project_provider_message_content_observation`, `project_provider_message_edit_observation`, `observed_edit_timestamp`.
  - Использование `derive_tdlib_reaction_summary_metadata`, поля метаданных `reaction_summary`, `text`, `tdlib_content`, `last_provider_content_update_source`, `provider_edit_timestamp`, `last_provider_edit_source`, `tdlib_reply_markup`.
- `backend/src/integrations/telegram/runtime/manager/message_events/tests.rs`:
  - Тесты, подтверждающие, что события без спроецированного сообщения пропускаются.
  - Подтверждение перехода с `telegram.message.created` на `signal.raw.telegram.message.observed`.
- `backend/src/integrations/telegram/runtime/manager/participant_events.rs`:
  - Логика публикации `telegram.participant.updated`: формирование события, поля subject (`telegram_chat_participant`), payload, provenance, очистка префикса `tdlib.`.
- `backend/src/integrations/telegram/runtime/manager/participants.rs`:
  - Основная функция `sync_chat_members`, константа `TELEGRAM_MEMBER_SYNC_TARGET_LIMIT = 500`.
  - Ветвление по типам чатов: приватный, базовые группы, супергруппы.
  - Функции `tdlib_supergroup_id`, `tdlib_basic_group_id`, `tdlib_private_user_id`.
  - `sync_provider_roster_snapshots`, `sync_private_chat_members`, `merge_supergroup_member_snapshots`.
  - Вычисление `telegram_participant_id`, признак исчерпывающего реестра, обработка отсутствующих участников.
- `backend/src/integrations/telegram/runtime/manager/participants_runtime_tests.rs`:
  - Тесты сверки команд после `sync_provider_roster_snapshots`: join и leave reconciliation, порядок событий в `event_log`.
- `backend/src/integrations/telegram/runtime/manager/realtime_events.rs`:
  - `spawn_telegram_runtime_event_bridge` и все обрабатываемые варианты `TelegramRuntimeEvent`.
  - Константа `TELEGRAM_RUNTIME_EVENT_BRIDGE_RUNTIME` и gating логика.
  - Функции `publish_command_reconciled_events` и `command_event_payload`.
- `backend/src/integrations/telegram/runtime/manager/registry.rs`:
  - Методы `actor_state`, `stop_account`, `set_actor_handle`, `actor_command_tx`, `active_account_ids`.
- `backend/src/integrations/telegram/runtime/manager/search.rs`:
  - `search_provider_messages`, вызов `request_actor_search_chat_messages` / `request_actor_search_messages`, ингест результатов, публикация raw-сигнала.
- `backend/src/integrations/telegram/runtime/manager/send.rs`:
  - `send_manual_message`, `send_reply_message`, `send_forward_message`. Диспетчеризация по runtime_kind (`fixture`, `tdlib_qr_authorized`, `live_blocked`).
  - Ингест через `ingest_tdlib_message_snapshot`, вычисление `rendered_preview_hash`.
- `backend/src/integrations/telegram/runtime/manager/sync_chats.rs`:
  - `sync_chats`: fixture-ветвь и tdlib-ветвь, ингест чатов и папок, извлечение folder_ids через `tdlib_folder_ids_from_chat_snapshots`.
- `backend/src/integrations/telegram/runtime/manager/sync_history.rs`:
  - `sync_history` диспетчеризация; `sync_fixture_history` использует `recent_messages`.
- `backend/src/integrations/telegram/runtime/manager/sync_history_tdlib.rs`:
  - `sync_tdlib_history`: проверка приватного чата для Full, ингест сообщений, публикация сигналов, сверка команд участия и реакций.
  - Функция `ensure_private_chat_for_full_sync` и ограничение полной синхронизации только приватными чатами.
- `backend/src/integrations/telegram/runtime/manager/tdlib_actor.rs`:
  - `ensure_tdlib_actor`: проверка, создание актора, ключ сессии, запуск event-моста, установка handle.
- `backend/src/integrations/telegram/runtime/manager/topic_events.rs`:
  - `publish_topic_event`: `upsert_topic_snapshot`, `reconcile_topic_commands_from_provider_state` для `topic_close`/`topic_reopen`, событие `telegram.topic.updated`.
  - Функция `topic_unread_count`.
- `backend/src/integrations/telegram/runtime/manager/topics.rs`:
  - `sync_forum_topics` и вычисление `telegram_topic_id`.
- `backend/src/integrations/telegram/runtime/models.rs`:
  - Все модели запросов/ответов, их валидация, режимы `TelegramHistorySyncMode`, `TelegramMediaSendType`, `TelegramMediaDownloadRequest` с дефолтами, `TelegramRuntimeStatus`.

# Drift candidates / Кандидаты на drift

- **Тип события создания сообщения** — тест `publish_message_created_event_publishes_signal_hub_raw_signal_instead_of_legacy_event` явно проверяет отсутствие записей с `telegram.message.created` и использование `signal.raw.telegram.message.observed`. Если в существующих runbook'ах, дашбордах или внутренней документации всё ещё фигурирует `telegram.message.created`, это прямое расхождение.
- **Поле метаданных `provider_edit_timestamp`** — функция `observed_edit_timestamp` ожидает значение в формате RFC 3339. Если в какой‑либо документации указан иной формат или иное имя поля, возможен drift.
- **Валидация `from_message_id`** — модель `TelegramHistorySyncRequest` требует, чтобы `from_message_id` было положительным; режим `"older"` без идентификатора вызывает ошибку. Если внешние спецификации API допускают отрицательные значения или не требуют `from_message_id` для older, это расхождение.
- **Полная синхронизация истории только для приватных чатов** — функция `ensure_private_chat_for_full_sync` запрещает режим `Full` для групп и каналов. Если контракты API или пользовательская документация предполагают обратное, возможен drift.
- Других расхождений в рамках предоставленного контекста не выявлено.
