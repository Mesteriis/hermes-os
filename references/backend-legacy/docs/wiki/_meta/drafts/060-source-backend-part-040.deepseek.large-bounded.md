### Summary
Добавляется документация по двум ключевым интеграционным модулям бэкенда: клиент OmniRoute (чат и эмбеддинги) и Telegram-клиент (учётные записи, чаты, команды, жизненный цикл сообщений, реконсиляция). Описания опираются исключительно на исходный код, попавший в чанк `060-source-backend-part-040`.

### Proposed pages

**Путь**: `components/backend.md`

```markdown
# Backend

## Интеграция с OmniRoute

Модуль `backend/src/integrations/omniroute/client/` предоставляет клиент для взаимодействия с сервисом OmniRoute.

### Модели

- `OmniRouteChatResult` – результат чат-запроса: поле `model` и текстовый `content`.
- `OmniRouteEmbedResult` – результат эмбеддинга: поле `model` и вектор `embedding` (`Vec<f32>`).

### Транспорт

`OmniRouteClient` реализует HTTP-клиент (на базе `reqwest`):
- `fn endpoint(&self, path: &str) -> Result<Url, OmniRouteError>` – сборка URL.
- `async fn get_json<T>(&self, path: &str)` – GET с авторизацией (`bearer_auth`).
- `async fn post_json<T>(&self, path: &str, body: &Value)` – POST с JSON-телом и авторизацией.
- `async fn decode_response<T>(response: reqwest::Response)` – проверка статуса и десериализация; ошибки: `OmniRouteError::Endpoint` (неуспешный статус) и `OmniRouteError::Protocol` (проблема десериализации).

## Интеграция с Telegram

Модуль `backend/src/integrations/telegram/client/` реализует логику аккаунтов, чатов, сообщений и команд для Telegram.

### Учётные записи (`accounts`)

`TelegramStore` управляет аккаунтами через `ProviderAccountStore` и `SecretReferenceStore` / `TelegramSecretVault`.

- **Фикстурная настройка** (`setup_fixture_account`): принимает `TelegramAccountSetupRequest`, валидирует, требует `provider_kind` из подмножества `telegram_user`/`telegram_bot`. Конфигурация: `runtime: "fixture"`, `tdlib_data_path`, `transcription_enabled`.
- **Live-настройка** (`setup_live_blocked_account`): валидирует, определяет runtime: `"tdlib_qr_authorized"` (если QR-авторизация) или `"live_blocked"`. Сохраняет `api_id` (если не QR) и `tdlib_data_path`. Сохраняет учётные данные через `store_live_account_credentials`.
- **Сохранение учётных данных** (`store_account_credential`): создаёт `NewSecretReference`, сохраняет в `SecretReferenceStore`, пишет секрет в vault, привязывает к аккаунту через `provider_secret_binding_store`. Для `TelegramUser` сохраняет `api_hash` (если не QR) и опциональный `session_encryption_key`. Для `TelegramBot` сохраняет `bot_token`.
- **Жизненный цикл аккаунтов**: `list_accounts` (с фильтром `include_removed`), `logout_account` и `remove_account` меняют `lifecycle_state` в конфиге, записывая временные метки (`logged_out_at`, `removed_at`, `lifecycle_updated_at`). Удалённый аккаунт нельзя перевести в другое состояние повторно.

### Чаты (`chats`)

- **`upsert_chat`**: вставка в `telegram_chats` с `INSERT ... ON CONFLICT DO UPDATE`; метаданные объединяются через `||`.
- **Списки**: `list_chats` (опциональный фильтр по `account_id`, лимит), `list_all_chats_for_account`, `list_chat_group_filters` (включает локальную группу `"All"` и папки из метаданных `folder_labels`).
- **Применение папок провайдера** (`apply_provider_chat_folders`): по снимкам `TelegramTdlibChatFolderSnapshot` обновляет `folder_labels`, `folder_name`, `provider_folder_ids`.
- **Проекция метаданных из TDLib** (`tdlib_chat_projection_metadata`):
  - Базовые поля: `runtime: "tdlib"`, `raw_record_id`.
  - `tdlib_permissions` – избранные булевы разрешения.
  - `is_marked_as_unread` – из `raw.is_marked_as_unread`.
  - `tdlib_notification_settings` (`use_default_mute_for`, `mute_for`), вычисляется `is_muted`.
  - `tdlib_chat_positions` (main, archive, folder), вычисляются `is_archived`, `is_pinned`.
  - Для приватного чата: `is_saved_messages` и `saved_messages_source`, если ID пользователя совпадает с владельцем.
  - Для супергруппы: `tdlib_supergroup_id`, `is_channel_supergroup`, `is_forum`.
  - Для basic-группы: `tdlib_basic_group_id`, `is_basic_group`.
- **Флаги метаданных** (`metadata_flags`):
  - `set_chat_metadata_bool`, `set_chat_metadata_number`, `set_chat_last_read_at`.
  - `apply_provider_unread_counts`: записывает `unread_count`, `mention_count`, `last_read_inbox_provider_message_id`.
  - `recompute_chat_unread_count`: пересчитывает непрочитанные из хранилища сообщений.

### Состояние чатов и реконсиляция (`chat_state`, `chat_reconciliation`)

- `apply_provider_marked_as_unread`, `apply_provider_notification_settings`, `apply_provider_chat_position` – обновляют метаданные чата на основе событий провайдера.
- `TelegramProviderChatPositionUpdate` – структура с `list_kind`, `order`, `is_pinned`, `source_event`.
- **Реконсиляция**: функции сверяют состояние провайдера с активными командами:
  - `reconcile_dialog_boolean_commands_from_provider_state` – общая логика для архивности (`archive`/`unarchive`), mute (`mute`/`unmute`), mark_unread (`mark_unread`), pin (`pin`/`unpin`). Команды с несовпадающим состоянием переходят в `mark_command_mismatch`, совпадающие – в `mark_command_reconciled`. Учитывается `PROVIDER_RECONCILIATION_CLOCK_SKEW` (5 секунд).
  - `reconcile_mark_read_commands_from_provider_state` – для команд `mark_read` сравнивает `provider_message_id` целевого сообщения с последним прочитанным.
  - `reconcile_mute_commands_from_provider_state`, `reconcile_pin_commands_from_provider_state`, `reconcile_archive_commands_from_provider_state` – используют `reconcile_dialog_boolean_commands_from_provider_state`.

### Команды (`commands`)

`TelegramProviderWriteCommand` хранит всю информацию об исходящей операции.

- **Создание команды** (`insert_command`): `command_id` генерируется как `tcmd_{timestamp}_{hash}`, статус – `queued`. Параметры: `command_kind`, `idempotency_key`, `capability_state`, `action_class`, `confirmation_decision`, `payload`, `target_ref`, `audit_metadata`.
- **Жизненный цикл**:
  - `update_command_status` – обновление статуса и `result_payload`.
  - `retry_command` / `schedule_command_retry` – перевод в `retrying` с `next_attempt_at = now + 30s`, снятие блокировок.
  - `dead_letter_command` – перевод в `dead_letter`.
  - `mark_command_awaiting_provider` – перевод в `executing` с `reconciliation_status: "awaiting_provider"`.
  - `mark_command_reconciled` – завершение: `status = completed`, `reconciliation_status = observed`, заполнение `provider_observed_at`, `provider_state`.
  - `mark_command_mismatch` – фиксация расхождения.
- **Запросы**: `find_command_by_idempotency`, `list_commands`, `list_commands_filtered`, `list_queued_commands_for_execution` (отбирает по статусу, retry_count, времени, допустимым `command_kind`).

### Жизненный цикл сообщений (`lifecycle`)

- **Версии сообщений** (`message_versions`):
  - `insert_message_version`: создаёт запись с автоинкрементным `version_number`.
  - `latest_message_version`, `latest_version_number`.
  - `record_provider_edit_observation`: создаёт версию с данными от провайдера, с дедупликацией по телу сообщения, таймстемпу и `source_event`.
  - `local_edit_diff`: формирует diff (длины, хеши SHA-256, превью) между предыдущим и новым текстом.
- **Tombstone** (`tombstones`):
  - `insert_tombstone`: сохраняет факт удаления/восстановления с параметрами `reason_class`, `actor_class`, `is_provider_delete`, `is_local_visible`.
  - `is_message_visible`: определяет текущую видимость по последнему tombstone.
  - `list_tombstones`.
  - `record_provider_delete_observation`: фиксирует удаление провайдером с проверкой дубликатов.
- **Операции** (`operations`):
  - `record_edit`: создаёт версию с diff, ставит в очередь команду `edit`.
  - `record_delete`: создаёт tombstone и команду `delete`.
  - `record_restore_visibility`: создаёт tombstone с `is_local_visible = true` и команду `restore_visibility`.
  - `record_pin_state`: сохраняет observation о pin/unpin и команду `pin`/`unpin`.
- **Реконсиляция сообщений** (`provider_reconciliation`):
  - `reconcile_edit_commands_from_provider_state`: сравнивает `new_text` из payload команды с актуальным `body_text` от провайдера.
  - `reconcile_message_pin_commands_from_provider_state`: сравнивает ожидаемое `is_pinned`.
  - `reconcile_delete_commands_from_provider_state`: завершает команды `delete` как reconciled.

### Идентификаторы (`identifiers`)

Стабильные идентификаторы на основе SHA-256:
- `telegram_chat_id` → `telegram_chat:v4:<hash>`
- `telegram_message_id` → `message:v4:telegram:<hash>`
- `telegram_raw_record_id` → `raw:v4:telegram:<hash>`
- `telegram_text_preview_hash` → `sha256:<hash>`
- `telegram_secret_ref` → `secret:provider-account:<account>:<purpose>`
- `telegram_account_from_provider_account` – преобразование `ProviderAccount` → `TelegramAccount`.
- `telegram_account_lifecycle_state` – значение `lifecycle_state` из конфига (по умолчанию `active`).
- `ensure_telegram_account_active` – проверка, что аккаунт в состоянии `active`.

### Ошибки (`TelegramError`)

Перечисление ошибок включает:
- `InvalidRequest`, `TdlibRuntimeUnavailable`, `TdlibRuntime`, `QrGeneration`, `QrLoginNotFound`
- `ProviderAccountStore`, `MediaStorage`
- Прозрачные варианты: `SecretReference`, `DatabaseVault`, `HostVault`, `CommunicationMessagePort`, `ObservationStore`, `Sqlx`.

### Интеграция с системой наблюдений

- `link_telegram_entity_in_transaction` – связывание observation с сущностью в домене `telegram`.
- Во всех операциях создания/обновления (чаты, команды, версии, tombstone) вызываются `capture_*_observation_in_transaction`, фиксирующие событие и связь.
```

