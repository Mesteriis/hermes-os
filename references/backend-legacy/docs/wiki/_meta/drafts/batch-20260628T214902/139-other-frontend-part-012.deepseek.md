### Summary / Резюме

Создана страница `components/frontend.md` русской Obsidian wiki, описывающая общие UI-компоненты, переходы, инструменты трассировки событий, настройку почты и панели интеграции Zoom. Документация основана исключительно на предоставленных исходных файлах фронтенда.

### Proposed pages / Предлагаемые страницы

#### `components/frontend.md`

# Компоненты фронтенда

Документирует общие UI-компоненты, инструменты трассировки, настройку почты и панели интеграции Zoom, присутствующие в кодовой базе.

## Общие UI-компоненты (shared/ui)

- **`Avatar`** (`shared/ui/Avatar.vue`): Аватар с изображением и fallback-текстом. Поддерживает размеры `sm`, `md`, `lg`, `xl`. Использует `reka-ui` (AvatarRoot, AvatarImage, AvatarFallback). При отсутствии `src` или `fallback` отображает `?` или первые два символа `alt`.
- **`Badge`** (`shared/ui/Badge.vue`): Бейдж с вариантами: `default`, `accent`, `success`, `warning`, `danger`, `info`, `neutral` и размерами `sm`, `md`.
- **`Button`** (`shared/ui/Button.vue`): Кнопка с вариантами: `default`, `secondary`, `outline`, `ghost`, `destructive`; размерами `sm`, `md`, `lg`; поддержкой иконки (`icon`), состояния загрузки (`loading`) и отключения. Эмиттирует событие `click`.
- **`Card`**, **`CardContent`**, **`CardDescription`**, **`CardFooter`**, **`CardHeader`**, **`CardTitle`** (`shared/ui/Card*.vue`): Набор компонентов для построения карточек: контейнер, заголовок, описание, тело, футер.
- **`Command`** (`shared/ui/Command.vue`): Командная палитра (модальное окно поиска) с группами элементов. Поддерживает поиск по названию, описанию и ключевым словам, навигацию стрелками и Enter. Использует `reka-ui` Dialog. Принимает пропсы `groups` (массив `CommandGroup`), `placeholder`, `emptyMessage`. Эмиттирует `select` с выбранным элементом.
- **`Dialog`** (`shared/ui/Dialog.vue`): Модальное окно на базе `reka-ui` Dialog. Имеет слоты `trigger`, `header`, `default`, `footer` и кнопку закрытия. Принимает `open`, `title`, `description`.
- **`DropdownMenu`** (`shared/ui/DropdownMenu.vue`): Выпадающее меню на основе `reka-ui`. Содержит слот `trigger` и использует `DropdownMenuItem` и `DropdownMenuLabel` в качестве дочерних.
- **`DropdownMenuItem`** (`shared/ui/DropdownMenuItem.vue`): Элемент выпадающего меню с опциональной иконкой, отключённым состоянием и отступом (`inset`).
- **`DropdownMenuLabel`** (`shared/ui/DropdownMenuLabel.vue`): Заголовок группы в выпадающем меню.

## Переходы (shared/transitions)

- **`FadeTransition`** (`shared/transitions/FadeTransition.vue`): Компонент-обёртка для Vue `<Transition>` с плавным появлением/исчезновением (opacity). Принимает `duration` (мс, по умолчанию 200), `mode` (`in-out`/`out-in`, по умолчанию `out-in`), `appear`.
- **`SlideTransition`** (`shared/transitions/SlideTransition.vue`): Анимированный переход со сдвигом. Направления: `up`, `down`, `left`, `right` (по умолчанию `up`). Принимает `duration` (мс, 200), `distance` (CSS-значение, `1rem`), `mode`, `appear`. Анимация включает сдвиг и изменение прозрачности.

## Инструмент трассировки событий (platform/event-tracing)

- **`EventTraceWorkspace`** (`platform/event-tracing/EventTraceWorkspace.vue`): Рабочая область для просмотра трасс событий. Позволяет искать трассу по `event_id` или `correlation_id`. Использует запросы `useEventTraceByEventIdQuery` и `useEventTraceByCorrelationIdQuery`. Передаёт результат в `EventTracePanel`.
- **`EventTracePanel`** (`platform/event-tracing/EventTracePanel.vue`): Панель отображения трассы с заголовком (correlation_id, метрики: количество событий, рёбер, отсутствующих родителей, dead letters). Список событий с позицией (`#position`), типом, идентификатором, родительским событием или меткой `root`. При выборе события показывает детали: тип, источник, субъект, causation_id, recorded_at, аннотации потребителей/ DLQ, и JSON-представление `subject`. Подсвечивает активное событие.

## Настройка почты (mail)

