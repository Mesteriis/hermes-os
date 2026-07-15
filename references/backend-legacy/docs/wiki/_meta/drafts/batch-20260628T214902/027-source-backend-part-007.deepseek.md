## Summary / Резюме

Создать страницу `components/backend.md` в русской Obsidian‑вики, описывающую модули‑обработчики бэкенда (calendar, communications, calls) на основе предоставленных исходных файлов. Страница должна перечислить ключевые эндпоинты каждого подмодуля, их зависимости от доменных сервисов и особенности (требования к БД, блокировки хранилища, форматы экспорта/импорта, известные ограничения). Факты строго привязаны к встроенному коду.

## Proposed pages / Предлагаемые страницы

### `components/backend.md`

```markdown
# Компоненты бэкенда

## Calendar (Календарь)

### Модуль `calendar`

ADR-0073: обработчики календаря разделены по задокументированным обязанностям домена Calendar.

Включает подмодули: `accounts`, `analytics`, `brain`, `events`, `health`, `intelligence`, `meetings`, `reminders`, `rules`, `scheduling`, `search`, `sync`.

#### health

- `get_calendar_watchtower` – возвращает события, требующие подготовки (`preparation`) и события без исходов (`without_outcomes`) через `CalendarWatchtowerService`.
- `get_calendar_health` – возвращает анализ загрузки встреч (`meeting_load_analysis`) через `CalendarWatchtowerService`.

#### intelligence

- `post_event_classify` – классифицирует событие по названию, количеству участников и длительности; обновляет `event_type` через `CalendarIntelligenceService::classify_event`.
- `post_event_analyze` – вычисляет `importance`, `readiness` (через `CalendarIntelligenceService::calculate_importance`, `calculate_readiness`) и риски (`detect_risks`); обновляет `importance_score` и `readiness_score`.
- `get_event_risks` – возвращает риски для события.

#### meetings

- `get_meeting_notes` – список заметок встречи.
- `post_meeting_note` – создание заметки вручную (источник `manual`, `CalendarCommandService::create_meeting_note_manual`).
- `get_meeting_outcomes` – список исходов встречи.
- `post_meeting_outcome` – добавление исхода через `CalendarMeetingOutcomeApplicationService::add_manual`.
- `post_event_follow_up` – устанавливает статус `needs_follow_up`.
- `get_event_follow_up_status` – получает статус follow-up по исходам.
- `get_event_recordings` – список записей встречи.
- `post_event_recording` – добавление записи (источник по умолчанию `manual`) через `CalendarCommandService::add_event_recording_manual`.
- `get_event_transcript` – получение транскрипта встречи.

#### reminders

- `get_event_reminders` – список напоминаний.
- `post_event_reminder` – создание напоминания вручную.
- `post_event_reminder_toggle` – включение/отключение напоминания.

#### rules

- `get_calendar_rules` – список всех правил календаря.
- `post_calendar_rule` – создание правила с DSL и режимом подтверждения (`approval_mode`).
- `put_calendar_rule` – обновление правила.
- `delete_calendar_rule` – удаление правила.

#### scheduling

- `get_deadlines` – список дедлайнов с фильтрацией по статусу.
- `post_deadline` – создание дедлайна вручную (с указанием `severity`, `source_entity_type`, `source_entity_id`).
- `get_focus_blocks` – список focus-блоков с фильтрацией по `from`/`to`.
- `post_focus_block` – создание focus-блока вручную (с `purpose`, `linked_project_id`, `protection_level`).
- `post_smart_schedule` – поиск свободных слотов через `SmartSchedulingService::find_slots` (длительность по умолчанию 30 мин, горизонт 48 ч).

#### search

- `get_calendar_search` – поиск событий через `CalendarBrainService::search_events`.

#### sync

- `post_calendar_import` – импорт событий из массива `events` (каждый элемент обрабатывается через `CalendarEventStore::create_file_import`); также отмечает факт получения ICS-данных (`ics_data`).  
- `post_calendar_sync` – запуск ручной синхронизации аккаунта; в ответе явно указано, что провайдер‑синхронизация отложена на будущее.  
- `get_event_export` – экспорт события в формате `json`, `ics` или `md` (Markdown).

#### Другие подмодули

Подмодули `accounts`, `analytics`, `brain` и `events` не включены в данный контекстный пакет; их детали не подтверждены.

---

## Communications (Коммуникации)

### Модуль `communications`

Включает подмодули: `account_management`, `account_setup`, `account_support`, `communication_messages`, `communication_queries`.

#### account_management

Управление email-аккаунтами:

- `get_v1_email_accounts` – список всех email-аккаунтов (фильтр по `provider_kind`).  
- `get_v1_email_account` – детали одного аккаунта.  
- `get_v1_email_account_export` – экспорт аккаунта с возможностями и настройками синхронизации; секретные ключи конфигурации удалены (`sanitize_account_config`).  
- `post_v1_email_account_import` – импорт аккаунта с синхронизацией Signal Hub; запрещает наличие секретных материалов в payload.  
- `post_v1_email_account_logout` – выход из аккаунта, отключение синхронизации и установка статуса Signal Hub `disconnected`.  
- `delete_v1_email_account` – удаление аккаунта после проверки использования (если есть retaining evidence, возвращает `EmailAccountDeleteConflict`).  
- `get_v1_email_account_sync_status` – статусы синхронизации всех аккаунтов.  
- `get_v1_email_account_sync_settings` / `put_v1_email_account_sync_settings` – просмотр и изменение настроек синхронизации.  
- `post_v1_email_account_sync_now` / `post_v1_email_account_sync_full_resync` – ручной запуск синхронизации аккаунта.

#### account_setup

Настройка новых email-аккаунтов:

- **Gmail OAuth**  
  - `post_gmail_oauth_start` – начало OAuth, сохраняет `GmailOAuthPendingGrant` в `pending_gmail_oauth`.  
  - `post_gmail_oauth_complete` – завершение OAuth с кодом авторизации; после успеха синхронизирует Signal Hub и создаёт аккаунт Google Workspace Calendar (`upsert_google_workspace_calendar_account`).  
  - `get_gmail_oauth_callback` – обработка коллбэка от Google; возвращает HTML‑страницу с подтверждением или ошибкой, автоматически закрывает окно.  
- **IMAP**  
  - `post_imap_account_setup` – настройка IMAP‑аккаунта; если провайдер `icloud`, дополнительно создаётся аккаунт Apple iCloud Calendar (`upsert_apple_icloud_calendar_account`).

#### account_support

Вспомогательные типы и утилиты, используемые обработчиками:

- Типы ответов: `EmailAccountListResponse`, `EmailAccountView`, `EmailAccountCapabilities` (read, sync, send, oauth, imap, smtp и т.д.), `EmailAccountExportResponse`, `EmailAccountLogoutResponse`, `EmailAccountDeleteResponse`.  
- `email_account_or_not_found` – поиск email-аккаунта.  
- `email_account_capabilities` – вычисление возможностей на основе конфигурации (учёт `auth_state == "logged_out"`, наличия SMTP и OAuth‑полей).  
- `sanitize_account_config` – удаление секретных ключей из конфигурации (список маркеров: `password`, `secret`, `token`, `credential` и т.д.).  
- `contains_secret_material` – проверка наличия секретных полей в произвольном JSON.  
- `require_unlocked_host_vault` – требует разблокированное хранилище (`VaultMode::Unlocked`).  
- `mail_sync_store`, `mail_sync_service` – получение хранилища и сервиса фоновой синхронизации почты.

#### communication_messages

Получение сообщений:

- `get_v1_communication_messages` – постраничный список сообщений с фильтрацией по `account_id`, `workflow_state`, `channel_kind`, `conversation_id`, поисковому запросу и `local_state`.  
- `get_v1_communication_message` – детали сообщения с телом в HTML (если blob доступен локально), вложениями и сырыми заголовками.  
- `rich_email_message_detail_for_message` – извлекает `body_html` и `headers` из сырой записи, только если `raw_blob_storage_kind == "local_fs"`.

#### communication_queries

Запросы к черновикам, папкам и вложениям:

- **drafts**  
  - `get_v1_drafts` – пагинированный список черновиков.  
  - `post_v1_draft` – создание или обновление черновика.  
  - `get_v1_draft` – получение черновика по `draft_id`.  
  - `delete_v1_draft` – удаление черновика.  
- **folders**  
  - `get_v1_mail_folders` – список папок.  
  - `post_v1_mail_folder` – создание папки.  
  - `put_v1_mail_folder` – обновление папки.  
  - `delete_v1_mail_folder` – удаление папки.  
  - `get_v1_mail_folder_messages` – сообщения в папке.  
  - `post_v1_copy_message_to_folder` / `post_v1_move_message_to_folder` – копирование/перемещение сообщений.  
- **attachments** (частично, исходник обрезан)  
  - `get_v1_attachment_search` – поиск вложений.  
  - `get_v1_attachment_preview` – превью вложений (текст, изображения, аудио, видео, PDF) с кодеком base64 data URL и явными ограничениями размера (`MAX_*_PREVIEW_BYTES`).  
  - `get_v1_attachment_archive_inspection` – инспекция ZIP‑архивов (глубина по умолчанию).

---

## Calls (Звонки)

### Модуль `calls`

- `post_call` – добавление звонка (`CallIntelligenceStore::upsert_call`).  
- `get_calls` – список звонков с фильтрацией по `account_id`, `provider_chat_id`, `provider` и лимитом.  
- `post_call_transcript_fixture` – создание транскрипта из локальной фикстуры (`FixtureSpeechToTextProvider`) с пометкой `provenance.runtime = "fixture"`.  
- `get_call_transcript` – получение транскрипта звонка.

---

## Используемые домены и службы

Обработчики опираются на следующие доменные сервисы и хранилища (основные):

- Calendar: `CalendarWatchtowerService`, `CalendarIntelligenceService`, `CalendarCommandService`, `CalendarMeetingOutcomeApplicationService`, `SmartSchedulingService`, `CalendarBrainService`.  
- Communications: `CommunicationCommandService`, `MessageProjectionStore`, `CommunicationFolderStore`, `CommunicationDraftStore`, `AttachmentSearchStore`, `MailBackgroundSyncService`, `EmailAccountSetupService`.  
- Calls: `CallIntelligenceStore`, `FixtureSpeechToTextProvider`.  
- Инфраструктура: `ApiError`, `AppState`, `Database`, `HostVault`.

## Примечания

- Все обработчики требуют сконфигурированную БД (`pool.ok_or(ApiError::DatabaseNotConfigured)`).  
- Импорт календаря поддерживает JSON (массив событий) и факт получения ICS‑данных.  
- Экспорт календаря – JSON (по умолчанию), ICS или Markdown.  
- Комментарий в `post_calendar_sync` явно сообщает: «Provider sync is deferred to future implementation».  
- При настройке почтовых аккаунтов проверяется разблокировка локального хранилища (`require_unlocked_host_vault`).  
- Секретные ключи конфигурации почтовых аккаунтов редактируются при экспорте; импорт не должен содержать секретного материала.  
- Файл `attachments.rs` обрезан после 12000 символов – полный набор обработчиков вложений не подтверждён.

---

*Данная страница основана исключительно на встроенных исходных файлах. Подмодули `calendar/accounts`, `calendar/analytics`, `calendar/brain`, `calendar/events` не раскрыты; их поведение не подтверждено.*
```

