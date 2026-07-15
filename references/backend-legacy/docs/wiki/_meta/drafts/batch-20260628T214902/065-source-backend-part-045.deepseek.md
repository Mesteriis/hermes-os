### Summary / Резюме

Страница `components/backend.md` должна быть дополнена описанием модуля рантайма интеграции Telegram. В состав входят: модель состояний рантайма (`state.rs`), перечень команд и событий, логика получения статуса и проверки учётных данных (`status.rs`, `validation.rs`), клиент TDLib и загрузка нативной библиотеки (`tdjson/client.rs`, `library_paths.rs`), парсинг ответов TDLib (`tdjson/parsing/*`), манипуляции с папками чатов (`folder_requests.rs`), а также подсистема QR-логина (`tdjson/qr_login/*`). Все детали взяты исключительно из предоставленных исходных файлов.

### Proposed pages / Предлагаемые страницы

- `components/backend.md`

```markdown
# Компоненты бэкенда

## Модуль рантайма Telegram

Модуль `backend/src/integrations/telegram/runtime` управляет жизненным циклом и взаимодействием с Telegram через библиотеку TDLib. Он построен по actor-модели: внешние компоненты отправляют команды через канал, а actor обрабатывает их и генерирует события.

### Состояния рантайма

`enum TelegramRuntimeState` (файл `state.rs`) задаёт пять возможных состояний:

- `Stopped` — рантайм остановлен.
- `Running` — активен и готов обрабатывать запросы.
- `Blocked` — заблокирован (например, для учётных записей с `runtime=live_blocked`).
- `Degraded` — работает в ограниченном режиме.
- `Error` — произошла ошибка.

Метод `as_str()` возвращает строковое представление: `"stopped"`, `"running"`, `"blocked"`, `"degraded"`, `"error"`.

Состояние actor'а хранится в структуре `TelegramRuntimeActorState`, содержащей текущий `status`, опциональную строку `last_error` и временную метку `updated_at`. Actor может создаваться с каналом команд (`with_command`) или без (`without_command`).

Дескриптор `TelegramRuntimeActorHandle` объединяет состояние и опциональный `Sender<TelegramRuntimeCommand>`.

### Команды

`enum TelegramRuntimeCommand` (файл `state.rs`) перечисляет все поддерживаемые операции. Каждый вариант содержит параметры запроса и канал `reply_tx: Sender<Result<..., TelegramError>>` для возврата результата вызывающей стороне.

| Команда | Назначение |
|---------|------------|
| `LoadChats` | Загрузить список чатов. |
| `GetChatFolders` | Получить папки чатов по списку идентификаторов. |
| `SyncHistory` | Синхронизировать историю сообщений (поддерживает режимы `Older`/`Newer`). |
| `SendText` | Отправить текстовое сообщение. |
| `SendMedia` | Отправить медиа-сообщение. |
| `DownloadFile` | Скачать файл из Telegram. |
| `EditMessage` | Редактировать сообщение. |
| `DeleteMessage` | Удалить сообщение. |
| `SetReaction` | Установить или снять реакцию на сообщение. |
| `PinMessage` | Закрепить или открепить сообщение. |
| `ToggleChatUnread` | Переключить отметку «непрочитан». |
| `ToggleChatArchive` | Архивировать или разархивировать чат. |
| `ToggleChatMute` | Включить или выключить беззвучный режим. |
| `AddChatToFolder` | Добавить чат в папку. |
| `RemoveChatFromFolder` | Удалить чат из папки. |
| `JoinChat` | Вступить в чат. |
| `LeaveChat` | Покинуть чат. |
| `ReplyMessage` | Ответить на сообщение. |
| `ForwardMessage` | Переслать сообщение. |
| `GetForumTopics` | Получить список тем форума. |
| `CreateForumTopic` | Создать тему форума. |
| `ToggleForumTopicClosed` | Закрыть или открыть тему форума. |
| `GetSupergroupMembers` | Получить участников супергруппы. |
| `GetSupergroupAdministrators` | Получить администраторов супергруппы. |
| `GetBasicGroupMembers` | Получить участников базовой группы. |
| `SearchMessages` | Глобальный поиск сообщений. |
| `SearchChatMessages` | Поиск сообщений в конкретном чате. |

Каждая команда возвращает результат через `reply_tx`, обёрнутый в `Result<..., TelegramError>`.

### События

`enum TelegramRuntimeEvent` (файл `state.rs`) представляет события, которые рантайм генерирует для оповещения других компонентов системы:

- `MessageCreated(TelegramTdlibMessageSnapshot)` — новое сообщение.
- `MessageContentUpdated(TelegramTdlibMessageContentSnapshot)` — содержимое сообщения изменилось.
- `MessageEdited(TelegramTdlibMessageEditedSnapshot)` — сообщение отредактировано.
- `MessagePinnedUpdated(TelegramTdlibMessagePinnedSnapshot)` — сообщение закреплено или откреплено.
- `MessageDeleted(TelegramTdlibMessageDeleteSnapshot)` — сообщения удалены.
- `MessageInteractionInfoUpdated(TelegramTdlibMessageInteractionInfoSnapshot)` — обновлена информация о взаимодействиях.
- `TypingChanged(TelegramTdlibTypingSnapshot)` — изменился статус набора текста.
- `TopicUpdated(TelegramTdlibTopicUpdateSnapshot)` — обновлена тема форума.
- `ChatUnreadUpdated(TelegramTdlibChatUnreadSnapshot)` — обновлена информация о непрочитанных.
- `ChatMarkedAsUnreadUpdated(TelegramTdlibChatMarkedAsUnreadSnapshot)` — чат отмечен как непрочитанный.
- `ChatNotificationSettingsUpdated(TelegramTdlibChatNotificationSettingsSnapshot)` — изменились настройки уведомлений чата.
- `ChatPositionUpdated(TelegramTdlibChatPositionSnapshot)` — изменилась позиция чата в списке.
- `ChatRemovedFromList(TelegramTdlibChatRemovedFromListSnapshot)` — чат удалён из списка.
- `ChatFoldersUpdated(Vec<TelegramTdlibChatFolderSnapshot>)` — обновились папки чатов.

### Загрузка учётной записи и определение статуса

Файл `status.rs` содержит логику получения учётной записи провайдера и формирования структуры `TelegramRuntimeStatus`.

- **`load_telegram_account`** — загружает `ProviderAccount` по `account_id`, валидирует, что идентификатор не пуст (через `validate_non_empty`), и что `provider_kind` относится к Telegram (`is_telegram()`).
- **`account_runtime_kind`** — определяет режим работы (`runtime_kind`) из поля `config.runtime` учётной записи. Если поле отсутствует, возвращает `"unknown"` для `TelegramUser`/`TelegramBot` и `"unsupported"` для остальных.
- **`tdjson_probe`** — проверяет доступность нативной библиотеки TDLib через `TdJsonLibrary::load`.
- **`status_from_account`** — собирает итоговый `TelegramRuntimeStatus`, включая:
  - `runtime_kind`
  - `status` (из actor_state или `default_state_for_runtime`)
  - флаги доступности библиотеки, наличия `telegram_api_id` / `telegram_api_hash`
  - `live_send_available` — `true`, если `runtime_kind == "tdlib_qr_authorized"`, actor в состоянии `Running`, библиотека доступна и учётные данные Telegram-приложения заданы
  - `runtime_blockers` — список причин, по которым рантайм не может работать. Блокирует: `"live_tdlib_runtime_blocked"`, `"tdjson_runtime_unavailable"`, `"telegram_api_id_missing"`, `"telegram_api_hash_missing"`, а также последнюю ошибку, если она есть.
- **`default_state_for_runtime`** — возвращает `Blocked` для `"live_blocked"`, иначе `Stopped`.

### Валидация входных данных

Файл `validation.rs` предоставляет две вспомогательные функции:

- `validate_non_empty(field, value)` — обрезает строку и возвращает `InvalidRequest`, если она пуста.
- `validate_limit(limit)` — проверяет, что `limit` находится в диапазоне [1, 100], иначе возвращает `InvalidRequest`.

## Клиент TDLib (tdjson)

Модуль `tdjson` (файл `tdjson.rs` и вложенные подмодули) обеспечивает загрузку нативной библиотеки `libtdjson`, низкоуровневое взаимодействие через C ABI и парсинг ответов в типизированные структуры Rust.

### Загрузка библиотеки и экземпляр клиента

Файл `tdjson/client.rs` определяет:

- **`TdJsonLibrary`** — обёртка над динамической библиотекой. Метод `load(configured_path)` получает список кандидатов через `tdjson_library_candidates` (см. `library_paths.rs`), пытается загрузить библиотеку и разрешить символы: `td_json_client_create`, `td_json_client_send`, `td_json_client_receive`, `td_json_client_execute`, `td_json_client_destroy`. При ошибке возвращает `TelegramError::TdlibRuntimeUnavailable` с перечнем попыток.
- **`TdJsonClient`** — активный экземпляр клиента. Предоставляет методы:
  - `send_json` — отправляет JSON-запрос (преобразует в CString, вызывает `send`).
  - `receive_json(timeout_seconds: f64)` — получает ответ с таймаутом, возвращает `Option<Value>`.
  - `execute_json` — синхронно выполняет запрос и возвращает ответ.
  - При дропе вызывается `destroy` для освобождения ресурса.
- **`runtime_available`** — проверяет возможность загрузки библиотеки без создания клиента.

### Поиск библиотеки

Файл `tdjson/library_paths.rs` формирует список путей-кандидатов для загрузки `libtdjson` с учётом платформы (macOS, Linux, Windows) и архитектуры (arm64, x86_64). Приоритет:

1. Путь, указанный в конфигурации.
2. Пути относительно директории исполняемого файла (бандл Tauri).
3. Пути относительно текущей рабочей директории.
4. Стандартные системные пути (Homebrew, `/usr/local/lib`, `/usr/lib` и т.д.).

В ресурсных директориях ищется файл с именем, зависящим от ОС (`libtdjson.dylib`, `libtdjson.so`, `tdjson.dll`) в поддиректории, соответствующей платформе и архитектуре (например, `macos-arm64`). Функция `tdjson_platform_dir` возвращает каталог платформы, а `tdjson_library_file_name` — имя файла библиотеки.

### Парсинг ответов TDLib

Модуль `tdjson/parsing` содержит набор функций для разбора JSON-ответов от TDLib в типизированные snapshot-структуры.

#### Чаты (`chats.rs`)

- `parse_tdlib_chat_ids` — извлекает массив `chat_ids` из ответа TDLib.
- `parse_tdlib_chat_snapshot` — создаёт `TelegramTdlibChatSnapshot` с полями: `provider_chat_id`, `chat_kind` (определяется по `type.@type`: `chatTypePrivate`/`chatTypeSecret` → `Private`, `chatTypeBasicGroup` → `Group`, `chatTypeSupergroup` → `Group` или `Channel` в зависимости от `is_channel`), `title`, `username` (предпочитает поле `username`, иначе первый активный из `usernames.active_usernames`), `last_message_at` (дата последнего сообщения).

#### Сообщения (`messages.rs`, `message_parts.rs`, `message_events.rs`)

- `parse_tdlib_message_snapshot` — разбор одного сообщения в `TelegramTdlibMessageSnapshot`:
  - `provider_chat_id`, `provider_message_id`
  - `sender_id` и `sender_display_name` — через `tdlib_message_sender` (различает `messageSenderUser` → `"user:{id}"` и `messageSenderChat` → `"chat:{id}"`)
  - `text` — извлекается из `content.text` для текстовых или `content.caption` для медиа-сообщений
  - `occurred_at` — Unix-время отправки
  - `delivery_state` — `Sent` для исходящих (`is_outgoing == true`), иначе `Received`
- `parse_tdlib_message_list` — разбор массива сообщений из ответа `getChatHistory`.
- Событийные парсеры:
  - `parse_tdlib_new_message_snapshot` — событие `updateNewMessage`.
  - `parse_tdlib_message_delete_snapshot` — `updateDeleteMessages` (возвращает `provider_chat_id`, `provider_message_ids`, флаги `is_permanent` и `from_cache`).
  - `parse_tdlib_message_interaction_info_snapshot` — `updateMessageInteractionInfo`.
  - `parse_tdlib_message_content_snapshot` — `updateMessageContent` (возвращает новое содержимое и текст).
  - `parse_tdlib_message_edited_snapshot` — `updateMessageEdited` (возвращает `edit_timestamp` и опциональную `reply_markup`).
  - `parse_tdlib_message_pinned_snapshot` — `updateMessageIsPinned` (возвращает `is_pinned`).

#### События чатов и уведомлений (`events.rs`)

- `parse_tdlib_typing_snapshot` — `updateUserChatAction` (возвращает `provider_chat_id`, `provider_thread_id`, `sender_id`, `action`, `is_active`).
- `parse_tdlib_topic_update_snapshot` — `updateForumTopicInfo` (парсит вложенную `TelegramTdlibTopicSnapshot`).
- `parse_tdlib_chat_unread_snapshot` — обрабатывает `updateChatReadInbox` и `updateChatUnreadMentionCount`.
- `parse_tdlib_chat_marked_as_unread_snapshot` — `updateChatIsMarkedAsUnread`.
- `parse_tdlib_chat_notification_settings_snapshot` — `updateChatNotificationSettings` (возвращает `use_default_mute_for` и `mute_for`).
- `parse_tdlib_chat_position_snapshot` — `updateChatPosition` (поддерживает списки `main`, `archive`, `folder` с извлечением `provider_folder_id`).
- `parse_tdlib_chat_removed_from_list_snapshot` — `updateChatRemovedFromList` (фрагмент исходного кода обрезан, детали не полностью подтверждены).
- `authorization_state`, `is_tdlib_parameters_not_specified_error`, `is_tdlib_database_encryption_key_needed_error`, `tdlib_error_message` — вспомогательные функции для обработки ошибок и состояний авторизации.

#### Файлы (`files.rs`)

- `parse_tdlib_file_snapshot` — разбирает объект `file` в `TelegramTdlibFileSnapshot` с деталями загрузки (`local_path`, `is_downloading_active`, `is_downloading_completed`, `downloaded_size_bytes`), размерами (`size_bytes`, `expected_size_bytes`) и удалёнными идентификаторами (`remote_id`, `remote_unique_id`).

#### Участники (`participants.rs`)

- `parse_tdlib_chat_member_list` — разбор списка участников из ответа `chatMembers`.
- `parse_tdlib_basic_group_member_list` — разбор списка участников из ответа `basicGroupFullInfo`.
- Каждый участник преобразуется в `TelegramTdlibChatMemberSnapshot` с полями:
  - `provider_member_id` — `"user:{id}"` или `"chat:{id}"` в зависимости от `messageSenderUser` / `messageSenderChat`
  - `display_name`, `username`
  - `role` — `"owner"`, `"admin"`, `"restricted"`, `"banned"`, `"left"`, `"member"`, `"unknown"`
  - `status` — строковое представление статуса без префикса `chatMemberStatus`
  - `is_admin`, `is_owner`
  - `permissions` — объект, содержащий все поля статуса, кроме `@type`

#### Темы форумов (`topics.rs`)

- `parse_tdlib_topic_list` — разбор массива тем из ответа `forumTopics`.
- `parse_tdlib_created_forum_topic` — разбор информации о созданной теме.
- `parse_forum_topic_info` — создаёт `TelegramTdlibTopicSnapshot` с полями: `provider_topic_id`, `title`, `icon_emoji`, `is_pinned`, `is_closed`, `unread_count`, `last_message_at`.

#### Вспомогательные утилиты (`values.rs`)

- `tdlib_string_id` — извлекает числовое поле из JSON и преобразует в строку.
- `tdlib_i64_value` — извлекает `i64` (допускает и `u64` с проверкой приведения).
- `tdlib_unix_datetime_value` — преобразует Unix-таймстемп в `DateTime<Utc>`.

### Редактирование папок чатов

Файл `folder_requests.rs` содержит функцию `tdlib_edit_chat_folder_remove_chat_request`, которая формирует запрос `editChatFolder` для удаления чата из папки. Она принимает `chat_folder_id`, `chat_id`, текущий снапшот папки и строку `extra`. Проверяет, что снапшот имеет `@type: "chatFolder"`, затем перестраивает списки `pinned_chat_ids` и `included_chat_ids` (удаляя указанный `chat_id`) и `excluded_chat_ids` (добавляя `chat_id`, если его ещё нет). Все остальные свойства папки (имя, иконка, цвет, флаги) копируются из снапшота.

### Безопасные идентификаторы

Файл `identifiers.rs` предоставляет `safe_path_segment(value: &str) -> String`, преобразующую строку в безопасный сегмент файлового пути: символы вне `[a-zA-Z0-9]` заменяются на дефис, результирующая строка обрезается по краевым дефисам. Если результат пуст, возвращается `"account"`.

## QR-логин Telegram

Подсистема `tdjson/qr_login` реализует процесс авторизации через QR-код с использованием TDLib.

### Запуск сессии

`start_qr_login` (файл `qr_login/driver.rs`) принимает конфигурацию (`AppConfig`), общую карту активных сессий (`PendingQrLoginMap`) и запрос `TelegramQrLoginStartRequest`. Валидирует запрос, загружает TDLib-библиотеку, отменяет предыдущие сессии для той же учётной записи, создаёт канал команд и `worker_completion`, генерирует `setup_id` (на основе `account_id`) и помещает ответ со статусом `Preparing` в общую карту. Затем в отдельном потоке (имя `telegram-qr-login-{suffix}`) запускает `drive_qr_login`.

### Основной цикл воркера

`drive_qr_login` (файл `qr_login/worker.rs`) создаёт клиент TDLib, настраивает директорию базы данных, отправляет `getAuthorizationState`. В цикле с периодичностью 1 секунда:

1. Обрабатывает входящие команды через `drain_qr_login_commands` (ввод пароля или отмена).
2. Проверяет таймауты: если QR-ссылка не получена за `QR_FIRST_LINK_TIMEOUT` или общее время сессии превысило `QR_SESSION_LIFETIME`, сессия завершается с ошибкой `Failed` или `Expired`.
3. Получает события от TDLib через `receive_json(1.0)` и передаёт в `handle_qr_login_event`.

### Обработка событий авторизации

`handle_qr_login_event` (файл `qr_login/authorization.rs`) диспетчеризует входящие TDLib-события:

- Сначала проверяет ошибки настройки (отсутствие параметров или ключа шифрования базы данных) через `handle_tdlib_setup_event`.
- Затем ошибки TDLib: если пароль был отправлен и отклонён, уведомляет о необходимости повторного ввода.
- Затем извлекает `authorization_state` и действует по типу:
  - `authorizationStateWaitTdlibParameters` → отправляет параметры клиента (`send_tdlib_parameters`).
  - `authorizationStateWaitEncryptionKey` → отправляет запрос проверки ключа базы данных.
  - Состояния, допускающие QR-запрос (определяется `state_allows_qr_request`) → отправляет `requestQrCodeAuthentication`.
  - `authorizationStateWaitOtherDeviceConfirmation` → извлекает `link`, формирует ответ с QR-кодом (`qr_waiting_response`) и сохраняет в карту активных сессий. Помечает `qr_link_issued = true`.
  - `authorizationStateWaitPassword` → устанавливает статус `WaitingPassword` с подсказкой пароля (если TDLib предоставил `password_hint`).
  - `authorizationStateReady` → получает идентификационные данные пользователя (`fetch_authorized_user_identity`), устанавливает статус `Ready`, закрывает сессию TDLib.
  - `authorizationStateClosed`, `authorizationStateClosing`, `authorizationStateLoggingOut` → завершает сессию с ошибкой.
  - Неподдерживаемое состояние после успешного QR-запроса → ошибка.

### Управление паролем и отмена

- `submit_qr_login_password` (файл `qr_login/commands.rs`) проверяет, что сессия находится в статусе `WaitingPassword`, и отправляет команду `CheckPassword` в воркер. Обновляет сообщение в ответе на `"Checking Telegram password."`.
- `cancel_qr_login` удаляет сессию из карты, отправляет команду `Cancel` и ожидает завершения воркера.
- `cancel_existing_qr_logins_for_account` отменяет все сессии для заданной учётной записи.

### Команды воркера

`drain_qr_login_commands` (файл `qr_login/tdlib_commands.rs`) неблокирующе читает канал команд. При получении `CheckPassword` отправляет TDLib-запрос `checkAuthenticationPassword`; при `Cancel` отправляет `close`. Возвращает признак того, была ли отправлена команда пароля.

Функции `send_tdlib_parameters` и `close_tdlib_session` отправляют соответствующие TDLib-запросы.
```