- **`AccountSetupModal`** (`shared/mailSetup/AccountSetupModal.vue`): Модальное окно добавления почтового аккаунта. Два шага: выбор провайдера (Gmail, iCloud, IMAP) и заполнение данных. Для Gmail запускает OAuth-авторизацию через `useStartGmailOAuthSetupMutation` и открывает URL в новом окне. Для iCloud и IMAP использует `useSetupImapEmailAccountMutation`. Валидация через `vee-validate`. Поля формы зависят от провайдера: для IMAP требуются хост, порт, логин, пароль, настройки TLS/STARTTLS; для iCloud — email и пароль приложения. (Файл обрезан, полный набор полей для iCloud не виден.)
- **`MailSyncSettingsStrip`** (`shared/mailSync/MailSyncSettingsStrip.vue`): Компактная панель настроек синхронизации почты. Принимает пропсы `settings` (текущие настройки или null), `isLoading`, `isSaving`. Позволяет включить/отключить синхронизацию, задать `batch_size` (1–500) и `poll_interval_seconds` (60–86400). Эмиттирует событие `update` с обновлёнными настройками. Использует `vee-validate` для валидации.

## Интеграция Zoom (integrations/zoom)

### Основная панель — `ZoomSettingsPanel`

Компонент `ZoomSettingsPanel` (`integrations/zoom/components/ZoomSettingsPanel.vue`, файл обрезан) управляет аккаунтами Zoom и включает вложенные панели.

- **Вложенные панели**: `ZoomAuditEventsPanel`, `ZoomBridgeLab`, `ZoomRecordingMaintenancePanel`, `ZoomObservedCallsPanel`, `ZoomRecordingImportsPanel`.
- **Управление аккаунтами**:
  - **Создание фикстуры**: `handleCreateZoomFixture` через мутацию `useSetupZoomFixtureAccountMutation`. Требует `account_id`, `display_name`, `external_account_id`; опционально `account_email`.
  - **Создание live-аккаунта**: `handleCreateZoomLive` через `useSetupZoomLiveAccountMutation`. Требует `account_id`, `display_name`, `external_account_id`, `client_id`; опционально `token_secret_ref`, `client_secret_ref`, `webhook_secret_ref`. Поддерживает `auth_shape`: `oauth_user` или `server_to_server`.
  - **OAuth**: `handleStartZoomOAuth` запускает OAuth-авторизацию (мутация `useStartZoomOAuthMutation`), открывает URL в новом окне и сохраняет `setup_id`, `state`. `handleCompleteZoomOAuth` завершает OAuth, используя `setup_id`, `state`, `authorization_code`.
  - **Server-to-Server авторизация**: `authorizeZoomServerToServer` (мутация `useAuthorizeZoomServerToServerMutation`).
  - **Токены**: `handleRefreshZoomToken`, `handleMaintainZoomTokens` для обновления и обслуживания токенов.
  - **Рантайм**: `handleStartZoomRuntime`, `handleStopZoomRuntime` и `handleRemoveZoomRuntime`.
- **Статус**: Отображает `selectedZoomRuntime` (статус, блокировщики, политику ротации токенов) и `zoomCapabilities` (planned_features, unsupported_features). Показывает выбранный аккаунт: `selectedZoomAccountId`, `selectedDisplayName`, `selectedClientId`, `selectedAccountEmail`.

(Из-за обрезания файла часть логики не видна.)

### Панели интеграции

- **`ZoomBridgeLab`** (`integrations/zoom/components/ZoomBridgeLab.vue`): Лаборатория runtime-бриджа. Позволяет вручную отправлять наблюдения (meeting, recording, transcript) через мутации `useBridgeZoomMeetingMutation`, `useBridgeZoomRecordingMutation`, `useBridgeZoomTranscriptMutation` и импортировать файлы транскриптов через `useImportZoomTranscriptFileMutation`. Требует выбранный Zoom-аккаунт. Результат отображается в формате `meeting:<call_id>:<event_id>`, `recording:<recording_id>:<event_id>`, `transcript:<transcript_id>:<event_id>`, или `transcript-file:<transcript_id>:<event_id>:<import_format>`. Ошибки записываются в `settingsStore.error`. Все поля ввода имеют предустановленные примеры JSON.
- **`ZoomObservedCallsPanel`** (`integrations/zoom/components/ZoomObservedCallsPanel.vue`): Отображает спроецированные доказательства звонков (`provider calls`) для выбранного аккаунта. Загружает список через `useZoomProviderCallsQuery` с лимитом 12. При выборе звонка показывает метаданные (`provider_call_id`, `direction`, `call_state`, `started_at`, `topic`, `host_email`), ссылки на записи (`recording_id`, `recording_type`, `file_extension`, `file_size_bytes`, `recorded_at`, `download_ref`), и доказательства транскрипта (`transcript_status`, `stt_provider`, `language_code`, `source_audio_ref`, `transcript_text`, `provenance`). Если транскрипт отсутствует, выводится сообщение. Для извлечения записей используется `extractZoomRecordingRefs`, для форматирования provenience — `formatZoomTranscriptProvenance`.
- **`ZoomRecordingImportsPanel`** (`integrations/zoom/components/ZoomRecordingImportsPanel.vue`): Панель аудита импортированных записей. Загружает список импортов через `useZoomRecordingImportsQuery` с лимитом 12. Для каждого элемента показывает `recording_id`, `meeting_id`, `source`, `filename`, `content_type`, `size_bytes`, `scan_status`, `storage_kind`, `retention_mode`, `expires_at`, `sha256`. Позволяет удалить локальный импорт через `useRemoveZoomRecordingImportMutation`; результат операции содержит флаг `blob_file_removed`.
- **`ZoomRecordingMaintenancePanel`** (`integrations/zoom/components/ZoomRecordingMaintenancePanel.vue`): Ручная синхронизация записей и очистка по retention. Отображается только для аккаунтов типа `zoom_user` или `zoom_server_to_server`. Синхронизация (`useSyncZoomRecordingsMutation`) принимает параметры: `user_id`, `from`, `to`, `page_size`, `max_meetings`, `api_base_url`. Результат синхронизации включает поля `meetings_recorded`, `recordings_recorded`, `media_downloads_recorded`, `transcripts_recorded`. Очистка (`useCleanupZoomRetentionMutation`) удаляет записи и транскрипты (параметры `remove_recordings`, `remove_transcripts`, `limit`). Показывает текущие настройки retention из application settings (`privacy.zoom_recording_import_retention_days`, `privacy.zoom_transcript_retention_days`).

