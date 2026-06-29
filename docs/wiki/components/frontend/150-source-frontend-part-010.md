---
chunk_id: 150-source-frontend-part-010
batch_id: batch-20260628T214902
group: frontend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 150-source-frontend-part-010 — frontend/source

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Создаётся страница `components/frontend.md` для русской Obsidian wiki, описывающая архитектуру фронтенда на основе предоставленных исходных файлов. На странице документируются домены `projects`, `review`, `settings` и `tasks`: используемые API-функции, хранилища Pinia, запросы TanStack Vue Query, типы TypeScript, презентационные утилиты и тестовые границы. Описание строится строго по встроенному коду, без внешних предположений.

## Предложенные страницы

#### `components/frontend.md`

```markdown
# Компоненты фронтенда

## Общая структура

Фронтенд построен на **Vue 3** с использованием **Pinia** для управления состоянием и **TanStack Vue Query** для кеширования и запросов к API. Взаимодействие с бекендом осуществляется через `ApiClient`, расположенный в `platform/api/`. Доменная логика разбита на модули (`domains/`), каждый из которых содержит:

- `api/` – функции для HTTP-запросов к бекенду
- `stores/` – Pinia-хранилища состояния UI
- `queries/` – обёртки над Vue Query для запросов и мутаций
- `types/` – TypeScript-типы домена
- `components/` – Vue-компоненты
- `lib/` – вспомогательные функции

## Домен `projects`

Управление проектами: отображение списка, деталей, временной шкалы.

### API и запросы

- `useProjectsQuery()` – возвращает `ProjectSummary[]`, запрашивая `fetchProjects(25)`
- `useProjectQuery(projectId)` – возвращает `ProjectDetail | null`, запрос выполняется только при непустом `projectId`
- Функции `fetchProjects` и `fetchProjectDetail` импортируются из `../api/projects` (в данном контексте не показаны)

### Хранилище `useProjectsStore` (Pinia `projects-ui`)

Состояние:
- `selectedProjectId` – выбранный проект
- `projectsError` – сообщение об ошибке
- `isProjectsLoading` – флаг загрузки

Действия:
- `selectProject(projectId)`
- `setError(msg)`, `clearError()`
- `setLoading(val)`

### Вспомогательные функции (stores/projects.ts)

- `projectStatusLabel(status: string)` – преобразует статус вида `planning` / `active` / `on_hold` / `completed` / `archived` в читаемый текст (замена `_` на пробелы, капитализация)
- `projectTimelineIcon(itemKind)` – возвращает иконку для элемента таймлайна: `tabler:mail` для `'message'`, `tabler:file-text` для `'document'`, иначе `tabler:circle-dot`
- `projectDocumentIcon(documentKind)` – иконка для типа документа: `tabler:file-type-pdf`, `tabler:file-text`, `tabler:file`
- `formatProjectDate(value)` – форматирует дату в `Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', year: 'numeric' })` или `'Not set'` / `'Invalid date'`
- `formatProjectDateTime(value)` – аналогично с добавлением часов и минут, возвращает `'No activity'` при отсутствии значения

### Типы (`types/project.ts`)

- `ProjectStatus` – строковый литерал статусов проекта
- `ProjectRecord` – поля проекта (id, name, kind, status, ...)
- `ProjectStats` – счётчики сообщений, документов, людей, связей графа, дата последней активности
- `ProjectSummary` – объединяет `ProjectRecord`, `ProjectStats` и `graph_node_id`
- `ProjectTimelineItem` – элемент временной шкалы
- `ProjectPersonSummary`, `ProjectMessageSummary`, `ProjectDocumentSummary` – сводки
- `ProjectDetail` – расширенная детальная информация (проект + статистика + таймлайн + ключевые люди + последние сообщения + документы)
- `ProjectListResponse` – ответ списка проектов

## Домен `review`

Обработка входящих элементов ревью (inbox) и рабочего пространства ревью (связи, решения, обязательства, противоречия).

### API – элементы ревью (`api/items.ts`)