### Source coverage / Покрытие источников

- `backend/src/integrations/telegram/runtime/state.rs`:
  - Enum `TelegramRuntimeState` (5 состояний, метод `as_str`)
  - Структуры `TelegramRuntimeActorState`, `TelegramRuntimeActorHandle`
  - Enum `TelegramRuntimeCommand` (все варианты команд)
  - Enum `TelegramRuntimeEvent` (все варианты событий)

- `backend/src/integrations/telegram/runtime/status.rs`:
  - `load_telegram_account` (загрузка учётной записи, проверка провайдера)
  - `status_from_account` (сбор `TelegramRuntimeStatus`, вычисление `live_send_available`, `runtime_blockers`)
  - `tdjson_probe` (проверка библиотеки)
  - `runtime_blockers` (список блокировок)
  - `default_state_for_runtime` (Blocked / Stopped)
  - `account_runtime_kind` (определение режима)

- `backend/src/integrations/telegram/runtime/tests.rs`:
  - Тесты `history_sync_request_accepts_older_cursor` и `history_sync_response_exposes_next_cursor` (подтверждают сериализацию и валидацию запросов/ответов синхронизации)

- `backend/src/integrations/telegram/runtime/validation.rs`:
  - `validate_non_empty` (проверка непустых строк)
  - `validate_limit` (ограничение 1..100)

