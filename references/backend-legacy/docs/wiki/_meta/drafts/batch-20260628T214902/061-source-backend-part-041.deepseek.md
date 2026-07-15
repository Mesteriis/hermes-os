### Summary / Резюме

Страница `components/backend.md` обновлена русским описанием модуля `backend/src/integrations/telegram/client/`. Добавлены: структура модуля, ключевые модели данных, хранилище `TelegramStore`, основные операции (инжест сообщений, запросы, управление участниками, реакции, наблюдения) и ограничения контекста. Все утверждения опираются исключительно на встроенный исходный код.

### Proposed pages / Предлагаемые страницы

#### `components/backend.md`

```markdown
# Backend: клиент Telegram

Модуль `backend/src/integrations/telegram/client/` реализует интеграцию с Telegram в качестве провайдера коммуникаций. Он управляет аккаунтами, чатами, сообщениями, участниками, реакциями и синхронизацией состояния с TDLib.

## Структура модуля

Модуль организован в несколько подмодулей:

- `accounts` — управление учётными записями Telegram
- `chat_metadata`, `chat_reconciliation`, `chat_state`, `chats` — работа с чатами и их состоянием
- `messages` — обработка сообщений (получение, инжест, запросы, вложения, метаданные)
- `models` — модели данных (аккаунты, чаты, сообщения, топики, QR-логин)
- `participants` — участники чатов и их синхронизация
- `reactions` — реакции на сообщения
- `observations` — наблюдения (метаданные, содержимое, состояния вложений)
- `commands` (не показан в данном контексте) — команды провайдера
- `store` (не показан) — `TelegramStore`
- `lifecycle` — жизненный цикл аккаунтов и команд
- `rows` — преобразование строк БД в доменные типы
- `errors` — `TelegramError`

Корневой `mod.rs` реэкспортирует ключевые публичные типы и функции, включая `TelegramStore`, модели запросов/ответов, типы сообщений и функций для работы с реакциями и участниками.

## Основные типы и модели

### Аккаунты (`models/accounts.rs`)

- `TelegramAccountSetupRequest` / `TelegramLiveAccountSetupRequest` — запросы на настройку аккаунта с валидацией полей (`account_id`, `display_name`, `external_account_id`, `provider_kind`, `api_id`, `api_hash`, `bot_token` и др.). Поддерживается авто-определение QR-авторизации через `is_finalized_qr_user_account`.
- `TelegramAccount` — представление учётной записи: `account_id`, `provider_kind`, `display_name`, `external_account_id`, `runtime`, `lifecycle_state`, `transcription_enabled`, `tdlib_data_path`, `created_at`, `updated_at`.
- `TelegramAccountSetupResponse` — ответ с `credential_bindings`.
- `TelegramCredentialBinding` — связь с секретом: `secret_purpose`, `secret_ref`, `secret_kind` (`SecretKind`), `store_kind` (`SecretStoreKind`).

### Чаты (`models/chats.rs`)

- `NewTelegramChat` — данные для создания/обновления чата. Поля: `account_id`, `provider_chat_id`, `chat_kind` (`TelegramChatKind`), `title`, `username`, `sync_state` (`TelegramSyncState`), `last_message_at`, `metadata` (JSON). Метод `validate` проверяет обязательные поля.
- `TelegramChat` — представление чата из БД: `telegram_chat_id`, `account_id`, `provider_chat_id`, `chat_kind`, `title`, `username`, `sync_state`, `last_message_at`, `metadata`, `created_at`, `updated_at`.
- `TelegramChatKind` — `Private`, `Group`, `Channel`, `Bot`. Преобразуется из строки через `TryFrom<&str>`.
- `TelegramSyncState` — `Fixture`, `Syncing`, `Synced`, `Degraded`, `Error`.
- `TelegramChatMember` — участник чата: `sender_id`, `sender_display_name`, `message_count`, `last_message_at`, `source`, `provider_member_id`, `username`, `role`, `status`, `is_admin`, `is_owner`, `permissions` (JSON), `observed_at`.
- `NewTelegramChatParticipant` — для upsert: `participant_id`, `telegram_chat_id`, `account_id`, `provider_chat_id`, `provider_member_id`, `display_name`, `username`, `role`, `status`, `is_admin`, `is_owner`, `permissions`, `raw_payload`, `source`.

### Сообщения (`models/messages.rs`)

- `NewTelegramMessage` — входное сообщение для инжеста: `account_id`, `provider_chat_id`, `provider_message_id`, `chat_kind`, `chat_title`, `sender_id`, `sender_display_name`, `text`, `import_batch_id`, `occurred_at`, `delivery_state`. Методы: `validate` (требует также `text`), `validate_for_runtime` (для tdlib допускает пустой `text`), `source_fingerprint` (SHA-256 от ключевых полей).
- `TelegramDeliveryState` — `Received`, `Sent`, `SendDryRun`, `SendBlocked`.
- `TelegramObservedMessage` — результат инжеста: `raw_record_id`, `message_id`, `raw` (NewRawCommunicationRecord), `telegram_chat_id`.
- `TelegramMessage` — проекция для внешнего использования: `message_id`, `raw_record_id`, `account_id`, `provider_message_id`, `provider_chat_id` (опционально), `chat_title`, `sender`, `sender_display_name`, `text`, `occurred_at`, `projected_at`, `channel_kind`, `delivery_state`, `metadata` (JSON).
- `TelegramMessageVersion` — версионирование (ADR-0091): `version_id`, `version_number`, `body_text`, `edit_timestamp`, `source_event`, `raw_diff_payload`, `provenance`.
- `TelegramMessageTombstone` — запись об удалении/скрытии: `tombstone_id`, `reason_class` (`TombstoneReasonClass`), `actor_class` (`TombstoneActorClass`), `is_provider_delete`, `is_local_visible`, `metadata`, `provenance`.
- `TombstoneReasonClass` — `DeletedByOwner`, `DeletedByCounterparty`, `DeletedByProvider`, `ModerationRemoved`, `AccountRemoved`, `RetentionPolicy`, `Unknown`.
- `TombstoneActorClass` — `Owner`, `Provider`, `Automation`, `System`, `Unknown`.
- Запросы: `TelegramManualSendRequest` (валидация `command_id`, `account_id`, `provider_chat_id`, `text`), `TelegramReplyRequest`, `TelegramForwardRequest` (валидация `command_id`, `account_id`, `provider_chat_id`, `from_provider_chat_id`, `from_provider_message_id`).
- `TelegramCommandKind` — перечисление: `SendText`, `SendMedia`, `Edit`, `Delete`, `RestoreVisibility`, `MarkRead`, `MarkUnread`, `Pin`, `Unpin`, `Archive`, `Unarchive`, `Mute`, `Unmute`, `React`, `Unreact`, `Reply`, `Forward`, `Join`, `Leave`, `FolderAdd`, `FolderRemove`, `TopicCreate`, `TopicClose`, `TopicReopen`, `AdminAction`.

### QR-логин (`models/qr_login.rs`)

- `TelegramQrLoginStartRequest` — запрос с валидацией (`account_id`, `display_name`, `external_account_id`, `api_id`, `api_hash`). Методы `with_app_credentials`, `required_api_id`, `required_api_hash`.
- `TelegramQrLoginPasswordRequest` — содержит `password`.
- `TelegramQrLoginStatusResponse` — поля: `setup_id`, `account_id`, `status` (`TelegramQrLoginStatus`: `WaitingQrScan`, `WaitingPassword`, `Ready`, `Expired`, `Failed`, `RuntimeUnavailable`), `qr_link`, `qr_svg`, `poll_after_ms` и др.

### Топики (`models/topics.rs`)

- `TelegramTopic` — `topic_id`, `telegram_chat_id`, `account_id`, `provider_topic_id`, `provider_chat_id`, `title`, `icon_emoji`, `is_pinned`, `is_closed`, `unread_count`, `last_message_at`, `metadata`, `created_at`, `updated_at`.
- `NewTelegramTopic` — структура без временных меток для создания.
- `TelegramTopicCreateRequest` / `TelegramTopicCloseRequest` — запросы с валидацией `command_id`, `account_id`, `provider_chat_id` (+ `title` для создания, `is_closed` для закрытия).

## Хранилище `TelegramStore`

`TelegramStore` (определён в `store.rs`, не включён в данный контекст) использует пул соединений с БД и методы, реализованные в подмодулях:

### Поиск аккаунтов и чатов

- **`account_lookup.rs`**: `telegram_provider_account` — получает `ProviderAccount` по `account_id`, проверяет, что `provider_kind` — это telegram.
- **`chat_lookup.rs`**: `telegram_chat` — поиск чата в таблице `telegram_chats` по `account_id` и `provider_chat_id`.

### Инжест сообщений

- **`ingestion.rs`**: `observe_message_with_runtime` — основной конвейер: валидация, upsert чата (`NewTelegramChat` с `sync_state: Synced`), извлечение метаданных (упоминания, публичная ссылка, вложения, медиа-альбомы, структурированные доказательства, реакции), формирование payload и `NewRawCommunicationRecord`, генерация `message_id` через `stable_hash`.
- **`manual_send.rs`**: `manual_send_message` — отправка сообщения вручную; поддерживается только runtime `fixture` (для `tdlib` возвращает ошибку). Формирует `NewTelegramMessage` с `provider_message_id = "manual:{command_id}"` и вызывает `ingest_fixture_message`.
- **`tdlib_ingestion.rs`**: `ingest_tdlib_message_snapshot` — преобразует `TelegramTdlibMessageSnapshot` в `NewTelegramMessage` (чат из существующего или `Private`), вызывает `ingest_message_with_runtime` с меткой `"tdlib"`.

### Запросы сообщений (`queries.rs`)

- `message_by_provider_message_id`, `message_by_id` — поиск одного сообщения.
- `recent_messages` — список с фильтром по `account_id`, `provider_chat_id`, лимит (валидация через `validate_message_list_limit`).
- `messages_by_ids` — пакетный запрос по `message_ids`.
Все используют `provider_channel_message_store` с `TELEGRAM_CHANNEL_KINDS = ["telegram_user", "telegram_bot"]`.

### Вложения (`attachments.rs`)

- `attachment_anchor_for_message` — получает `TelegramAttachmentAnchor` для сообщения через `provider_channel_message_store`.
- `update_message_attachment_download_state` — обновляет состояние загрузки вложения, записывая наблюдение `TelegramAttachmentDownloadObservation`.

### Интеллект (`intelligence.rs`)

- `refresh_message_intelligence_candidates` — заглушка, ничего не делает.

### Публикация сигналов (`raw_signals.rs`)

- `publish_observed_message_raw_signal` — сохраняет сырую запись, строит `CommunicationRawSignalEvent`, идемпотентно добавляет в `EventStore` и опционально публикует в `EventBus`.

## Участники чатов (`participants.rs`)

- `upsert_chat_participant` — upsert записи в `telegram_chat_participants` с фиксацией наблюдения (`TELEGRAM_CHAT_PARTICIPANT`) через `ObservationStore` и `link_telegram_entity_in_transaction`.
- `mark_absent_members_from_exhaustive_roster` — для участников из `source = 'tdlib'`, отсутствующих в переданном списке `observed_member_ids`, устанавливает статус `absent_exhaustive` и дополняет `permissions`/`raw_payload`. Каждое изменение записывает наблюдение.
- `telegram_self_provider_member_id` — извлекает свой `provider_member_id` из `external_account_id`, поддерживая форматы `user:<id>` и `telegram:<id>`.
- `reconcile_join_commands_from_provider_roster` / `reconcile_leave_commands_from_provider_roster` — сверяет команды `join`/`leave` со статусом `queued`, `retrying`, `executing` и признаёт их сверенными через `mark_command_reconciled`.

## Реакции (`reactions.rs`)

- `sync_provider_reactions` — синхронизирует реакции из `TdlibProviderReaction`: upsert в `telegram_message_reactions` с `is_active = true`, запись наблюдения.
- `sync_self_provider_reactions` — деактивирует реакции, не входящие в `chosen_reactions`, и активирует выбранные.
- `reconcile_reaction_commands_from_provider_reactions` — сверяет команды `react`/`unreact` с провайдерским состоянием.
Все операции записывают наблюдения (`TELEGRAM_MESSAGE_REACTION`) и используют транзакции.

## Метаданные сообщений (`message_metadata.rs` и `reaction_metadata.rs`)

Функции извлечения структурированных данных из TDLib JSON:

- `derive_mention_metadata` — упоминания через регулярное выражение (`@...`) и сущности TDLib (`textEntityTypeMention`, `textEntityTypeMentionName`).
- `telegram_public_message_link` — ссылка `https://t.me/{username}/{message_id}`, если username и message_id валидны.
- `derive_tdlib_media_album_metadata` — `media_album_id` и `album_key`.
- `derive_tdlib_attachment_metadata` — вложения: стикеры, анимации, видео-кружки; с mime-типом и именем файла.
- `derive_tdlib_structured_evidence` — опросы (`telegram_poll`), локации (`telegram_location`), контакты (`telegram_contact_card`), join/leave (`telegram_join_leave`).
- `derive_tdlib_reaction_summary_metadata` — сводка реакций (эмодзи и кастомные) из `interaction_info.reactions`.
- `derive_tdlib_provider_reactions` — реакции отправителей из `recent_reactions`.
- `derive_tdlib_chosen_reaction_emojis` — эмодзи, выбранные текущим пользователем.