Функции взаимодействия с `/api/v1/review/items`:

- `fetchReviewItems({ status?, limit? })` → `ReviewItemsResponse`
- `approveReviewItem(reviewItemId)` → `ReviewItem`
- `dismissReviewItem(reviewItemId)` → `ReviewItem`
- `takeReviewItem(reviewItemId)` → `ReviewItem`
- `archiveReviewItem(reviewItemId)` → `ReviewItem`
- `promoteReviewItem(reviewItemId, payload: ReviewItemPromotionRequest)` → `ReviewItem`

Все функции используют `ApiClient.instance.get/post`.

### API – рабочее пространство (`api/workspace.ts`)

Функции для сущностей:

- **Связи** (`/api/v1/relationships`): `fetchRelationships(limit)`, `reviewRelationship(id, reviewState)`
- **Решения** (`/api/v1/decisions`): `fetchDecisionReviewItems({ reviewState, limit })`, `reviewDecision(id, { review_state })`
- **Обязательства** (`/api/v1/obligations`): `fetchObligationReviewItems({ reviewState, limit })`, `reviewObligation(id, { review_state })`
- **Противоречия** (`/api/v1/contradictions`): `fetchContradictions(limit)`, `reviewContradiction(observationId, { review_state })`

### Хранилище `useReviewStore` (Pinia `review`)

Состояние:
- `relationships`, `decisions`, `obligations`, `contradictions`, `reviewItems` – массивы сущностей
- `error` – аккумулированная строка ошибок
- `reviewingItemKey` – ключ элемента, находящегося в процессе ревью

Вычисляемые свойства:
- `relationsSuggestedCount`, `decisionsSuggestedCount`, `obligationsSuggestedCount`, `contradictionsSuggestedCount` – количество элементов с `review_state === 'suggested'`
- `reviewItemsCount` – количество элементов инбокса со статусом `'new'` или `'in_review'`
- `totalSuggestedCount` – сумма всех предложенных

Действия:
- `loadAll()` – параллельно загружает все категории; при ошибках накапливает сообщения в `error`
- `reviewItem(action: ReviewWorkspaceItemAction)` – выполняет ревью-действие (подтверждение/отклонение/архивация/продвижение) через API, затем обновляет локальный массив. Возвращает строку ошибки или пустую строку при успехе.

Вспомогательная функция `reviewItemKey(action)` генерирует строковый ключ на основе типа и идентификатора сущности (например, `relationship:...`, `review_item:...`).

### Типы (`types/review.ts`)

- Состояния ревью: `RelationshipReviewState`, `DecisionReviewState`, `ObligationReviewState`, `ContradictionReviewState`, `UserRelationshipReviewState`
- Сущности: `Relationship`, `Decision`, `Obligation`, `ContradictionObservation`, `ReviewItem`
- Ответы: `RelationshipListResponse`, `DecisionListResponse`, `ObligationListResponse`, `ContradictionListResponse`, `ReviewItemsResponse`
- Статусы элемента ревью: `ReviewItemStatus` (`'new' | 'in_review' | 'approved' | 'promoted' | 'dismissed' | 'archived'`)
- Виды элементов ревью: `ReviewItemKind` (11 строковых литералов, таких как `'new_person'`, `'potential_relationship'` и т.д.)
- `ReviewWorkspaceItemAction` – дискриминируемый union для действий над различными типами элементов
- `ReviewItemPromotionRequest` – запрос на продвижение с целевым доменом, видом и идентификатором сущности

## Домен `settings`

Управление настройками приложения, учётными записями провайдеров и **Signal Hub** (источники сигналов, соединения, профили, политики, здоровье, рантайм, переповтор).

### API – общие настройки и учётные записи (`api/settings.ts`)

Реэкспортирует из платформенного слоя:
- `fetchApplicationSettings`, `saveApplicationSetting`
- Ключи настроек: `FRONTEND_LAYOUT_SETTING_KEY`, `FRONTEND_SIDEBAR_SETTING_KEY`, `FRONTEND_LOCALE_SETTING_KEY`, `FRONTEND_THEME_SETTING_KEY`, `FRONTEND_UI_STATE_SETTING_KEY`