- `backend/src/integrations/telegram/tdjson.rs`:
  - Объявление модулей: client, folder_requests, identifiers, library_paths, parsing, qr_login, qr_login_support, requests, snapshots
  - Реэкспорт основных типов и функций

- `backend/src/integrations/telegram/tdjson/client.rs`:
  - `TdJsonLibrary` (загрузка, создание клиента, список символов)
  - `TdJsonClient` (send / receive / execute / Drop)
  - `runtime_available`

- `backend/src/integrations/telegram/tdjson/folder_requests.rs`:
  - `tdlib_edit_chat_folder_remove_chat_request` (построение запроса удаления чата из папки)

- `backend/src/integrations/telegram/tdjson/identifiers.rs`:
  - `safe_path_segment`

- `backend/src/integrations/telegram/tdjson/library_paths.rs`:
  - `tdjson_library_candidates`, `tdjson_library_candidates_with_context` (список путей)
  - `tdjson_platform_dir`, `tdjson_library_file_name` (имена платформ и библиотек)
  - Логика поиска для macOS, Linux, Windows

- `backend/src/integrations/telegram/tdjson/parsing.rs`:
  - Структура модуля (chats, events, files, message_events, message_parts, messages, participants, topics, values)

- `backend/src/integrations/telegram/tdjson/parsing/chats.rs`:
  - `parse_tdlib_chat_ids`, `parse_tdlib_chat_snapshot` (определение `chat_kind`, `username`, `title`, `last_message_at`)