## Source coverage / Покрытие источников

| Source file | Covered facts |
|---|---|
| `backend/src/app/handlers/calendar/mod.rs` | Состав подмодулей календаря по ADR-0073, re‑exports, импорт всех используемых типов/сервисов. |
| `backend/src/app/handlers/calendar/health.rs` | `get_calendar_watchtower`, `get_calendar_health`, использование `CalendarWatchtowerService`. |
| `backend/src/app/handlers/calendar/intelligence.rs` | `post_event_classify` (классификация, обновление `event_type`), `post_event_analyze` (расчёт importance/readiness/risks, обновление скорингов), `get_event_risks`. |
| `backend/src/app/handlers/calendar/meetings.rs` | CRUD встреч: заметки (`get_meeting_notes`, `post_meeting_note`), исходы (`get_meeting_outcomes`, `post_meeting_outcome`), follow‑up (`post_event_follow_up`, `get_event_follow_up_status`), записи (`get_event_recordings`, `post_event_recording`), транскрипт (`get_event_transcript`). |
| `backend/src/app/handlers/calendar/reminders.rs` | `get_event_reminders`, `post_event_reminder`, `post_event_reminder_toggle`. |
| `backend/src/app/handlers/calendar/rules.rs` | `get_calendar_rules`, `post_calendar_rule` (с DSL и approval_mode), `put_calendar_rule`, `delete_calendar_rule`. |
| `backend/src/app/handlers/calendar/scheduling.rs` | `get_deadlines`, `post_deadline`, `get_focus_blocks`, `post_focus_block`, `post_smart_schedule`. |
| `backend/src/app/handlers/calendar/search.rs` | `get_calendar_search` через `CalendarBrainService`. |
| `backend/src/app/handlers/calendar/sync.rs` | `post_calendar_import` (JSON‑массив событий, ICS‑data), `post_calendar_sync` (ручной триггер с пометкой об отложенной реализации), `get_event_export` (форматы json/ics/md). |
| `backend/src/app/handlers/communications/account_management.rs` | Управление email‑аккаунтами: список, экспорт, импорт, logout, удаление, статус/настройки синхронизации, ручной запуск синхронизации. |
| `backend/src/app/handlers/communications/account_setup.rs` | Реэкспорт обработчиков настройки (`post_gmail_oauth_start`, `post_gmail_oauth_complete`, `get_gmail_oauth_callback`, `post_imap_account_setup`) и вспомогательных модулей. |
| `backend/src/app/handlers/communications/account_setup/calendar.rs` | `upsert_google_workspace_calendar_account`, `upsert_apple_icloud_calendar_account` – создание календарных аккаунтов с привязкой к secret_ref. |
| `backend/src/app/handlers/communications/account_setup/gmail_callback.rs` | Полный flow коллбэка: проверка error/code/state, чтение pending‑состояния, вызов `complete_gmail_oauth`, синхронизация Signal Hub, создание календарного аккаунта, HTML‑страницы успеха/ошибки. |
| `backend/src/app/handlers/communications/account_setup/gmail_oauth.rs` | `post_gmail_oauth_start` (создание pending‑гранта, сохранение в `pending_gmail_oauth`), `post_gmail_oauth_complete` (завершение с проверкой state и кодом). |
| `backend/src/app/handlers/communications/account_setup/helpers.rs` | `gmail_pending_external_account_id`, `trimmed_optional`. |
| `backend/src/app/handlers/communications/account_setup/imap.rs` | `post_imap_account_setup`, автоматическое создание iCloud Calendar для провайдера `icloud`. |
| `backend/src/app/handlers/communications/account_setup/models.rs` | Модели запросов/ответов: `GmailOAuthStartApiRequest`, `ImapAccountSetupApiRequest`, их преобразование в доменные объекты; `EmailAccountSetupApiResponse`. |
| `backend/src/app/handlers/communications/account_support.rs` | Вспомогательные типы (`EmailAccountView`, `EmailAccountCapabilities`), функции: `email_account_or_not_found`, `email_account_capabilities`, `sanitize_account_config`, `contains_secret_material`, `require_unlocked_host_vault`, `mail_sync_store`, `mail_sync_service`, маппинг ошибок `mail_sync_api_error`. |
| `backend/src/app/handlers/communications/communication_messages.rs` | `get_v1_communication_messages` (курсорная пагинация, фильтрация), `get_v1_communication_message` (детали с HTML‑телом и заголовками), `rich_email_message_detail_for_message`. |
| `backend/src/app/handlers/communications/communication_queries.rs` | Реэкспорт подмодулей `drafts`, `folders`, `attachments`, `search`, `threads`, `personas`, etc. |
| `backend/src/app/handlers/communications/communication_queries/drafts.rs` | `get_v1_drafts`, `post_v1_draft`, `get_v1_draft`, `delete_v1_draft`. |
| `backend/src/app/handlers/communications/communication_queries/folders.rs` | `get_v1_mail_folders`, `post_v1_mail_folder`, `put_v1_mail_folder`, `delete_v1_mail_folder`, `get_v1_mail_folder_messages`, копирование/перемещение сообщений между папками. |
| `backend/src/app/handlers/communications/communication_queries/attachments.rs` (truncated) | `get_v1_attachment_search`, `get_v1_attachment_preview` (text/image/audio/video/pdf с data URL и лимитами размера), `get_v1_attachment_archive_inspection` (ZIP‑инспекция). |
| `backend/src/app/handlers/calls/handlers.rs` | `post_call`, `get_calls`, `post_call_transcript_fixture` (FixtureSpeechToTextProvider, provenance‑разметка), `get_call_transcript`. |
| `backend/src/app/handlers/calls/mod.rs` | Реэкспорт `handlers::*`. |

## Drift candidates / Кандидаты на drift

Из предоставленных исходных файлов расхождения между кодом и документацией не видны. Модуль `calendar/mod.rs` перечисляет подмодули `accounts`, `analytics`, `brain`, `events`, но их исходный код в данном контексте отсутствует — это не drift, а неполнота покрытия, которая явно отмечена на странице. Также комментарий в `post_calendar_sync` прямо заявляет, что провайдер‑синхронизация отложена, и это зафиксировано в вики. Никаких иных признаков расхождения кода, ADR или документации в рамках этого чанка нет.