Собственные функции:
- `fetchProviderAccounts()` → `/api/v1/settings/accounts`
- `fetchCalendarAccounts()` → `/api/v1/settings/accounts/calendar`
- `deleteMailAccount(accountId)` → `/api/v1/settings/accounts/mail/{id}`
- `logoutMailAccount(accountId)` → `/api/v1/settings/accounts/mail/{id}/logout`
- `exportMailAccountSettings(accountId)` → `/api/v1/settings/accounts/mail/{id}/export`
- `importMailAccountSettings(request)` → `/api/v1/settings/accounts/mail/import`

### API – Signal Hub (`api/signalHub.ts`, усечён)

Использует ConnectRPC-клиент `getSignalHubConnectClient()` из `platform/connect/`. Предоставляет функции:

- **Источники:** `fetchSignalHubSources()`, `fetchSignalHubSource(sourceCode)`, `fetchSignalHubCapabilities()`, `fetchSignalHubFixtureSources()`
- **Восстановление фикстур:** `restoreSignalHubSystemFixture()`, `emitSignalHubFixtureSignal(fixtureId)`
- **Профили:** `fetchSignalHubProfiles()`, `createSignalHubProfile()`, `updateSignalHubProfile()`, `removeSignalHubProfile()`, `applySignalHubProfile()`
- **Соединения:** `fetchSignalHubConnections()`, `createSignalHubConnection()`, `updateSignalHubConnection()`, `removeSignalHubConnection()`
- **Здоровье:** `fetchSignalHubHealth()`, `runSignalHubHealthCheck(request)`
- **Рантайм:** `fetchSignalHubRuntimeStates()`, `updateSignalHubRuntimeState()`
- **Replay:** `fetchSignalHubReplayRequests()`, `createSignalHubReplayRequest()`
- **Политики:** `fetchSignalHubPolicies()`, `createSignalHubPolicy()`
- **Управление сигналами:** `enableSignalHubSource()`, `disableSignalHubSource()`, `enableSignalHubSignals()`, `disableSignalHubSignals()`, `muteSignalHubSignals()`, `unmuteSignalHubSignals()`, `pauseSignalHubSignals()`, `resumeSignalHubSignals()`

### Queries – общие настройки (`queries/useSettingsQuery.ts`)

- `useApplicationSettingsQuery()`, `useProviderAccountsQuery()`, `useCalendarAccountsQuery()`
- `useSettingsWorkspaceQuery()` – агрегирует три запроса параллельно
- `findSetting(settings, key)` – поиск конкретной настройки
- `groupSettingsByCategory(settings)` – группировка по категории

Ключи запросов: `settingsKeys.all`, `.application()`, `.providerAccounts()`, `.calendarAccounts()`, `.workspace()`.

### Queries – Signal Hub (`queries/useSignalHubQuery.ts`)

Шаблон для всех запросов и мутаций: каждый query использует соответствующий API-вызов, каждая мутация (create/update/remove/enable/disable/mute/unmute/pause/resume) при успехе инвалидирует все ключи `signalHubKeys.all`.

Основные хуки:
- Queries: `useSignalHubSourcesQuery`, `useSignalHubCapabilitiesQuery`, `useSignalHubFixtureSourcesQuery`, `useSignalHubConnectionsQuery`, `useSignalHubProfilesQuery`, `useSignalHubHealthQuery`, `useSignalHubRuntimeStatesQuery`, `useSignalHubReplayRequestsQuery`, `useSignalHubPoliciesQuery`
- Mutations: `useRestoreSignalHubFixtureMutation`, `useEmitSignalHubFixtureMutation`, `useApplySignalHubProfileMutation`, `useCreateSignalHubProfileMutation`, `useUpdateSignalHubProfileMutation`, `useRemoveSignalHubProfileMutation`, `useCreateSignalHubConnectionMutation`, `useUpdateSignalHubConnectionMutation`, `useRemoveSignalHubConnectionMutation`, `useRunSignalHubHealthCheckMutation`, `useCreateSignalHubReplayRequestMutation`, `useCreateSignalHubPolicyMutation`, `useEnableSignalHubSourceMutation`, `useDisableSignalHubSourceMutation`, `useDisableSignalHubMutation`, `useEnableSignalHubMutation`, `useMuteSignalHubMutation`, `useUnmuteSignalHubMutation`, `usePauseSignalHubMutation`, `useResumeSignalHubMutation`, `useUpdateSignalHubRuntimeStateMutation`