- `backend/src/integrations/telegram/tdjson/parsing/events.rs` (частично обрезан):
  - Структуры snapshot-событий (ChatUnread, ChatMarkedAsUnread, ChatNotificationSettings, ChatPosition, ChatRemovedFromList, TopicUpdate, Typing, ChatFoldersUpdate)
  - `authorization_state`, `is_tdlib_parameters_not_specified_error`, `is_tdlib_database_encryption_key_needed_error`, `tdlib_error_message`
  - `parse_tdlib_typing_snapshot`, `parse_tdlib_topic_update_snapshot`
  - `parse_tdlib_chat_unread_snapshot`, `parse_tdlib_chat_marked_as_unread_snapshot`, `parse_tdlib_chat_notification_settings_snapshot`, `parse_tdlib_chat_position_snapshot` (подтверждено наполовину; детали `ChatRemovedFromList` и `ChatFoldersUpdated` не видны полностью)

- `backend/src/integrations/telegram/tdjson/parsing/files.rs`:
  - `parse_tdlib_file_snapshot` (поля файла)

- `backend/src/integrations/telegram/tdjson/parsing/message_events.rs`:
  - Все 6 парсеров событий сообщений и их модульные тесты

- `backend/src/integrations/telegram/tdjson/parsing/message_parts.rs`:
  - `tdlib_message_sender` (определение отправителя)
  - `tdlib_message_text` (извлечение текста)