## Наблюдения (`observations.rs`)

- `TelegramAttachmentDownloadObservation` — данные о загрузке вложения.
- Методы `TelegramStore`: `append_message_metadata_observation`, `append_message_content_observation`, `append_message_pin_observation`, `append_attachment_download_observation`. Все они вызывают `append_message_observation_event` с соответствующим `event_kind` и payload, делегируя `provider_observation_events().append_provider_message_observation()`.

## Константы

- `TELEGRAM_MESSAGE_RECORD_KIND = "telegram_message"`
- `TELEGRAM_CHAT_RECORD_KIND = "telegram_chat"`
- `TELEGRAM_ACCOUNT_ACTIVE = "active"`, `TELEGRAM_ACCOUNT_LOGGED_OUT = "logged_out"`, `TELEGRAM_ACCOUNT_REMOVED = "removed"`

## Тесты

- `reactions/tests.rs`: проверка синхронизации реакций (деактивация отсутствующих собственных реакций).
- `participants/tests.rs`: тесты `telegram_self_provider_member_id`, `tdlib_self_membership_lifecycle`, `mark_absent_members_from_exhaustive_roster` (интеграционный).

## Ограничения контекста

Следующие данные отсутствуют или обрезаны в предоставленном чанке и не могут быть подтверждены:

