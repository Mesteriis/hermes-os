### Summary / Резюме

Чанк `067-source-backend-part-047` содержит исходный код нескольких интеграционных модулей бэкенда: WhatsApp, Yandex Telemost и Zoom. Страница `components/backend.md` обновляется для описания этих компонентов на основе предоставленных файлов. Включены детали реализации хранилищ, валидации, сред исполнения (runtime), клиентских HTTP-клиентов и мостов событий. Информация строго ограничена встроенными файлами; для обрезанных файлов контекст может быть неполным.

### Proposed pages / Предлагаемые страницы

- `components/backend.md`

```markdown
---
tags: [backend, components, integration]
---

# Компоненты бэкенда (чанк 047)

Документ описывает интеграционные компоненты бэкенда на основе исходного кода из чанка `067-source-backend-part-047`. Содержимое основано исключительно на предоставленных файлах; компоненты, исходный код которых не включён в чанк, не описаны. Информация, выходящая за пределы этого чанка, не подтверждена.

## Интеграция WhatsApp

Расположение: `backend/src/integrations/whatsapp/`.

Модульная структура:
- `client` — клиентское хранилище (`store`), модели, ошибки, валидация.
- `runtime` — среды исполнения (web-компаньон, native multi-device, Business Cloud), контракты, мост событий.

### Хранилище WhatsApp Web (`client/store/`)

- **`WhatsappWebStore`** реализует:
  - `recent_messages` — выборка последних сообщений по `account_id`, `provider_chat_id`, `limit` для каналов `"whatsapp_web"` и `"whatsapp_business_cloud"`. Лимит валидируется через `validate_limit`.
  - `upsert_session` — вставка или обновление сессии WhatsApp Web с `ON CONFLICT (account_id) DO UPDATE SET ...`. При сохранении записывает observation `whatsapp-web-session://{session_id}/upsert` через транзакцию.
  - `list_sessions` — список сессий с опциональным фильтром `account_id` и лимитом, сортировка `ORDER BY updated_at DESC, session_id ASC`.
  - `update_session_last_sync` — обновление `last_sync_at` только на более позднее значение (`GREATEST(COALESCE(last_sync_at, $2), $2)`). Пишет observation `sync_progress`.
  - `update_session_link_state` — обновление `link_state`, пишет observation `link_state_update`.
- Валидация (`validation.rs`):
  - `validate_limit(limit)`: допустимый диапазон `1..=100`, иначе `InvalidRequest`.
  - `validate_non_empty(field, value)`: обрезает пробелы, проверяет непустоту.
  - `validate_object(field, value)`: требует JSON-объект.
  - `validate_string_array(field, values)`: все строки должны быть непустыми после обрезки.

### Среда исполнения WhatsApp (`runtime/`)

#### Web-компаньон (`web_companion.rs`)
- `build_runtime` конструирует `ShapedWhatsAppProviderRuntime` с формой `WebCompanion`, используя `WhatsappWebStore` как внутренний провайдер.
- Функция `web_companion_bridge_contract_health_check` возвращает детальную JSON-схему состояния и контракта:
  - Драйвер: `webview_companion_bridge`, desktop-продюсер на Tauri, окно `whatsapp-companion-*`.
  - Извлечение событий: скрипт на `https://web.whatsapp.com`, передача через Tauri IPC в защищённый endpoint `/api/v1/integrations/whatsapp/runtime-bridge/runtime-events`.
  - Политика безопасности: запрещены чтения cookies, local/session storage, тел сообщений, медиа-байтов; передаются только метаданные.
  - Рантайм видим пользователю (visible owner window), headless-режим запрещён.

#### Контракты (`contracts.rs`)
- `WhatsAppProviderRuntimeShape`: перечисление `WebCompanion`, `NativeMultiDevice`, `BusinessCloud`.
- `WhatsAppSanitizedRuntimeEventDto`: санитизированное событие с контрактными полями (`account_id`, `provider_event_id`, `provider_shape`, `runtime_driver`, `event_family`, `source_fingerprint_seed` и др.). Гарантирует отсутствие `session_material` и `raw_provider_payload`.
- Запросы управления рантаймом: `WhatsAppRuntimeStartRequest`, `WhatsAppRuntimeStopRequest`, `WhatsAppRuntimeRevokeRequest`, `WhatsAppRuntimeRelinkRequest`, `WhatsAppRuntimeRemoveRequest`.
- Запросы действий: отправка текста, ответ, пересылка, редактирование, удаление, реакции, управление диалогом, публикация статуса, отправка голосового сообщения, загрузка/скачивание медиа.
- Трейт `WhatsAppProviderRuntime` и `WhatsAppRuntimeEventSink`.

