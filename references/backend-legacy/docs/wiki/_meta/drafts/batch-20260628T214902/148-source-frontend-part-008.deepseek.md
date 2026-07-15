### Summary / Резюме

Предлагается создать страницу `components/frontend.md` в русской Obsidian‑вики. Страница систематизирует устройство фронтенд‑части домена коммуникаций (`frontend/src/domains/communications/`) на основе фактического исходного кода. Описываются ключевые слои: TanStack‑хуки запросов (`queries`), Pinia‑хранилище UI‑состояния (`stores`), композабл‑контроллеры и представления (`views`), а также типовая модель (`types`). Каждое утверждение подкреплено встроенными файлами исходного кода. Страница помогает разработчику быстро ориентироваться в архитектуре.

---

### Proposed pages / Предлагаемые страницы

**Путь**: `components/frontend.md`

```markdown
# Компоненты фронтенда

## Обзор

Фронтенд‑часть домена `communications` отвечает за работу с почтой, Telegram‑чатами и WhatsApp‑беседами.  
Использует **Vue 3 Composition API**, **Pinia** для UI‑состояния и **TanStack Vue Query** для серверного состояния.  
Даты форматируются с помощью **date-fns**.

Исходный код организован в каталогах:

- `queries/` — хуки TanStack‑запросов к API,
- `stores/` — Pinia‑хранилище UI,
- `views/` — Vue‑компоненты и композабл‑контроллеры,
- `types/` — TypeScript‑типы и модели,
- `helpers/`, `forms/` — вспомогательные функции и валидация (не включены в данный пакет контекста).

## Модули запросов (queries)

Запросы к коммуникационному API построены на хуках **`useQuery`**, **`useInfiniteQuery`** и **`useMutation`** из TanStack Vue Query. Каждая группа определяет собственные **ключи запросов** для кеширования и инвалидации.

### Ключи Telegram‑запросов

```ts
export const telegramBusinessQueryKeys = {
  chats:          ['communications', 'telegram', 'chats'],
  chatDetail:     ['communications', 'telegram', 'chat-detail'],
  chatMembers:    ['communications', 'telegram', 'chat-members'],
  messages:       ['communications', 'telegram', 'messages'],
  topics:         ['communications', 'telegram', 'topics'],
  topicMessages:  ['communications', 'telegram', 'topic-messages'],
  search:         ['communications', 'telegram', 'search'],
}
```

### Ключи WhatsApp‑запросов

```ts
export const whatsappBusinessQueryKeys = {
  conversations:      ['communications', 'whatsapp', 'conversations'],
  conversationDetail: ['communications', 'whatsapp', 'conversation-detail'],
  chatMembers:        ['communications', 'whatsapp', 'chat-members'],
  messages:           ['communications', 'whatsapp', 'messages'],
  search:             ['communications', 'whatsapp', 'search'],
}
```

### Хуки чтения

Все хуки оборачивают параметры через `MaybeRefOrGetter`, используют `computed` для реактивных ключей и управляют состоянием `enabled`.

**Telegram**

- `useTelegramChatsQuery` — список чатов.
- `useTelegramChatDetailQuery` — детали одного чата.
- `useTelegramChatMembersQuery` — участники чата с постраничной загрузкой (инфинитный запрос через `useInfiniteQuery`).
- `useTelegramMessagesQuery` — сообщения.
- `useTelegramDialogSearchQuery` — поиск диалогов.
- `useTelegramMessageSearchQuery` — поиск сообщений.
- `useTelegramMediaSearchQuery` — поиск медиа.
- `useTelegramPinnedMessagesQuery` — закреплённые сообщения.
- `useTelegramTopicsQuery` — список топиков форума.
- `useTelegramTopicMessagesQuery` — сообщения внутри топика.
- `useTelegramTopicSearchQuery` — поиск топиков.
- `useTelegramMessageVersionsQuery` — версии сообщения.
- `useTelegramMessageTombstonesQuery` — «надгробия» сообщений.

**WhatsApp**

- `useWhatsappBusinessConversationsQuery` — список бесед.
- `useWhatsappConversationDetailQuery` — детали беседы.
- `useWhatsappConversationMembersQuery` — участники беседы (инфинитный запрос).
- `useWhatsappBusinessMessagesQuery` — сообщения.
- `useWhatsappMessageSearchQuery` — поиск сообщений.
- `useWhatsappMediaSearchQuery` — поиск медиа.
- `useWhatsappPinnedMessagesQuery` — закреплённые сообщения.
- `useWhatsappMessageReactionsQuery` — реакции к сообщению.

**Мутации для WhatsApp** (видимые в обрезанной части файла):

- `useSendWhatsappMessageMutation`
- `useReplyWhatsappMessageMutation`
- `useForwardWhatsappMessageMutation`
- `useEditWhatsappMessageMutation`
- `useDeleteWhatsappMessageMutation`
- `usePinWhatsappMessageMutation`
- `useAddWhatsappReactionMutation`

(Также объявлены `useRemoveWhatsappReactionMutation` и другие, но их полное тело не поместилось в контекст.)

Мутации после успешного выполнения сбрасывают кеш через общую функцию `useInvalidateWhatsappBusinessState`.

**Почтовые запросы**

Центральный ре‑экспортный файл `useCommunicationsQuery.ts` собирает хуки из нескольких модулей:

- `mailActionQueries` — содержит мутации для AI‑ответа, проверки безопасности, переводов, меток и т.д.
- `callQueries` — запросы звонков (не включены в контекст).
- `mailCoreQueries` — основные запросы: **инфинитная загрузка тредов** (`useInfiniteQuery` с `fetchThreads`, курсорная пагинация, `getNextPageParam`, `select`), `useThreadMessagesQuery` и другие.
- `mailOperationQueries` — экспорт сообщений, управление пометками, удаление.
- `mailWorkspaceQueries` — `useSubscriptionsQuery`, `useTopSendersQuery`, `useCommunicationBlockersQuery` (используются в панели ресурсов).

Граничный тест `threadInfiniteQuery.boundary.test.ts` подтверждает, что:

- `mailCoreQueries.ts` содержит `useInfiniteQuery`,
- присутствует вызов `fetchThreads(toValue(accountId), 50, pageParam)`,
- реализована пагинация `getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined`,
- и трансформация `select: (data) => data.pages.flatMap((page) => page.items)`.

Тест `threadTranslationMutation.boundary.test.ts` фиксирует наличие `useTranslateThreadMutation`, использующей `translateThread(accountId, subject, targetLanguage, limit)`.

## Хранилище UI‑состояния (stores)

Хранилище — **Pinia**‑стор `useCommunicationsStore` с id `'communications-ui'`.

### Состояние

- `communicationMessages` — список сообщений (`CommunicationMessageSummary[]`).
- `selectedCommunicationMessageId`, `selectedConversationIndex`, `selectedCommunicationDetail` — выделенное сообщение и его детали.
- `selectedMessageIds`, `selectionAnchorMessageId`, `selectedMessageIdSet` (computed) — мультивыделение.
- `mailStateFilter` — фильтр по workflow‑состоянию, `mailLocalStateFilter` — фильтр локального состояния.
- `mailStateCounts` — счётчики состояний.
- `threads`, `selectedThread`, `selectedThreadId` (computed) — список тредов.
- `drafts` — черновики.
- `mailSyncStatuses`, `selectedMailSyncSettings` — статусы синхронизации.
- `mailboxHealth`, `topSenders` — здоровье ящика и топ‑отправители.
- `composeForm`, `isComposeOpen`, `isSendingMessage`, `composeSendError` — форма создания письма.
- `isMailActionRunning`, `mailActionStatus`, `mailActionError`, `lastMessageExport` — статус выполняемых действий.
- `messageSearchQuery` — поисковый запрос.
- `communicationsNavigatorMode`, `communicationsInspectorMode`, `activeMessageContextTab` — UI‑режимы.
- `communicationProjects`, `communicationTasks` — проекты и задачи (связанные с сообщениями).

### Методы

- `setMessages`, `selectMessage`, `selectMessageId`, `toggleMessageSelection`, `selectMessageRange`, `selectVisibleMessages`, `clearMessageSelection` — управление выделением.
- `openCompose`, `closeCompose`, `updateComposeForm`, `openSendReview`, `closeSendReview` — управление compose‑формой.
- `setStateFilter`, `setLocalStateFilter`, `setStateCounts` — фильтрация.
- `setMailSyncStatuses`, `setMailSyncStatusMessage`, `setMailSyncError`, `setIsMailSyncBusy` — синхронизация.
- `setThreads`, `selectThread`, `clearSelectedThread` — треды.
- `setMailActionRunning`, `setMailActionStatus`, `setMailActionError`, `setLastMessageExport` — статус действий.
- `setNavigatorMode`, `setExpandedContactKey`, `setInspectorMode`, `setActiveMessageContextTab` — навигация.

### Маппинг секций

Из теста `communications.test.ts` известны прямые и обратные маппинги:

| UI‑секция      | Workflow‑состояние |
| -------------- | ----------------- |
| `'unified'`    | `''`              |
| `'inbox'`      | `'new'`           |
| `'needs_reply'`| `'needs_action'`  |
| `'waiting'`    | `'waiting'`       |
| `'done'`       | `'done'`          |
| `'archived'`   | `'archived'`      |

Обратная функция: `communicationWorkflowStateSectionId`.

### Мультивыделение

- При добавлении к выделению `extendRange = true` вызывается выбор диапазона от якорного сообщения.
- `selectVisibleMessages` заменяет всё выделение переданным списком уникальных идентификаторов.
- При вызове `setMessages` выделение очищается от id, отсутствующих в новом списке.

Выбор почтового аккаунта: если `selectedMailAccountId` не задан, при получении статусов синхронизации автоматически выбирается первый аккаунт.

## Представления и контроллеры (views)

### `CommunicationsPage.vue`

Главная страница коммуникаций. Шаблон включает:

- модальное окно `AccountSetupModal`,
- панели звонков (`CommunicationsCallsPanel`) для секций `'calls'` и `'meetings'`,
- полосу папок `CommunicationFolderStrip` с передачей `activeFolderId` и `selectedMailAccountId`,
- панель поиска вложений `AttachmentSearchPanel`,
- список сообщений с поддержкой клавиатурного выделения (`@select-visible`, `@clear-selection`),
- навигатор тредов с атрибутами `threads`, `:has-thread-next-page`, `:is-fetching-thread-next-page`, `:selected-thread-id`, событиями `@select-thread` и `@load-more-threads`,
- детальный просмотр треда (`selectedThreadMessages`, `:thread-messages`, события ответа и сохранения черновика),
- outbox‑статус `OutboxStatusStrip`,
- обработчики: `@generate-ai-reply`, `@apply-ai-reply`, `@review-security`, `@review-recipients`, `@reply-all`, `@forward-message`, `@redirect-message`, `@mark-message-read`, `@mark-message-unread`, `@delete-from-provider`, `@send-bilingual-reply`.

**Важно**: вся логика вынесена в контроллер `useCommunicationsPageController`. В `CommunicationsPage.vue` **нет** прямых вызовов `fetch`, `ApiClient`, `watch(...)`, `onMounted`, `useMailListQuery`, `useBulkMessageActionMutation` и импортов из `../components/`. Контроллер не содержит импортов `'../components/'` и прямых fetch‑операций.

### `useCommunicationsPageController.ts`

Композабл‑контроллер:

- Подключает основные запросы: `useMailListQuery` (с учётом аккаунта, фильтра, channel‑kind, поиска, localState), `useMessageQuery`, `useStateCountsQuery`, `useSyncStatusesQuery`, `useDraftsQuery`, `useMailboxHealthQuery`, `useConversationsQuery`, `useThreadMessagesQuery`.
- Использует `useFolderMailList` для работы с папками.
- Использует `useOutboxStatusStrip` для панели исходящих.
- Использует `useMailResourceOverview` для подписок, топ‑отправителей и блокировщиков.
- Реактивно обновляет Pinia‑хранилище через `watch`-наблюдатели за данными запросов.
- Управляет навигацией: `selectSection` (через маппинг `communicationSectionWorkflowState`), `handleSavedSearchSelect`, `handleFolderSelect`, `handleFolderDeleted`.
- Пагинация: `handleLoadMoreMessages`, `handleLoadMoreThreads`, `handleLoadMoreDrafts`.
- Bulk‑действия: `handleBulkAction` с мутацией `useBulkMessageActionMutation`.
- Действия над выбранным сообщением делегированы в `useSelectedMessageActions`.
- Действия с тредами — в `useThreadReplyActions`.
- Действия синхронизации — в `useMailSyncActions`.

### `useSelectedMessageActions.ts`

Контроллер действий над текущим выделенным сообщением:

- Простые ответы: `handleReply`, `handleReplyAll`, `handleForwardMessage` (открывают compose‑форму).
- `handleRedirectMessage` — переадресация через `useRedirectMessageMutation`.
- `handleBilingualReplySend` — открытие compose с переводом (при `response.send_ready`).
- `handleTogglePin`, `handleToggleImportant`, `handleMute` — переключение флагов.
- `handleExportMessage(format)` — экспорт через `useExportMessageMutation`, сохраняет `lastMessageExport`.
- `handleMarkMessageRead`, `handleMarkMessageUnread` — пометка прочтения.
- `handleDeleteFromProvider` — удаление на стороне провайдера.
- `handleAddLabel`, `handleRemoveLabel`, `handleSnoozeMessage(until)` — метки и отложенное напоминание.
- `handleAnalyze` — AI‑анализ (`useAnalyzeMessageMutation`).
- `handleTranslate` — перевод (`useTranslateMessageMutation`), результат записывается в `mailMessageInsight`.
- `handleGenerateAiReply` — генерация AI‑ответа (`useGenerateAiReplyMutation`), результат в `mailMessageInsight`.
- `handleApplyAiReply` — открытие compose с телом AI‑ответа.
- `handleReviewSecurity` — проверка SPF/DKIM/DMARC через `useReviewMessageSecurityMutation`.
- `handleReviewRecipients` — подсказки CC через `useReviewMessageRecipientsMutation`.
- `handleCreateTask`, `handleCreateNote` — извлечение задач/заметок (`useExtractMessageTasksMutation`, `useExtractMessageNotesMutation`).

Все асинхронные действия обёрнуты в универсальный `runSelectedMessageAction`, управляющий состоянием `isMailActionRunning`, `mailActionStatus` и `mailActionError`. Контроллер не импортирует компоненты, fetch или ApiClient.

### `useMailSyncActions.ts`

- `handleSyncNow` — запускает синхронизацию через `useRunMailSyncNowMutation`, после завершения перезагружает список, состояние почтового ящика, счётчики, статусы синхронизации.
- `handleUpdateSyncSettings` — сохраняет настройки через `useUpdateMailSyncSettingsMutation`.
- `clearSyncStatus`, `loadInitialData` — сброс ошибок и начальная загрузка.
- Использует `useMailSyncSettingsQuery` для получения текущих настроек.

### `useMailResourceOverview.ts`

Предоставляет данные для панели ресурсов:

- `subscriptions` (через `useSubscriptionsQuery`),
- `topSenders` (через `useTopSendersQuery`),
- `blockers` (через `useCommunicationBlockersQuery`).

Имеет функции постраничной подгрузки: `handleLoadMoreSubscriptions`, `handleLoadMoreTopSenders`.

### `useThreadReplyActions.ts`

(Упоминается в граничных тестах `CommunicationsPage.boundary.test.ts`). Содержит мутации `useSaveDraftMutation`, `useSendMailMutation`, вспомогательные `buildComposeDraftPayload`, `composeFormToSendRequest`, и обработчики `handleReplyToThreadMessage`, `handleSaveThreadReplyDraft`, `handleSendThreadReply`. Не импортирует компоненты, fetch или ApiClient.

## Типовая модель (types)

Все типы сосредоточены в `types/communications.ts` и специализированных файлах.

### Базовые сущности

- `WorkflowState`: `'new' | 'reviewed' | 'needs_action' | 'waiting' | 'done' | 'archived' | 'muted' | 'spam'`
- `LocalMessageState`: `'active' | 'trash' | 'all'`
- `CommunicationMessageSummary` — сводка сообщения: идентификаторы, тема, отправитель, получатели, превью тела, время, канал, состояние workflow, AI‑категория/сводка, метаданные, количество вложений, локальное состояние.
- `CommunicationMessageDetailItem` — расширенная версия с полным телом, HTML, `local_state_reason`.
- `CommunicationMessageDetailResponse`: `{ message: ..., attachments: ... }`
- `CommunicationThread` / `CommunicationThreadSummary` — треды (поток сообщений).
- `ThreadMessage` — сообщение треда с вложениями.

### AI

- `CommunicationAiState`: `'NEW' | 'PROCESSING' | 'PROCESSED' | 'REVIEW_REQUIRED' | 'FAILED' | 'ARCHIVED'`
- `MessageAnalyzeResponse` — результат анализа: категория, сводка, ключевые пункты, элементы действий, риски, дедлайны, кандидаты событий/персон/организаций/документов/соглашений.

### Вложения

- `AttachmentScanStatus`: `'not_scanned' | 'clean' | 'suspicious' | 'malicious' | 'failed'`
- `AttachmentPreviewResponse` — превью с `preview_kind` (text, image, audio, video, pdf) и `data_url`.
- `AttachmentTranslationResponse` — перевод текста вложения (язык, уверенность, текст, модель).
- `ArchiveInspectionReport` — инспекция вложенных архивов (zip).

### Сертификаты

- `MailCertificate` — сертификат с полями: `cert_type` (smime, pgp, pdf_sign, cades, xades, gost_sign, unknown), `provider` (fnmt, dnie, cryptopro, gost, apple_keychain, pkcs12, yubikey, usb_token, other), `storage_kind` (os_keychain, encrypted_vault, pkcs12_file, pfx_file, smart_card, usb_token, external_vault), `trust_status` (trusted, untrusted, expired, revoked, pending_verification, self_signed).

### Двуязычные ответы

- `bilingualReplyToneOptions`: `'formal' | 'business' | 'friendly' | 'short' | 'detailed'`
- `BilingualReplyFlowResponse` — включает оригинал, перевод, черновик ответа на русском, обратный перевод.

### Операции с почтой

- `CommunicationDraft` — черновик (статусы: draft, scheduled, sending, sent, failed).
- `SendCommunicationRequest` / `SendCommunicationResponse` — отправка письма (SMTP, outbox, undo‑дедлайн).
- `CommunicationOutboxItem` — элемент исходящих (статусы: queued, scheduled, sending, sent, failed, canceled).
- `BulkMessageAction` — массовые действия: mark_read, mark_unread, archive, trash, restore, pin, unpin, important, not_important, add_label, remove_label, snooze.

### Каналы провайдеров

- `CommunicationProviderChannelKind`: `'telegram_user' | 'telegram_bot' | 'whatsapp_web' | string`
- `CommunicationProviderConversation` — беседа (для Telegram содержит `telegram_chat_id`, для WhatsApp — `conversation_id`).
- `CommunicationProviderTopic` — топик форума Telegram.

### Отчёты о прочтении

- `CommunicationReadReceipt` — запись о прочтении с `receipt_kind: 'read'`.

### Шаблоны

- `CommunicationTemplate` — шаблон письма с объявленными и неиспользуемыми переменными.
- `RichTemplateRenderResponse` — результат рендеринга (отсутствующие/неразрешённые переменные).
- `RichTemplateMailMergePreviewResponse` — предпросмотр слияния (сколько готово, сколько блокировано).

### Папки и сохранённые поиски

- `CommunicationFolder` — папка (имя, цвет, счётчик сообщений).
- `CommunicationSavedSearch` — сохранённый поиск с query, фильтрами, флагом `is_smart_folder`.

### Прочее

- `CommunicationPersona` — персоны аккаунтов.
- `ThreadTranslationResponse` — перевод треда (массив `ThreadTranslationItem`).

## Тестирование

Граничные тесты (`*.boundary.test.ts`) проверяют контракты между составными частями:

- `threadInfiniteQuery.boundary.test.ts` — убеждается, что `mailCoreQueries.ts` содержит `useInfiniteQuery` с конкретными параметрами пагинации и `useThreadMessagesQuery`.
- `threadTranslationMutation.boundary.test.ts` — проверяет наличие `useTranslateThreadMutation` и вызова `translateThread`.
- `CommunicationsPage.boundary.test.ts` — комплексная проверка архитектуры страницы:
  - использование контроллера, отсутствие бизнес‑логики в `.vue`,
  - секции `calls` и `meetings`,
  - полоса папок и поиска вложений с привязкой аккаунта,
  - клавиатурное выделение и range‑select,
  - постраничная загрузка тредов и сообщений,
  - outbox‑статус,
  - двуязычные ответы,
  - разделение логики синхронизации (`useMailSyncActions`), действий над сообщением (`useSelectedMessageActions`) и ответов в треде (`useThreadReplyActions`).

Юнит‑тест `communications.test.ts` покрывает маппинг секций, мультивыделение и выбор почтового аккаунта.
```