- `store.rs` — определение `TelegramStore`.
- `commands.rs`, `lifecycle.rs`, `evidence.rs`, `rows.rs`.
- Полные тексты `message_metadata.rs`, `reaction_metadata.rs`, `models/messages.rs`, `participants.rs`, `reactions.rs` (обрезаны после 12000 символов).
```

### Source coverage / Покрытие источников

- `backend/src/integrations/telegram/client/mod.rs` — список подмодулей, константы (`TELEGRAM_MESSAGE_RECORD_KIND`, `TELEGRAM_CHAT_RECORD_KIND`, `TELEGRAM_ACCOUNT_*`), публичный API (реэкспорты моделей, `TelegramStore`, функций реакций и участников).
- `backend/src/integrations/telegram/client/messages.rs` — структура подмодуля `messages`.
- `backend/src/integrations/telegram/client/messages/account_lookup.rs` — методы `telegram_account_record`, `telegram_provider_account`.
- `backend/src/integrations/telegram/client/messages/attachments.rs` — `TelegramAttachmentDownloadStateUpdate`, методы `attachment_anchor_for_message`, `update_message_attachment_download_state`.
- `backend/src/integrations/telegram/client/messages/chat_lookup.rs` — метод `telegram_chat`, SQL-запрос к `telegram_chats`.
- `backend/src/integrations/telegram/client/messages/ingestion.rs` — конвейер `observe_message_with_runtime`: валидация, upsert чата, извлечение метаданных, формирование `NewRawCommunicationRecord`, `message_id` через `stable_hash`.
- `backend/src/integrations/telegram/client/messages/intelligence.rs` — заглушка `refresh_message_intelligence_candidates`.
- `backend/src/integrations/telegram/client/messages/manual_send.rs` — `manual_send_message`, ограничения по runtime, формирование `NewTelegramMessage`.
- `backend/src/integrations/telegram/client/messages/message_metadata.rs` (частично) — функции `derive_mention_metadata`, `telegram_public_message_link`, `derive_tdlib_media_album_metadata`, `derive_tdlib_attachment_metadata`, `derive_tdlib_structured_evidence`.
- `backend/src/integrations/telegram/client/messages/queries.rs` — методы `message_by_provider_message_id`, `message_by_id`, `recent_messages`, `messages_by_ids`.
- `backend/src/integrations/telegram/client/messages/raw_signals.rs` — `publish_observed_message_raw_signal`: сохранение, построение события, идемпотентная запись, broadcast.
- `backend/src/integrations/telegram/client/messages/reaction_metadata.rs` (частично) — `TdlibProviderReaction`, `derive_tdlib_reaction_summary_metadata`, `derive_tdlib_provider_reactions`, `derive_tdlib_chosen_reaction_emojis`, тесты.
- `backend/src/integrations/telegram/client/messages/tdlib_ingestion.rs` — `ingest_tdlib_message_snapshot`.
- `backend/src/integrations/telegram/client/models.rs` — реэкспорты подмодулей моделей.
- `backend/src/integrations/telegram/client/models/accounts.rs` — `TelegramAccountSetupRequest`, `TelegramLiveAccountSetupRequest` (валидация, QR-авторизация), `TelegramAccount`, `TelegramAccountSetupResponse`, `TelegramCredentialBinding`.
- `backend/src/integrations/telegram/client/models/chats.rs` — `TelegramChat`, `NewTelegramChat`, `TelegramChatKind`, `TelegramSyncState`, `TelegramChatMember`, `NewTelegramChatParticipant`.
- `backend/src/integrations/telegram/client/models/messages.rs` (частично) — `NewTelegramMessage` (валидация, fingerprint), `TelegramDeliveryState`, `TelegramObservedMessage`, `TelegramMessage`, `TelegramManualSendRequest`/`Response`, `TelegramReplyRequest`, `TelegramForwardRequest`, `TelegramMessageVersion`, `TelegramMessageTombstone`, `TombstoneReasonClass`, `TombstoneActorClass`, `TelegramCommandKind`.
- `backend/src/integrations/telegram/client/models/qr_login.rs` — `TelegramQrLoginStartRequest` (валидация, credentials), `TelegramQrLoginPasswordRequest`, `TelegramQrLoginStatus`, `TelegramQrLoginStatusResponse`.
- `backend/src/integrations/telegram/client/models/topics.rs` — `TelegramTopic`, `NewTelegramTopic`, `TelegramTopicCreateRequest`, `TelegramTopicCloseRequest`, запросы валидации.
- `backend/src/integrations/telegram/client/observations.rs` — `TelegramAttachmentDownloadObservation`, методы наблюдений (`append_message_metadata_observation`, `append_message_content_observation`, `append_message_pin_observation`, `append_attachment_download_observation`).
- `backend/src/integrations/telegram/client/participants.rs` (частично) — `upsert_chat_participant`, `mark_absent_members_from_exhaustive_roster`, `telegram_self_provider_member_id`, `reconcile_join_commands_from_provider_roster`, `reconcile_leave_commands_from_provider_roster`.
- `backend/src/integrations/telegram/client/participants/tests.rs` — тесты `telegram_self_provider_member_id`, `tdlib_self_membership_lifecycle`, `mark_absent_members_from_exhaustive_roster`.
- `backend/src/integrations/telegram/client/reactions.rs` (частично) — `sync_provider_reactions`, `sync_self_provider_reactions`, `reconcile_reaction_commands_from_provider_reactions`, `add_reaction`.
- `backend/src/integrations/telegram/client/reactions/tests.rs` — тест синхронизации реакций и деактивации отсутствующих.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения между кодом и документацией не видны. Отсутствуют существующая wiki-страница `components/backend.md`, ADR или другие артефакты, с которыми можно было бы сравнить.