#### Native Multi-Device (`native_md.rs`)
- Доступен при фиче `whatsapp-native-md-runtime`.
- Канал команд: `durable_provider_command_outbox`.
- Приёмник событий: `signal_hub_raw_evidence`.
- Константы повторного подключения: базовые/максимальные задержки (5–300 с), максимум 5 попыток.
- Артефакты транзитной аутентификации (`NativeMdTransientAuthArtifacts`): QR-код и pair-код с TTL (по умолчанию 180 с).
- Реестр жизненного цикла (`NativeMdRuntimeLifecycleRegistry`): трекает состояния старта, переподключения, остановки; отправляет синтетические события в Signal Hub.

#### Business Cloud (`business_cloud.rs`)
- Конфигурационные ключи аккаунта: `business_cloud_live_smoke_enabled`, `business_cloud_graph_api_version`, `business_cloud_phone_number_id`; версия API по умолчанию `v24.0`.
- Поддерживаемые команды: `send_text`, `send_template`, `send_media`, `send_voice_note`.
- `BusinessCloudRuntimeManager` выполняет запросы к WhatsApp Business Cloud Graph API, требует токен доступа из host-vault, выполняет загрузку медиа отдельным вызовом.
- Основная функция: `execute_live_provider_command` маршрутизирует по типу команды и вызывает соответствующие методы (`execute_send_text`, `execute_send_template`, `execute_send_media`).
- Здоровье рантайма дополняется методом `decorate_runtime_health`.

> **Примечание:** Файлы `business_cloud.rs`, `contracts.rs`, `native_md.rs` и `mod.rs` обрезаны по лимиту 12000 символов; полное описание компонентов приведено на основе доступной части.

## Интеграция Yandex Telemost

Расположение: `backend/src/integrations/yandex_telemost/`.

Модули: `client` (ошибки, модели, хранилище, валидация), `runtime`, `runtime_bridge`.

### Клиентское хранилище (`client/store.rs`)

- **`YandexTelemostHttpClient`** взаимодействует с API Яндекс Телемоста:
  - `create_conference` — создание конференции.
  - `get_conference` — получение конференции по ID.
  - `update_conference` — обновление конференции.
  - `list_cohosts` — список соорганизаторов с пагинацией.
  Все методы используют OAuth-токен в заголовке `Authorization: OAuth <token>`.

- **`YandexTelemostStore`** управляет аккаунтами и интеграционными событиями:
  - `setup_account` — регистрирует аккаунт, сохраняет токен как `SecretReference` с назначением `ProviderAccountSecretPurpose::YandexTelemostOauthToken`, привязывает секрет к аккаунту, публикует события `account_configured` и `authorization`.
  - `list_accounts` — возвращает список аккаунтов с сортировкой по имени.
  - `runtime_status` — определяет статус авторизации, генерирует `YandexTelemostRuntimeStatus` и публикует событие.
  - `cleanup_retention` — удаляет устаревшие аудиозаписи и файлы временных меток спикеров согласно политике хранения (настраивается ключами `privacy.yandex_telemost_recording_retention_days` и `privacy.yandex_telemost_speaker_timeline_retention_days`).

### Модели (`client/models.rs`)

- Базовые константы: `YANDEX_TELEMOST_PROVIDER_KIND_STR` (`"yandex_telemost_user"`), `YANDEX_TELEMOST_RUNTIME_KIND` (`"yandex_telemost_webview_runtime"`), `YANDEX_TELEMOST_API_BASE_URL` (`"https://cloud-api.yandex.net/v1/telemost-api"`).
- `YandexTelemostAccount` — аккаунт с полями `account_id`, `display_name`, `external_account_id`, `runtime_kind`, `api_base_url`, `token_secret_ref`, флагами доступности webview и записи.
- Запросы/ответы конференций: `YandexTelemostConferenceRequest`, `YandexTelemostConferencePatchRequest`, `YandexTelemostConference` (содержит `join_url`, `access_level`, `waiting_room_level`, `live_stream`, `sip_*`).
- `YandexTelemostConferenceWebviewManifest` — манифест открытия конференции в webview: целевой origin `https://telemost.yandex.ru`, метка окна, конфигурация локальной записи (BlackHole на macOS, pulse на Linux, WASAPI loopback на Windows).
- Мосты: `YandexTelemostRecordingBridgeRequest/Response` и `YandexTelemostTranscriptBridgeRequest/Response`.