Ключи запросов тестируются в `queries/useSignalHubQuery.test.ts` – все они начинаются с `['signal-hub', ...]`.

### Хранилище `useSettingsStore` (Pinia `settings-ui`)

Состояние:
- `selectedSection` – тип `SettingsSection` (appearance, language, application, sidebar, integrations, signal-hub, ai)
- `actionMessage`, `errorMessage`, `savingSettingKey` – UI-обратная связь
- `settingDrafts` – черновики значений настроек
- `isSidebarSettingsSaving`, `sidebarError`, `newSidebarGroupLabel` – состояние редактирования сайдбара
- `selectedIntegrationId` – выбранная интеграция

Действия:
- `selectSection(section)`, `setActionMessage`, `setError`, `clearMessages`
- `updateSettingDraft(key, value)`
- `saveSetting(setting)` – отправляет значение через `saveApplicationSetting`, с приведением типа (`coerceValue`) в зависимости от `value_kind` (`boolean`, `integer`, `json`)
- `selectIntegration(id)`, `updateNewSidebarGroupLabel`

### Типы (`types/settings.ts` и `types/signalHub.ts`)

**Общие настройки:**
- Реэкспорт `ApplicationSetting`, `ApplicationSettingsResponse`, `ApplicationSettingValue`, `SettingValueKind` и ключей настроек из платформенного слоя
- `ProviderAccount` – учётная запись провайдера (свойства `account_id`, `provider_kind`, `display_name`, `email`, `is_active` и др.)
- `ProviderAccountListResponse` – `{ items: ProviderAccount[] }`
- `CalendarAccount` – учётная запись календаря

**Signal Hub:**
- `SignalHubSource` – источник (code, display_name, category, source_kind, булевы флаги supports_connections, supports_runtime, supports_replay, supports_pause, supports_mute, capability_schema_version, created_at, updated_at)
- `SignalHubCapability` – возможность (source_code, connection_id, capability, state, reason, requires_confirmation, action_class)
- `SignalHubConnection` – соединение (id, source_code, display_name, status, profile, settings, secret_ref, временные метки активности)
- `SignalHubHealth` – запись о здоровье (source_code, connection_id, level, summary, failure_count, consecutive_failure_count, evidence, временные метки)
- `SignalHubRuntimeState` – состояние рантайма (source_code, connection_id, runtime_kind, state, last_started_at, last_stopped_at, last_heartbeat_at, last_error_*, metadata)
- `SignalHubReplayRequest` – запрос воспроизведения (source_code, connection_id, event_pattern, from/to position/time, target_consumer, target_projection, status и др.)
- `SignalHubProfile` – профиль (code, display_name, description, policy_count, source_policies, is_system, is_active)
- `SignalHubPolicy` – политика (scope, source_code, connection_id, event_pattern, mode, reason, expires_at)
- Перечисления: `SignalHubPolicyScope` (global, source, connection, event_pattern, profile), `SignalHubPolicyMode` (enabled, disabled, muted, paused, replay_only, fixture_only)
- `SignalHubFixtureSource`, `SignalHubFixtureRestoreReport`, `SignalHubFixtureEmission`
- Запросы на создание/обновление соединений, профилей, политик, контроля сигналов, состояний рантайма

### Компоненты и презентационные утилиты