---

### Source coverage / Покрытие источников

| Исходный файл | Какие факты из него покрыты |
|---|---|
| `telegramBusinessQueries.ts` | Ключи `telegramBusinessQueryKeys`, хуки `useTelegramChatsQuery`, `useTelegramChatDetailQuery`, `useTelegramChatMembersQuery`, `useTelegramMessagesQuery`, `useTelegramDialogSearchQuery`, `useTelegramMessageSearchQuery`, `useTelegramMediaSearchQuery`, `useTelegramPinnedMessagesQuery`, `useTelegramTopicsQuery`, `useTelegramTopicMessagesQuery`, `useTelegramTopicSearchQuery`, `useTelegramMessageVersionsQuery`, `useTelegramMessageTombstonesQuery`; все используют `MaybeRefOrGetter`, `computed`, `enabled` и TanStack‑хуки. |
| `threadInfiniteQuery.boundary.test.ts` | Подтверждено, что `mailCoreQueries.ts` содержит `useInfiniteQuery`, вызов `fetchThreads(toValue(accountId), 50, pageParam)`, `getNextPageParam`, `select` и `useThreadMessagesQuery`. |
| `threadTranslationMutation.boundary.test.ts` | Подтверждено наличие `useTranslateThreadMutation` и вызова `translateThread(accountId, subject, targetLanguage, limit)` из API. |
| `useCommunicationsQuery.ts` | Ре‑экспорт `mailActionQueries`, `callQueries`, `mailCoreQueries`, `mailOperationQueries`, `mailWorkspaceQueries` и типов `NullableQueryParam`, `QueryParam`. |
| `whatsappBusinessQueries.ts` | Ключи `whatsappBusinessQueryKeys`, хуки `useWhatsappBusinessConversationsQuery`, `useWhatsappConversationDetailQuery`, `useWhatsappConversationMembersQuery`, `useWhatsappBusinessMessagesQuery`, `useWhatsappMessageSearchQuery`, `useWhatsappMediaSearchQuery`, `useWhatsappPinnedMessagesQuery`, `useWhatsappMessageReactionsQuery`; мутации `useSendWhatsappMessageMutation`, `useReplyWhatsappMessageMutation`, `useForwardWhatsappMessageMutation`, `useEditWhatsappMessageMutation`, `useDeleteWhatsappMessageMutation`, `usePinWhatsappMessageMutation`, `useAddWhatsappReactionMutation`; функция `useInvalidateWhatsappBusinessState`. |
| `communications.test.ts` | Маппинг `communicationSectionWorkflowState` и `communicationWorkflowStateSectionId`; поведение мультивыделения (toggle, range, selectVisibleMessages); автоматический выбор первого аккаунта при отсутствии выбранного. |
| `communications.ts` | Структура Pinia‑стора (список полей состояния и методов); маппинг секций (подтверждён тестом). |
| `aiState.ts` | Тип `CommunicationAiState` и его допустимые значения, типы `CommunicationAiStateRecord` и `CommunicationAiStateTransitionRequest`. |
| `attachments.ts` | Типы `AttachmentScanStatus`, `AttachmentPreviewResponse`, `AttachmentTranslationResponse`, `ArchiveInspectionReport`. |
| `bilingualReplyFlow.ts` | Константа `bilingualReplyToneOptions`, типы `BilingualReplyFlowResponse`, `BilingualReplyTone`. |
| `certificates.ts` | Типы `CertificateType`, `CertificateProvider`, `CertificateStorageKind`, `CertificateTrustStatus`, `MailCertificate`; константы‑списки опций для типов, провайдеров, storage‑kind и trust‑status. |
| `communications.ts` | Типы `WorkflowState`, `LocalMessageState`, `CommunicationMessageSummary`, `CommunicationMessageDetailItem`, `CommunicationMessageDetailResponse`, `CommunicationThread`, `CommunicationThreadSummary`, `ThreadMessage`, `MailSyncStatus`, `MailSyncSettings`, `MessageAnalyzeResponse`, `WorkflowActionKind`, `WorkflowActionResponse`, `TranslationResponse`, `AiReplyResponse`, `MessageAuthCheckResponse`, `MessageExportResponse`, `BulkMessageAction` и др. (до обрезанной части включительно). |
| `folders.ts` | Тип `CommunicationFolder` и связанные списки ответов. |
| `mailOperations.ts` | Типы `CommunicationDraft`, `SendCommunicationRequest`/`Response`, `CommunicationOutboxItem`, `BulkMessageAction` и его допустимые значения. |
| `multilingual.ts` | Типы `ThreadTranslationItem` и `ThreadTranslationResponse`. |
| `personas.ts` | Тип `CommunicationPersona`. |
| `providerChannels.ts` | Типы `CommunicationProviderChannelKind`, `CommunicationProviderConversation`, `CommunicationProviderTopic`, `CommunicationProviderMessage`, `CommunicationRawEvidenceResponse`. |
| `readReceipts.ts` | Типы `CommunicationReadReceipt` и `NewCommunicationReadReceipt`. |
| `savedSearches.ts` | Тип `CommunicationSavedSearch`, запросы/мутации для сохранённых поисков. |
| `templates.ts` | Типы `CommunicationTemplate`, `RichTemplateRenderResponse`, `RichTemplateMailMergePreviewResponse`. |
| `CommunicationsPage.boundary.test.ts` | Архитектурные ограничения: использование контроллера, отсутствие бизнес‑логики в .vue, поддержка секций calls/meetings, интеграция папок и поиска вложений, клавиатурное выделение, постраничная загрузка тредов и сообщений, outbox‑статус, двуязычные ответы, разделение логики синхронизации/действий/тредовых ответов. |
| `useCommunicationsPageController.ts` | Хуки запросов, используемые контроллером; реактивные `watch` для синхронизации с Pinia; управление пагинацией, навигацией по секциям/сохранённым поискам/папкам; bulk‑действия; делегирование в `useSelectedMessageActions` и `useThreadReplyActions`. |
| `useMailResourceOverview.ts` | Использование `useSubscriptionsQuery`, `useTopSendersQuery`, `useCommunicationBlockersQuery`; пагинация подписок и отправителей. |
| `useMailSyncActions.ts` | Операции `handleSyncNow`, `handleUpdateSyncSettings`, `clearSyncStatus`, `loadInitialData`; использование `useRunMailSyncNowMutation`, `useUpdateMailSyncSettingsMutation`, `useMailSyncSettingsQuery`. |
| `useSelectedMessageActions.ts` | Все перечисленные действия (`handleReply`, `handleReplyAll`, `handleForwardMessage`, `handleRedirectMessage`, `handleBilingualReplySend`, `handleTogglePin`, `handleToggleImportant`, `handleMute`, `handleExportMessage`, `handleMarkMessageRead`, `handleMarkMessageUnread`, `handleDeleteFromProvider`, `handleAddLabel`, `handleRemoveLabel`, `handleSnoozeMessage`, `handleAnalyze`, `handleTranslate`, `handleGenerateAiReply`, `handleApplyAiReply`, `handleReviewSecurity`, `handleReviewRecipients`, `handleCreateTask`, `handleCreateNote`); обёртка `runSelectedMessageAction`; отсутствие импортов компонентов, fetch и ApiClient. |

---

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений между кодом, документацией и ADR не видно. Ни один из встроенных файлов не содержит комментариев, ссылок на устаревшие соглашения или противоречащих друг другу утверждений. Граничные тесты подтверждают актуальную структуру кода. Дрифт не обнаружен.