### Валидация (`client/validation.rs`)

- `validate_required` — обрезка пробелов и проверка непустоты.
- `validate_json_object` — проверка на JSON-объект.
- `validate_api_base_url` — требует `https://`, `http://127.0.0.1` или `http://localhost`.
- `validate_telemost_join_url` — допускает только хосты `telemost.yandex.ru` и `telemost.yandex.com`.
- `yandex_telemost_oauth_token_secret_ref(account_id)` — формирует stable-путь секрета вида `provider/yandex_telemost_user/{normalized_account_id}/oauth-token`.
- `sanitize_yandex_telemost_payload` — рекурсивно удаляет секретные и аудио-ключи (`oauth_token`, `mp3_bytes`, `access_token` и др.).

### Рантайм-мост транскрипта (`runtime_bridge.rs`)

- `complete_yandex_telemost_transcript_bridge` — материализует файлы транскрипта (JSON, Markdown, summary Markdown) в директорию бандла, проверяет соответствие `bundle_id` и `account_id` с манифестом, обновляет манифест (`CallBundleManifest`) артефактами, публикует событие `realtime_conversation.transcript_completed`.
- Генерация Markdown включает заголовок и полный текст транскрипта; summary рендерится отдельно.

> **Примечание:** Файлы `client/models.rs`, `client/store.rs` и `runtime_bridge.rs` обрезаны; полное описание может быть неполным.

## Интеграция Zoom

Расположение: `backend/src/integrations/zoom/`.

В чанк включены только:
- `client.rs` — публикует модули `errors`, `models`, `store`, `validation`.
- `client/errors.rs` — перечисление `ZoomError`, объединяющее ошибки провайдера аккаунтов, секретных привязок, звонков (`CallError`), хранилища файлов (`StorageError`), хранилища событий, HTTP, сериализации, SQL, I/O.

Файлы `models.rs`, `store.rs`, `validation.rs` **отсутствуют в данном чанке**, их содержимое не подтверждено.

## Общие зависимости

Все интеграции опираются на платформенные порты:
- `ProviderAccountCommandPort`, `ProviderSecretBindingCommandPort`, `ProviderChannelMessageLookupPort` (из `platform::communications`).
- `SecretReferenceStore`, `HostVault` (из `platform::secrets` и `vault`).
- `EventStore`, `EventBus` (из `platform::events`).