### Source coverage

- `backend/src/integrations/omniroute/client/models.rs` – описание структур `OmniRouteChatResult`, `OmniRouteEmbedResult`.
- `backend/src/integrations/omniroute/client/transport.rs` – методы `endpoint`, `get_json`, `post_json`, `decode_response`.
- `backend/src/integrations/omniroute/mod.rs` – реэкспорт модуля `client`.
- `backend/src/integrations/telegram/client/accounts.rs` – объявление подмодулей `credential_bindings`, `fixture_setup`, `lifecycle`, `live_credentials`, `live_setup`.
- `backend/src/integrations/telegram/client/accounts/credential_bindings.rs` – метод `store_account_credential`.
- `backend/src/integrations/telegram/client/accounts/fixture_setup.rs` – метод `setup_fixture_account`.
- `backend/src/integrations/telegram/client/accounts/lifecycle.rs` – методы `list_accounts`, `logout_account`, `remove_account`, `update_account_lifecycle`.
- `backend/src/integrations/telegram/client/accounts/live_credentials.rs` – метод `store_live_account_credentials`.
- `backend/src/integrations/telegram/client/accounts/live_setup.rs` – метод `setup_live_blocked_account`, вспомогательные `live_runtime`, `live_account_config`.
- `backend/src/integrations/telegram/client/chat_metadata.rs` (первая часть) – функция `tdlib_chat_projection_metadata` и вложенные вспомогательные функции (permissions, notification_settings, chat_positions, private/basic/supergroup метаданные).
- `backend/src/integrations/telegram/client/chat_reconciliation.rs` – функции `reconcile_dialog_boolean_commands_from_provider_state`, `expected_*_state_for_command_kind`, `dialog_boolean_reconciliation_payload`.
- `backend/src/integrations/telegram/client/chat_state.rs` (первая часть) – методы `apply_provider_marked_as_unread`, `apply_provider_notification_settings`, `apply_provider_chat_position`, структура `TelegramProviderChatPositionUpdate`, функции `reconcile_marked_as_unread_commands_from_provider_state`, `reconcile_mark_read_commands_from_provider_state`.
- `backend/src/integrations/telegram/client/chats.rs` (первая часть) – методы `upsert_chat`, `list_chats`, `list_chat_group_filters`, `apply_provider_chat_folders`, `capture_chat_observation_in_transaction`.
- `backend/src/integrations/telegram/client/chats/metadata_flags.rs` – методы `set_chat_metadata_bool`, `set_chat_metadata_number`, `set_chat_last_read_at`, `apply_provider_unread_counts`, `recompute_chat_unread_count`.
- `backend/src/integrations/telegram/client/commands.rs` (первая часть) – функции `new_command_id`, `insert_command`, `update_command_status`, `retry_command`, `schedule_command_retry`, `dead_letter_command`, `mark_command_awaiting_provider`, `mark_command_reconciled`.
- `backend/src/integrations/telegram/client/commands/queries.rs` – функции `find_command_by_idempotency`, `list_commands`, `list_commands_filtered`, `list_queued_commands_for_execution`.
- `backend/src/integrations/telegram/client/errors.rs` – определение `TelegramError`.
- `backend/src/integrations/telegram/client/evidence.rs` – `link_telegram_entity_in_transaction`.
- `backend/src/integrations/telegram/client/identifiers.rs` – все функции идентификации и работы с конфигом аккаунта.
- `backend/src/integrations/telegram/client/lifecycle.rs` – публичный реэкспорт функций из подмодулей `message_versions`, `operations`, `provider_reconciliation`, `tombstones`.
- `backend/src/integrations/telegram/client/lifecycle/ids.rs` – `new_version_id`, `new_tombstone_id`.
- `backend/src/integrations/telegram/client/lifecycle/message_versions.rs` – `insert_message_version`, `list_message_versions`, `latest_message_version`, `latest_version_number`, `record_provider_edit_observation`, `local_edit_diff`.
- `backend/src/integrations/telegram/client/lifecycle/operations.rs` – `record_edit`, `previous_message_body`, `record_delete`, `record_restore_visibility`, `record_pin_state`.
- `backend/src/integrations/telegram/client/lifecycle/provider_reconciliation.rs` – `reconcile_edit_commands_from_provider_state`, `reconcile_message_pin_commands_from_provider_state`, `reconcile_delete_commands_from_provider_state`.
- `backend/src/integrations/telegram/client/lifecycle/tombstones.rs` – `insert_tombstone`, `list_tombstones`, `is_message_visible`, `record_provider_delete_observation`.

### Drift candidates

Из предоставленного контекста расхождений кода, документации или ADR не видно (нет эталонной документации для сравнения).