- `backend/src/integrations/telegram/tdjson/parsing/messages.rs`:
  - `parse_tdlib_message_list`, `parse_tdlib_message_snapshot` (разбор сообщения)

- `backend/src/integrations/telegram/tdjson/parsing/participants.rs`:
  - `parse_tdlib_chat_member_list`, `parse_tdlib_basic_group_member_list`, `parse_chat_member` (роли, разрешения)

- `backend/src/integrations/telegram/tdjson/parsing/topics.rs`:
  - `parse_tdlib_topic_list`, `parse_tdlib_created_forum_topic`, `parse_forum_topic_info`

- `backend/src/integrations/telegram/tdjson/parsing/values.rs`:
  - `tdlib_string_id`, `tdlib_i64_value`, `tdlib_unix_datetime_value`

- `backend/src/integrations/telegram/tdjson/qr_login.rs`:
  - Подмодули authorization, commands, driver, tdlib_commands, worker, worker_state

- `backend/src/integrations/telegram/tdjson/qr_login/authorization.rs`:
  - `handle_qr_login_event` (диспетчеризация состояний авторизации)
  - `handle_tdlib_error`, `handle_authorization_state`, `handle_wait_other_device_confirmation`, `handle_wait_password`, `handle_ready` (логика каждого шага)

- `backend/src/integrations/telegram/tdjson/qr_login/commands.rs`:
  - `submit_qr_login_password`, `cancel_qr_login`, `cancel_existing_qr_logins_for_account`

- `backend/src/integrations/telegram/tdjson/qr_login/driver.rs`:
  - `start_qr_login`, `start_qr_login_driver` (запуск потока воркера)

- `backend/src/integrations/telegram/tdjson/qr_login/tdlib_commands.rs`:
  - `drain_qr_login_commands`, `send_tdlib_parameters`, `close_tdlib_session`

- `backend/src/integrations/telegram/tdjson/qr_login/worker.rs`:
  - `drive_qr_login` (основной цикл), `drain_worker_commands`, `expire_stale_session`, `handle_tdlib_setup_event`

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений между кодом и документацией (дрифт) не видно, так как отсутствует текущее содержимое целевой wiki-страницы `components/backend.md`. Сравнение возможно только при наличии и исходников, и актуальной версии страницы.