Конкретные реализации платформенных портов не включены в данный чанк.
```

### Source coverage / Покрытие источников

- **backend/src/integrations/whatsapp/client/store/queries.rs**  
  - `WhatsappWebStore::recent_messages` — фильтрация по `account_id`, `provider_chat_id`, `limit`, каналы `["whatsapp_web", "whatsapp_business_cloud"]`, вызов базового `provider_channel_message_store`, маппинг в `WhatsappWebMessage`.

- **backend/src/integrations/whatsapp/client/store/sessions.rs**  
  - `upsert_session` — `ON CONFLICT (account_id) DO UPDATE`, транзакционная запись observation `"upsert"`.  
  - `list_sessions` — фильтр `account_id`, сортировка, лимит.  
  - `update_session_last_sync` — логика `GREATEST`, observation `"sync_progress"`.  
  - `update_session_link_state` — обновление `link_state`, observation `"link_state_update"`.

- **backend/src/integrations/whatsapp/client/validation.rs**  
  - `validate_limit` — диапазон `1..=100`.  
  - `validate_non_empty` — тримминг и проверка на пустоту.  
  - `validate_object` — требование JSON-объекта.  
  - `validate_string_array` — проверка непустых строк.

- **backend/src/integrations/whatsapp/mod.rs**  
  - Публичные модули: `client`, `runtime`.

- **backend/src/integrations/whatsapp/runtime/business_cloud.rs** (частично)  
  - Константы конфигурации, `BUSINESS_CLOUD_GRAPH_API_VERSION_CONFIG_KEY`, `BUSINESS_CLOUD_DEFAULT_GRAPH_API_VERSION`.  
  - Поддерживаемые команды `send_text`, `send_template`, `send_media`, `send_voice_note`.  
  - `BusinessCloudRuntimeManager` и его метод `execute_live_provider_command`.  
  - Требования к `access_token` из host-vault и загрузке медиа.

- **backend/src/integrations/whatsapp/runtime/contracts.rs** (частично)  
  - `WhatsAppProviderRuntimeShape` (три значения).  
  - `WhatsAppSanitizedRuntimeEventDto` и его `assert_event_spine_contract`.  
  - Запросы управления и действий.

- **backend/src/integrations/whatsapp/runtime/mod.rs** (частично)  
  - Структура `ShapedWhatsAppProviderRuntime`, макросы делегирования.  
  - `whatsapp_web_companion_runtime` — фабрика рантайма web-компаньона.

- **backend/src/integrations/whatsapp/runtime/native_md.rs** (частично)  
  - Каналы команд и событий (`DurableOutbox`, `SignalHubRawEvidence`).  
  - Константы переподключения.  
  - `NativeMdTransientAuthArtifacts`, `NativeMdTransientAuthArtifact`.  
  - `NativeMdRuntimeLifecycleRegistry` и события жизненного цикла.

- **backend/src/integrations/whatsapp/runtime/web_companion.rs**  
  - `build_runtime` с формой `WebCompanion`.  
  - `web_companion_bridge_contract_health_check` — JSON-контракт web-компаньона.

- **backend/src/integrations/yandex_telemost/client/errors.rs**  
  - Перечисление `YandexTelemostError` со всеми вариантами.

- **backend/src/integrations/yandex_telemost/client/mod.rs**  
  - Публичные элементы: ошибка, модели, `YandexTelemostHttpClient`, `YandexTelemostStore`, функции валидации.

- **backend/src/integrations/yandex_telemost/client/models.rs** (частично)  
  - Константы `YANDEX_TELEMOST_PROVIDER_KIND_STR`, `YANDEX_TELEMOST_RUNTIME_KIND`, `YANDEX_TELEMOST_API_BASE_URL`.  
  - `YandexTelemostAccount`, запросы/ответы конференций, мосты записи и транскрипта.

- **backend/src/integrations/yandex_telemost/client/store.rs** (частично)  
  - `YandexTelemostHttpClient`: методы `create_conference`, `get_conference`, `update_conference`, `list_cohosts`.  
  - `YandexTelemostStore`: `setup_account`, `list_accounts`, `runtime_status`, `cleanup_retention`.

- **backend/src/integrations/yandex_telemost/client/validation.rs**  
  - `validate_required`, `validate_json_object`, `validate_api_base_url`.  
  - `validate_telemost_join_url` — ограничение хостов.  
  - `yandex_telemost_oauth_token_secret_ref` — генерация stable-пути.  
  - `sanitize_yandex_telemost_payload` — удаление секретных ключей.

- **backend/src/integrations/yandex_telemost/mod.rs**  
  - Публичные модули: `client`, `runtime`, `runtime_bridge`.

- **backend/src/integrations/yandex_telemost/runtime.rs**  
  - Реэкспорт типов из client (webview manifest, runtime status).

- **backend/src/integrations/yandex_telemost/runtime_bridge.rs** (частично)  
  - `complete_yandex_telemost_transcript_bridge` и `materialize_yandex_telemost_transcript_artifacts`.  
  - Валидация запроса транскрипта, запись Markdown, публикация события.

- **backend/src/integrations/zoom/client.rs**  
  - Структура модуля `errors`, `models`, `store`, `validation`.

- **backend/src/integrations/zoom/client/errors.rs**  
  - `ZoomError` с вариантами ошибок (включая `CallError`, `StorageError`).

### Drift candidates / Кандидаты на drift

Из предоставленного контекста явных расхождений между кодом и документацией не видно. Полноценную проверку drift-кандидатов ограничивают следующие обстоятельства:

- Многие исходные файлы обрезаны по лимиту в 12000 символов (`business_cloud.rs`, `contracts.rs`, `mod.rs`, `native_md.rs`, `models.rs`, `store.rs`, `runtime_bridge.rs`), что не позволяет сопоставить полную реализацию.
- Для Zoom-интеграции доступен только файл ошибок; файлы `models.rs`, `store.rs`, `validation.rs` не встроены в чанк, их содержимое не подтверждено.
- Отсутствуют смежные артефакты (миграции БД, конфигурационные файлы, тесты, документация платформенных портов), необходимые для выявления drift.

В рамках данного чанка утверждается, что candidate drift отсутствует.