**`signalHubSettingsPresentation.ts`** – чистые функции для отображения:
- `capabilityLabels(source)` – список меток поддерживаемых возможностей
- `capabilityTone(state)` – тон (good / warn / bad / neutral) по состоянию
- `sourceControlState(policies, source)` – определяет состояние управления источником (running / muted / paused / disabled / off)
- `sourceIcon(source)`, `sourceIconForCode(sources, sourceCode)` – иконки для кодов источников (telegram, mail, browser, calendar, …)
- Функции тонов: `statusTone`, `sourceStateTone`, `healthTone`, `runtimeTone`
- Функции форматирования сводок: `formatSettingsSummary`, `formatConnectionTimeline`, `formatRuntimeTimeline`, `formatRuntimeError`, `formatHealthStatus`, `formatHealthEvidence`
- `policyTargetLabel`, `profilePolicyLabel`, `capabilityLabel` – текстовые метки политик и возможностей
- `connectionLabel` – метка соединения по id

**Контроллер `useSignalHubSettingsController`** (усечён) – управляет состоянием UI Signal Hub:
- Вкладки: источники, профили, соединения, рантайм, политики, здоровье, воспроизведение
- Состояние выбора: `selectedSourceCode`, `selectedProfileCode`, фильтры, параметры создания политик/соединений/профилей
- Вычисляемые свойства на основе данных запросов (например, `filteredSources`, `connectionCapableSources`, `selectedSourceCapabilities`, счётчики enabled/runtime/connected/unhealthy)
- Методы-обработчики: `handleRestoreFixture`, `handleEmitFixtureSignal`, `handleApplyProfile`, сброс редактора профиля и другие (полный код обрезан)

**Тесты границ** (`SignalHubSettings.boundary.test.ts` и `SettingsPage.signalHub.boundary.test.ts`) подтверждают, что:
- `SignalHubSettings.vue` использует локальные запросы и не импортирует интеграции напрямую, не содержит вызовов `ApiClient` или `fetch`
- Компоненты отрисовывают диагностику Signal Hub через презентационные функции
- `SettingsPage.vue` содержит вкладку `'signal-hub'` внутри секции настроек и не определяет отдельный маршрут

### Вспомогательные функции replay (`lib/signalHubReplay.ts` и тесты)

- `buildSignalHubReplayRequest(input)` – формирует объект запроса воспроизведения с нормализацией строк (trim, null для пустых), парсингом позиций в `bigint`, добавлением метаданных `requested_from: 'settings_signal_hub'`
- `describeSignalHubReplayRequest(request)` – строит читаемое описание запроса (включает connection, position, time, consumer, projection, дату), разделяя элементы через ` / `

## Домен `tasks`

Управление задачами, кандидатами задач, решениями и обязательствами.

### API (`api/tasks.ts`)

- **Кандидаты задач:** `fetchTaskCandidates(limit)`, `reviewTaskCandidate(id, reviewState)` → эндпоинт `/api/v1/task-candidates`
- **Записи задач:** `fetchTaskRecords({ status?, project_id?, source_type?, limit? })` → `/api/v1/tasks`; `updateTask(id, body)`; `setTaskStatus(id, status)` → `/api/v1/tasks/{id}/status`
- **Решения:** `fetchDecisions({ entityKind, entityId, limit? })`, `fetchDecisionReviewItems({ reviewState, limit? })`, `reviewDecision(id, request)` → `/api/v1/decisions`
- **Обязательства:** `fetchObligations({ entityKind, entityId, limit? })`, `fetchObligationReviewItems({ reviewState, limit? })`, `reviewObligation(id, request)` → `/api/v1/obligations`

### Queries (`queries/useTasksQuery.ts`)

- `useTaskCandidatesQuery()` – возвращает `TaskCandidate[]` через `fetchTaskCandidates(50)`
- `useTasksQuery()` – возвращает `Task[]` через `fetchTaskRecords({ limit: 50 })`

### Хранилище `useTasksStore` (Pinia `tasks-ui`)