### Source coverage / Покрытие источников

- **`frontend/src/integrations/zoom/components/ZoomBridgeLab.vue`** (обрезан): Формы ручного инжеста meeting/recording/transcript, использование `useBridgeZoom*Mutation`, обработка JSON, отображение результата.
- **`frontend/src/integrations/zoom/components/ZoomObservedCallsPanel.vue`**: Загрузка provider calls (лимит 12), детализация звонка, записей и транскрипта, извлечение recording refs и форматирование provenance.
- **`frontend/src/integrations/zoom/components/ZoomRecordingImportsPanel.vue`**: Загрузка импортированных записей (лимит 12), отображение метаданных, удаление локального импорта, флаг `blob_file_removed`.
- **`frontend/src/integrations/zoom/components/ZoomRecordingMaintenancePanel.vue`**: Ручная синхронизация записей (параметры `from`, `to`, `page_size`, `max_meetings`, `api_base_url`) и retention cleanup, настройки из application settings.
- **`frontend/src/integrations/zoom/components/ZoomSettingsPanel.vue`** (обрезан): Управление аккаунтами Zoom (фикстура, live, OAuth, S2S), токенами и рантаймом, отображение статуса, политик ротации, capabilities.
- **`frontend/src/platform/event-tracing/EventTracePanel.vue`**: Отображение трассы с метриками, событиями, аннотациями, dead letters и деталями события.
- **`frontend/src/platform/event-tracing/EventTraceWorkspace.vue`**: Поиск трассы по `event_id`/`correlation_id`, передача данных в `EventTracePanel`.
- **`frontend/src/shared/mailSetup/AccountSetupModal.vue`** (обрезан): Модальное окно добавления почты (Gmail OAuth, iCloud/IMAP), двухшаговая форма.
- **`frontend/src/shared/mailSync/MailSyncSettingsStrip.vue`**: Полоса настроек синхронизации (вкл/выкл, batch_size, poll_interval_seconds).
- **`frontend/src/shared/transitions/FadeTransition.vue`**: Плавное появление/исчезновение с настраиваемой длительностью.
- **`frontend/src/shared/transitions/SlideTransition.vue`**: Переход со сдвигом по направлениям, длительностью и расстоянием.
- **`frontend/src/shared/ui/Avatar.vue`**: Аватар на базе reka-ui с размерами и fallback.
- **`frontend/src/shared/ui/Badge.vue`**: Бейдж с вариантами и размерами.
- **`frontend/src/shared/ui/Button.vue`**: Кнопка с вариантами, состояниями и иконкой.
- **`frontend/src/shared/ui/Card.vue`**, **`CardContent.vue`**, **`CardDescription.vue`**, **`CardFooter.vue`**, **`CardHeader.vue`**, **`CardTitle.vue`**: Компоненты карточки.
- **`frontend/src/shared/ui/Command.vue`**: Командная палитра с поиском, группами и навигацией.
- **`frontend/src/shared/ui/Dialog.vue`**: Модальное окно на reka-ui с слотами.
- **`frontend/src/shared/ui/DropdownMenu.vue`**, **`DropdownMenuItem.vue`**, **`DropdownMenuLabel.vue`**: Выпадающее меню и его элементы.

### Drift candidates / Кандидаты на drift

No code/docs/ADR drift visible from the provided context. / Из данного контекста расхождения не видны.