Состояние:
- `tasksError`, `contextReviewError` – строки ошибок
- `isAiTaskRefreshSubmitting`, `isContextReviewLoading` – флаги отправки
- `reviewEntityKind`, `reviewEntityId` – параметры ревью
- `reviewingContextItemId` – ID элемента в процессе ревью
- `decisions`, `obligations` – массивы сущностей

Действия: `setError`, `clearError`, `setReviewEntityKind`, `setReviewEntityId`, `setReviewingItemId`, `setDecisions`, `setObligations`, `setContextReviewLoading`, `setContextReviewError`.

### Вспомогательные функции (stores/tasks.ts)

- `taskSourceLabel(item)` – строка вида `SourceKind · source_id`
- `taskConfidence(item)` – округлённый процент уверенности (×100)
- `taskCreatedTime(value)` – форматирование даты создания задачи
- `formatDecisionTime(value)` / `formatEntityKind(kind)` / `formatDecisionEntity(kind, id)` – форматирование для решений
- `formatObligationDueTime(value)` / `formatObligationEntity(kind, id)` – форматирование для обязательств

Даты форматируются через `Intl.DateTimeFormat('en', ...)` с месяцем, днём, часом и минутами.

## Зависимости от платформы

Все домены зависят от платформенных модулей, не раскрытых в этом контексте:
- `platform/api/ApiClient` – HTTP-клиент с методами `get`, `post`, `put`, `delete`
- `platform/connect/signalHubClient` – ConnectRPC-клиент для Signal Hub
- `platform/settings/applicationSettingsClient` – функции и типы для работы с настройками приложения (ключи, значения)
```

## Покрытие источников

Каждый файл из раздела Source Files покрыт в предложенной странице:

- `frontend/src/domains/projects/queries/useProjectsQuery.ts` – описаны запросы `useProjectsQuery`, `useProjectQuery`, их ключи и логика (загрузка с лимитом 25, условный запрос деталей)
- `frontend/src/domains/projects/stores/projects.ts` – перечислены состояние и действия хранилища `projects-ui`, а также все утилиты: `projectStatusLabel`, `projectTimelineIcon`, `projectDocumentIcon`, `formatProjectDate`, `formatProjectDateTime`
- `frontend/src/domains/projects/types/project.ts` – перечислены все типы и интерфейсы: `ProjectStatus`, `ProjectRecord`, `ProjectStats`, `ProjectSummary`, `ProjectTimelineItem`, `ProjectPersonSummary`, `ProjectMessageSummary`, `ProjectDocumentSummary`, `ProjectDetail`, `ProjectListResponse`
- `frontend/src/domains/review/api/items.ts` – перечислены все API-функции для `/api/v1/review/items`: `fetchReviewItems`, `approveReviewItem`, `dismissReviewItem`, `takeReviewItem`, `archiveReviewItem`, `promoteReviewItem`
- `frontend/src/domains/review/api/workspace.ts` – перечислены все API-функции для `/api/v1/relationships`, `/api/v1/decisions`, `/api/v1/obligations`, `/api/v1/contradictions`
- `frontend/src/domains/review/stores/review.ts` – документировано хранилище `review`: состояние, вычисляемые свойства (счётчики suggested), метод `loadAll` (параллельная загрузка, аккумулирование ошибок), метод `reviewItem` (диспетчиризация действий, обновление локального состояния), вспомогательная функция `reviewItemKey`
- `frontend/src/domains/review/types/review.ts` – перечислены все типы состояний ревью, сущности (`Relationship`, `Decision`, `Obligation`, `ContradictionObservation`, `ReviewItem`), ответы списков, `ReviewItemStatus`, `ReviewItemKind`, `ReviewWorkspaceItemAction`, `ReviewItemPromotionRequest`
- `frontend/src/domains/settings/api/settings.ts` – описаны реэкспорты из платформы и собственные функции для учётных записей и календарей, экспорта/импорта почтовых аккаунтов
- `frontend/src/domains/settings/api/signalHub.test.ts` (усечён) – не детализирован, но использован косвенно: тесты подтверждают поведение API, которые перечислены в секции Signal Hub; факты о вызовах ConnectRPC, обработке ответов не внесены, так как они являются тестовыми утверждениями, а не спецификацией
- `frontend/src/domains/settings/api/signalHub.ts` (усечён) – перечислены все видимые функции: `fetchSignalHubSources`, `fetchSignalHubSource`, `fetchSignalHubCapabilities`, `fetchSignalHubFixtureSources`, `restoreSignalHubSystemFixture`, профили, эмитирование фикстур, соединения, здоровье, рантайм, replay, политики, управление сигналами. Отмечено использование `getSignalHubConnectClient()`
- `frontend/src/domains/settings/components/SignalHubSettings.boundary.test.ts` – зафиксированы проверки использования локальных запросов и отсутствия прямых интеграционных импортов, а также отрисовка диагностик через презентационные функции
- `frontend/src/domains/settings/components/signalHubSettingsPresentation.ts` – покрыты все экспортируемые функции: `capabilityLabels`, `capabilityTone`, `sourceControlState`, `sourceIcon`, `sourceIconForCode`, `statusTone`, `sourceStateTone`, `healthTone`, `runtimeTone`, `connectionLabel`, `formatSettingsSummary`, `formatConnectionTimeline`, `formatRuntimeTimeline`, `formatRuntimeError`, `formatHealthStatus`, `formatHealthEvidence`, `policyTargetLabel`, `profilePolicyLabel`, `capabilityLabel`
- `frontend/src/domains/settings/components/useSignalHubSettingsController.ts` (усечён) – описаны: вкладки, состояние (selectedSourceCode, selectedProfileCode, фильтры, параметры политик/соединений/профилей/replay), вычисляемые свойства, видимые обработчики (`handleRestoreFixture`, `handleEmitFixtureSignal`, `handleApplyProfile`, `resetProfileEditor`). Полный перечень методов не документирован из-за обрезки
- `frontend/src/domains/settings/queries/useSettingsQuery.ts` – перечислены хуки (`useApplicationSettingsQuery`, `useProviderAccountsQuery`, `useCalendarAccountsQuery`, `useSettingsWorkspaceQuery`) и вспомогательные функции `findSetting`, `groupSettingsByCategory`, ключи запросов
- `frontend/src/domains/settings/queries/useSignalHubQuery.test.ts` – описаны ключи запросов (проверка структуры `signalHubKeys`)
- `frontend/src/domains/settings/queries/useSignalHubQuery.ts` – перечислены все хуки запросов и мутаций, описана логика инвалидации кеша при мутациях
- `frontend/src/domains/settings/stores/settings.ts` – документированы состояние и действия хранилища `settings-ui`, включая `coerceValue` для приведения типов
- `frontend/src/domains/settings/types/settings.ts` – описаны `ProviderAccount`, `CalendarAccount`, реэкспорты из платформы
- `frontend/src/domains/settings/types/signalHub.ts` – перечислены все интерфейсы и типы (источники, возможности, соединения, здоровье, рантайм, replay, профили, политики, запросы/ответы)
- `frontend/src/domains/settings/views/SettingsPage.signalHub.boundary.test.ts` – зафиксирована проверка нахождения Signal Hub как секции в `SettingsPage.vue`, а не отдельного маршрута
- `frontend/src/domains/tasks/api/tasks.ts` – перечислены все API-функции для `/api/v1/task-candidates`, `/api/v1/tasks`, `/api/v1/decisions`, `/api/v1/obligations`
- `frontend/src/domains/tasks/queries/useTasksQuery.ts` – описаны `useTaskCandidatesQuery`, `useTasksQuery`
- `frontend/src/domains/tasks/stores/tasks.ts` – документированы состояние и действия хранилища `tasks-ui`, а также все утилиты: `taskSourceLabel`, `taskConfidence`, `taskCreatedTime`, `formatDecisionTime`, `formatEntityKind`, `formatDecisionEntity`, `formatObligationDueTime`, `formatObligationEntity`

## Исходные файлы

- [`frontend/src/domains/projects/queries/useProjectsQuery.ts`](../../../../frontend/src/domains/projects/queries/useProjectsQuery.ts)
- [`frontend/src/domains/projects/stores/projects.ts`](../../../../frontend/src/domains/projects/stores/projects.ts)
- [`frontend/src/domains/projects/types/project.ts`](../../../../frontend/src/domains/projects/types/project.ts)
- [`frontend/src/domains/review/api/items.ts`](../../../../frontend/src/domains/review/api/items.ts)
- [`frontend/src/domains/review/api/workspace.ts`](../../../../frontend/src/domains/review/api/workspace.ts)
- [`frontend/src/domains/review/stores/review.ts`](../../../../frontend/src/domains/review/stores/review.ts)
- [`frontend/src/domains/review/types/review.ts`](../../../../frontend/src/domains/review/types/review.ts)
- [`frontend/src/domains/settings/api/settings.ts`](../../../../frontend/src/domains/settings/api/settings.ts)
- [`frontend/src/domains/settings/api/signalHub.test.ts`](../../../../frontend/src/domains/settings/api/signalHub.test.ts)
- [`frontend/src/domains/settings/api/signalHub.ts`](../../../../frontend/src/domains/settings/api/signalHub.ts)
- [`frontend/src/domains/settings/components/SignalHubSettings.boundary.test.ts`](../../../../frontend/src/domains/settings/components/SignalHubSettings.boundary.test.ts)
- [`frontend/src/domains/settings/components/signalHubSettingsPresentation.ts`](../../../../frontend/src/domains/settings/components/signalHubSettingsPresentation.ts)
- [`frontend/src/domains/settings/components/useSignalHubSettingsController.ts`](../../../../frontend/src/domains/settings/components/useSignalHubSettingsController.ts)
- [`frontend/src/domains/settings/lib/signalHubReplay.test.ts`](../../../../frontend/src/domains/settings/lib/signalHubReplay.test.ts)
- [`frontend/src/domains/settings/lib/signalHubReplay.ts`](../../../../frontend/src/domains/settings/lib/signalHubReplay.ts)
- [`frontend/src/domains/settings/queries/useSettingsQuery.ts`](../../../../frontend/src/domains/settings/queries/useSettingsQuery.ts)
- [`frontend/src/domains/settings/queries/useSignalHubQuery.test.ts`](../../../../frontend/src/domains/settings/queries/useSignalHubQuery.test.ts)
- [`frontend/src/domains/settings/queries/useSignalHubQuery.ts`](../../../../frontend/src/domains/settings/queries/useSignalHubQuery.ts)
- [`frontend/src/domains/settings/stores/settings.ts`](../../../../frontend/src/domains/settings/stores/settings.ts)
- [`frontend/src/domains/settings/types/settings.ts`](../../../../frontend/src/domains/settings/types/settings.ts)
- [`frontend/src/domains/settings/types/signalHub.ts`](../../../../frontend/src/domains/settings/types/signalHub.ts)
- [`frontend/src/domains/settings/views/SettingsPage.signalHub.boundary.test.ts`](../../../../frontend/src/domains/settings/views/SettingsPage.signalHub.boundary.test.ts)
- [`frontend/src/domains/tasks/api/tasks.ts`](../../../../frontend/src/domains/tasks/api/tasks.ts)
- [`frontend/src/domains/tasks/queries/useTasksQuery.ts`](../../../../frontend/src/domains/tasks/queries/useTasksQuery.ts)
- [`frontend/src/domains/tasks/stores/tasks.ts`](../../../../frontend/src/domains/tasks/stores/tasks.ts)

## Кандидаты на drift

Из предоставленного контекста видимых расхождений кода, документации или ADR не обнаружено. Все описания строго соответствуют встроенному коду. Файлы, частично обрезанные (`signalHub.ts`, `useSignalHubSettingsController.ts`), не позволяют подтвердить их полноту, но задокументированные части не противоречат друг другу.
