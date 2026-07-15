---
chunk_id: 143-source-frontend-part-003
batch_id: batch-20260628T214902
group: frontend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 143-source-frontend-part-003 — frontend/source

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Чанк содержит API-функции и boundary-тесты компонентов из домена `communications` (Telegram, WhatsApp, Email, сохранённые поиски, треды, read-receipts, и т.д.). Необходимо дополнить русскую wiki-страницу `components/frontend.md` документацией, описывающей фактические точки API (HTTP-методы, URL, параметры, преобразования данных) и ключевые характеристики компонентов Vue (используемые библиотеки, пропсы, события, отсутствие прямых вызовов API), основанные исключительно на предоставленных исходниках и тестах.

## Предложенные страницы

#### `components/frontend.md`

```markdown
# Frontend

## API-слой коммуникаций

Модуль `frontend/src/domains/communications/api/` содержит функции для взаимодействия с API коммуникаций через `ApiClient`. Все функции делегируют HTTP-запросы через `ApiClient.instance` или через связующие (connect) функции из `connectCommunications`.

### readReceipts

- `recordReadReceipt(request: NewCommunicationReadReceipt): Promise<CommunicationReadReceipt>`
  - Метод: `POST`
  - URL: `/api/v1/communications/read-receipts`
  - Тело: `request` (тип `NewCommunicationReadReceipt`)

### savedSearchApi

- `fetchSavedSearches(smartFolder?, accountId?, limit=500, cursor?): Promise<SavedSearchListResponse>`
- `createSavedSearch(request: SavedSearchInput): Promise<CommunicationSavedSearch>`
- `updateSavedSearch(savedSearchId, request: SavedSearchUpdate): Promise<CommunicationSavedSearch>`
- `deleteSavedSearch(savedSearchId): Promise<SavedSearchDeleteResponse>`

Используют connect-функции из `connectCommunications`.

### sendApi

- `sendEmail(request: SendCommunicationRequest): Promise<SendCommunicationResponse>`
  - Использует `sendCommunicationConnect`.
- `redirectMessage(messageId, request: RedirectMessageRequest): Promise<SendCommunicationResponse>`
  - Использует `redirectMessageConnect`.

### telegramBusinessApi

Набор функций для работы с Telegram-сообщениями и чатами через агрегирующий API `/api/v1/communications/...`. Используется `channel_kind=telegram` для фильтрации.

- `fetchTelegramBusinessChats(accountId?, limit)`
  - `GET /api/v1/communications/conversations?limit=...`
- `fetchTelegramBusinessChatDetail(conversationId)`
  - `GET /api/v1/communications/conversations/:id`
- `fetchTelegramBusinessChatMembers(conversationId, ...)`
  - `GET /api/v1/communications/conversations/:id/members?...`
- `fetchTelegramBusinessMessages(accountId?, providerChatId?, limit)`
  - `GET /api/v1/communications/messages?channel_kind=telegram...`
  - Преобразует `CommunicationMessageSummary` в `TelegramMessage` через `communicationMessageToTelegramMessage` (поля: `provider_record_id` → `provider_message_id`, `subject` → `chat_title`, `body_text_preview` → `text`, `message_metadata` → `metadata`).
- `searchTelegramBusinessChats({q, account_id?, limit?})`
  - `GET /api/v1/communications/conversations/search?q=...`
- `searchTelegramBusinessMessages({q, account_id?, provider_chat_id?, limit?})`
  - `GET /api/v1/communications/search/messages?q=...`
- `searchTelegramBusinessMedia({q?, account_id?, provider_chat_id?, kind?, limit?})`
  - `GET /api/v1/communications/search/media?...`
- `fetchTelegramBusinessPinnedMessages({telegram_chat_id, limit?})`
  - `GET /api/v1/communications/conversations/:id/pinned-messages?...`
- `sendTelegramBusinessMessage({account_id, provider_chat_id, text})`
  - `POST /api/v1/communications/conversations/:id/messages`
  - Тело: `{account_id, text}`
- `replyToTelegramBusinessMessage({message_id, text})`
  - `POST /api/v1/communications/messages/:id/reply`
  - Тело: `{text}`
- `forwardTelegramBusinessMessage({message_id, provider_chat_id})`
  - `POST /api/v1/communications/messages/:id/forward`
  - Тело: `{conversation_id}`
- `editTelegramBusinessMessage(params)`
  - `PATCH /api/v1/communications/messages/:id`
- `deleteTelegramBusinessMessage(params)`
  - `DELETE` с телом на `/api/v1/communications/messages/:id`
- `restoreTelegramBusinessMessageVisibility(params)`
  - `POST /api/v1/communications/messages/:id/restore-visibility`
- `pinTelegramBusinessMessage({message_id})`
  - `POST /api/v1/communications/messages/:id/pin` (пустое тело)
- `markTelegramBusinessMessageRead({message_id, account_id, provider_chat_id})`
  - `POST /api/v1/communications/messages/:id/mark-read`
- `fetchTelegramBusinessMessageVersions(messageId)`
  - `GET /api/v1/communications/messages/:id/versions`
- `fetchTelegramBusinessMessageTombstones(messageId)`
  - `GET /api/v1/communications/messages/:id/tombstones`
- `fetchTelegramBusinessReplyChain(messageId)`
  - `GET /api/v1/communications/messages/:id/reply-chain`
- `fetchTelegramBusinessForwardChain(messageId)`
  - `GET /api/v1/communications/messages/:id/forward-chain`
- `fetchTelegramBusinessReactions(messageId)`
  - `GET /api/v1/communications/messages/:id/reactions`
- `addTelegramBusinessReaction(messageId, request)`
  - `POST /api/v1/communications/messages/:id/reactions`
- `removeTelegramBusinessReaction(messageId, request)`
  - `DELETE /api/v1/communications/messages/:id/reactions` с query-параметрами

> Примечание: исходный файл `telegramBusinessApi.ts` обрезан в контексте; в тестах также присутствует функция `searchTelegramBusinessTopics`, которая опрашивает `GET /api/v1/communications/topics/search?q=...&telegram_chat_id=...&limit=...`. Полный перечень методов может быть шире.

### whatsappBusinessApi

Аналог для WhatsApp Web. Использует `channel_kind=whatsapp_web`. Возвращаемые типы адаптированы под WhatsApp.

- `fetchWhatsappWebBusinessConversations(accountId?, limit)`
  - `GET /api/v1/communications/conversations?...channel_kind=whatsapp_web`
  - Фильтрует результаты с помощью `isWhatsappConversation`, исключая не-WhatsApp чаты.
- `fetchWhatsappWebBusinessConversationDetail(conversationId)`
  - `GET /api/v1/communications/conversations/:id`
- `fetchWhatsappWebBusinessConversationMembers(conversationId, ...)`
  - `GET /api/v1/communications/conversations/:id/members?` (возвращает `TelegramChatMemberListResponse`)
- `fetchWhatsappWebBusinessMessages(accountId?, providerChatId?, limit)`
  - `GET /api/v1/communications/messages?...channel_kind=whatsapp_web`
  - Преобразует `CommunicationMessageSummary` в `WhatsappWebMessage`.
- `searchWhatsappWebBusinessMessages({q, account_id?, provider_chat_id?, limit?})`
  - `GET /api/v1/communications/search/messages?...channel_kind=whatsapp_web`
- `searchWhatsappWebBusinessMedia({q?, account_id?, provider_chat_id?, kind?, limit?})`
  - `GET /api/v1/communications/search/media?...channel_kind=whatsapp_web`
- `fetchWhatsappWebBusinessPinnedMessages({conversation_id, limit?})`
  - `GET /api/v1/communications/conversations/:id/pinned-messages?...`
- `sendWhatsappBusinessMessage({account_id, provider_chat_id, text})`
  - `POST /api/v1/communications/conversations/:id/messages`
- `replyToWhatsappBusinessMessage({message_id, text})`
  - `POST /api/v1/communications/messages/:id/reply`
- `forwardWhatsappBusinessMessage({message_id, provider_chat_id})`
  - `POST /api/v1/communications/messages/:id/forward`
- `editWhatsappBusinessMessage(params)`
  - `PATCH /api/v1/communications/messages/:id`
- `deleteWhatsappBusinessMessage(params)`
  - `DELETE` с телом на `/api/v1/communications/messages/:id`
- `pinWhatsappBusinessMessage({message_id})`
  - `POST /api/v1/communications/messages/:id/pin`
- `pinWhatsappBusinessConversation({conversation_id})`
  - `POST /api/v1/communications/conversations/:id/pin`
- `unpinWhatsappBusinessConversation({conversation_id})`
  - `POST /api/v1/communications/conversations/:id/unpin`
- `archiveWhatsappBusinessConversation({conversation_id})`
  - `POST /api/v1/communications/conversations/:id/archive`
- `unarchiveWhatsappBusinessConversation({conversation_id})`
  - `POST /api/v1/communications/conversations/:id/unarchive`
- `muteWhatsappBusinessConversation({conversation_id})`
  - `POST /api/v1/communications/conversations/:id/mute`
- `unmuteWhatsappBusinessConversation({conversation_id})`
  - `POST /api/v1/communications/conversations/:id/unmute`
- `markWhatsappBusinessConversationRead({conversation_id})`
  - `POST /api/v1/communications/conversations/:id/read`
- `markWhatsappBusinessConversationUnread({conversation_id})`
  - `POST /api/v1/communications/conversations/:id/unread`
- Реакции: `fetchWhatsappBusinessReactions`, `addWhatsappBusinessReaction`, `removeWhatsappBusinessReaction` (аналогичны Telegram, см. тесты).

### threadApi

- `fetchThreads(accountId?, limit, cursor?)` → `fetchCommunicationThreadsConnect`
- `fetchThreadMessages(accountId, subject, limit)` → `fetchCommunicationThreadMessagesConnect`
- `translateThread(accountId, subject, targetLanguage, limit)` → `translateCommunicationThreadConnect`

## Компоненты Vue

Все компоненты, рассматриваемые в данном чанке, следуют архитектурному принципу: **никаких прямых вызовов `fetch` или `ApiClient` из шаблонов/логики компонентов**. Вместо этого используются:

- **TanStack Query** (`use...Query`, `use...Mutation`) для серверного состояния
- **Vee-Validate + Zod** для валидации форм
- **TanStack Table / TanStack Virtual** для таблиц и виртуального скролла
- **Reka UI** для примитивов (Sheet и др.)

Ниже приведены утверждения о компонентах, основанные исключительно на проверках в boundary-тестах, входящих в этот чанк.

### AttachmentSearchPanel

- Форма: Vee/Zod (`attachmentSearchForm`), используется `setFieldValue`.
- Данные: `useAttachmentSearchQuery`, таблица `useVueTable` + `getCoreRowModel`, виртуализация `useVirtualizer` с `fetchNextPage`/`hasNextPage`.
- Предзагрузка результатов: `useAttachmentSearchResultPrefetch`, триггерится по `mouseenter` и `focus`.
- Пропс: `accountId: string | null`.
- Событие submit: `@submit.prevent="submitSearch"`.
- **Не использует** `../api/communications` и `fetch(...)`.

### BilingualReplyPanel

- Форма: Vee/Zod (`bilingualReplyFlowForm`), `setFieldValue`.
- Мутация: `usePrepareBilingualReplyFlowMutation`.
- Поля: Original, Translation, Reply in Russian, Back Translation.
- Эмит: `send-bilingual-reply`.
- **Не использует** `../api/` и `fetch()`.

### BulkActionsBar

- Локальные метаданные: поддерживает действия `pin`, `unpin`, `important`, `not_important`, `add_label`, `remove_label`, `snooze`.
- Тип: `BulkActionCommand`, `BulkMessageActionRequest`.
- Отложенное время (snooze): `nextBusinessMorningIso()`.
- Команды с payload: например `{ action: 'add_label', label: 'Follow up' }`.

### CommunicationFolderStrip

- Форма: Vee/Zod (`mailFolderForm`).
- Виртуализация: TanStack Virtual, горизонтальная (`horizontal: true`), `useVirtualizer`, `fetchNextPage`, `hasNextPage`, `isFetchingNextPage`, обработчик скролла `handleFolderVirtualScroll`.
- CRUD папок: мутации `useCopyMessageToFolderMutation`, `useCreateCommunicationFolderMutation`, `useUpdateCommunicationFolderMutation`, `useDeleteCommunicationFolderMutation`, `useMoveMessageToFolderMutation`.
- Реордеринг: `useCommunicationFolderReorder`, drag-and-drop (`handleFolderDragOver`, `handleFolderDrop`, `handleDragStart`, `handleDragEnd`), `folderReorder.canHandleDragOver(event)`.
- Иерархия: `mailFolderHierarchyDeleteImpact`, `mailFolderParentPathOptions`, `splitCommunicationFolderName`, `validateCommunicationFolderParentPath`, `composeCommunicationFolderName`, предпросмотр пути папки.
- Диалоги: создание/удаление, описание диалога.
- События: `select` с `folder_id` при клике, перетаскивание сообщений на папку (используется `parseCommunicationMessageDragPayload`).
- **Не использует** `../api/communications` и `ApiClient`.

### CommunicationList

- Клавиатурное множественное выделение: `@keydown="handleKeydown"`.
- Атрибуты доступности: `tabindex="0"`, `role="listbox"`, `aria-multiselectable="true"`.
- Клавиши: `Space` (toggle), `a` с `metaKey/ctrlKey` (select all visible), `Escape` (clear), `ArrowDown/ArrowUp` (перемещение с Shift-диапазоном).
- Эмиты: `emit('toggleSelection', message_id, shiftKey)`, `emit('selectVisible', visibleMessageIds)`, `emit('clearSelection')`.
- **Не использует** `fetch()` и `ApiClient`.

### CommunicationListItem

- Drag-and-drop: сериализует выделенные сообщения через `createCommunicationMessageDragPayload(props.message.message_id, props.selectedMessageIds)`.
- Пропс: `selectedMessageIds: string[]`.
- Доступность: `role="option"`, `:aria-selected="isChecked || isSelected"`.

### CommunicationViewer

- Содержит `MessageBodyTab`, пересылает события `send-bilingual-reply`.
- AI-состояние: запрос `useMessageAiStateQuery`, мутация `useUpdateMessageAiStateMutation`, состояния `REVIEW_REQUIRED` и `FAILED`, панель AI.
- Действия с сообщением: `exportMessage`, `markMessageRead`, `markMessageUnread`, `deleteFromProvider`, `addLabel`, `removeLabel`, `snoozeMessage`, `replyAll`, `forwardMessage`, `redirectMessage`.
- Эмиты: `@send-bilingual-reply`, `@export-message`, `@mark-message-read`, `@mark-message-unread`, `@delete-from-provider`, `@add-label`, `@remove-label`, `@snooze-message`, `@reply-all`, `@forward-message`, `@redirect-message`.
- **Не использует** `fetch()` и `ApiClient`.

### CommunicationsActionBar

- Экспорт сообщений: `lastMessageExport`, `messageExportDownloadHref`, ссылка с `download` и текстом "Export ready".
- Включает стрипы: `MailResourceOverviewStrip`, `MailSyncSettingsStrip` (из `../../../shared/mailSync/`), `MailCertificateStrip`.
- Пагинация: `hasMoreDrafts`, `loadMoreDrafts`, аналогично для подписок, топ-отправителей, блокировщиков.
- Синхронизация: `syncSettings`, `updateSyncSettings`.
- **Не использует** `../api/`, `fetch()`, `ApiClient`.

### CommunicationsCallsPanel

- Данные: `useProviderCallsQuery`, `useProviderCallTranscriptQuery`.
- Режим: `mode: 'calls' | 'meetings'`. В режиме `meetings` источник — `'zoom'`, показываются `meetingParticipants`, `meetingRecordingRefs`, ссылка "Open join URL".
- **Не использует** `fetch()`.

### CommunicationsConversationList

- Список тредов: `threads: CommunicationThreadSummary[]`, `selectedThreadId`, `accountId`.
- Предзагрузка: `useThreadMessagesPrefetch`, `handleThreadPrefetch` по `mouseenter`/`focus`.
- Пагинация: `hasThreadNextPage`, `loadMoreThreads`.
- Эмиты: `selectThread`, `loadMoreThreads`.
- **Не использует** `fetch()` и `ApiClient`.

### CommunicationsDetailPane

- Содержит `CommunicationViewer` и перенаправляет его события.
- Для выбранного треда использует `ThreadConversationView` с `selectedThread`, `threadMessages`, `isThreadReplySending`, событиями `@open-message`, `@reply-to-message`, `@save-reply-draft`, `@send-reply`.
- Действия: `sendBilingualReply`, `exportMessage`, `addLabel`, `removeLabel`, `snoozeMessage`, `replyAll`, `forwardMessage`, `redirectMessage`, `markMessageRead`, `markMessageUnread`, `deleteFromProvider`.

### CommunicationsListPane

- Режим папок: пропс `isFolderMode`. Если не в режиме папок, рендерит `CommunicationList`.
- Пропсы/эмиты: `accountId`, `threads`, `selectedThreadId`, `hasThreadNextPage`, `selectThread`, `loadMoreThreads`.
- Множественное выделение: эмиты `selectVisible`, `clearSelection`.

### ComposeDrawer

- Мутации TanStack: `useSendMailMutation`, `useSaveDraftMutation`, `useDeleteDraftMutation`, `useComposeDraftAutosave`.
- UI: обёртка `Sheet` (из `../../../shared/ui/Sheet.vue`), `content-class="compose-drawer"`.
- Редактор: `RichComposeEditor`, режимы `'html', 'rich'` и `'html', 'source'`.
- Пикеры: `ComposeTemplatePicker` (эмиты `@apply`, `@saved`, `@deleted`), `ComposeSignaturePicker` (`@apply`).
- Вложения: `stagedAttachments`, `handleAttachmentFiles`, предупреждение, что загрузка вложений не подключена к отправке у провайдера.
- Стили: импорт `./ComposeDrawer.css`, отсутствует `<style scoped>`.
- **Не использует** `../api/communications`.

### ComposeSignaturePicker

- Запрос: `usePersonasQuery`.
- Эмит: `apply` с `persona.signature`.
- **Не использует** прямой API.

### ComposeTemplatePicker

- Запросы: `useRichTemplatesQuery`, мутации `useRenderRichTemplateMutation`, `useCreateRichTemplateMutation`, `useDeleteRichTemplateMutation`, `usePreviewRichTemplateMailMergeMutation`.
- Эмиты: `apply`, `saved`, `deleted`.
- Функциональность: выбор шаблона (`updateSelectedTemplate`), mail merge предпросмотр (`missingTemplateVariables`, `parseTemplateMailMergePreviewRows`, `resolveTemplateVariableValues`, `previewResult`), диагностика (`storedTemplateDiagnosticMessages`, `templateDiagnosticCount`), категории (`deriveTemplateLibraryCategories`), маппинг получателей (`TemplateRecipientMappingPanel`, `applyTemplateRecipientMapping`), диалог сохранения (`openSaveTemplate`, `preserveExisting: isSameTemplate`, кнопка "Save copy").
- Импорт CSS: `./ComposeTemplatePicker.css`.

### DraftStrip

- Виртуализация: TanStack Virtual (`useVirtualizer`, `draftVirtualizer`, `virtualDraftRows`).
- Пагинация: `hasMore`, `loadMore`, эмит `loadMore`.
- **Не использует** `../api/` и `fetch()`.

### MailCertificateStrip

- Запросы: `useMailCertificatesQuery`, `useExpiringMailCertificatesQuery`.
- Мутация: `useCreateMailCertificateMutation`, схема `certificateVeeValidationSchema`.
- UI: "Expiring certificates", "Add certificate", "Storage reference".
- **Не использует** `../api/`, `fetch()`, `ApiClient`.
```

## Покрытие источников

| Исходный файл | Покрытые факты |
|---|---|
| `frontend/src/domains/communications/api/readReceipts.ts` | Функция `recordReadReceipt`, HTTP-метод, URL, тип запроса и ответа. |
| `frontend/src/domains/communications/api/savedSearchApi.ts` | Функции `fetchSavedSearches`, `createSavedSearch`, `updateSavedSearch`, `deleteSavedSearch` с делегированием на connect-функции. |
| `frontend/src/domains/communications/api/sendApi.ts` | Функции `sendEmail` и `redirectMessage` с использованием connect-функций. |
| `frontend/src/domains/communications/api/telegramBusinessApi.ts` (обрезан) | Перечислены все видимые функции: `fetchTelegramBusinessChats`, `fetchTelegramBusinessChatDetail`, `fetchTelegramBusinessChatMembers`, `fetchTelegramBusinessMessages` (с маппингом полей), `searchTelegramBusinessChats`, `searchTelegramBusinessMessages`, `searchTelegramBusinessMedia`, `fetchTelegramBusinessPinnedMessages`, `sendTelegramBusinessMessage`, `replyToTelegramBusinessMessage`, `forwardTelegramBusinessMessage`, `editTelegramBusinessMessage`, `deleteTelegramBusinessMessage`, `restoreTelegramBusinessMessageVisibility`, `pinTelegramBusinessMessage`, `markTelegramBusinessMessageRead`, `fetchTelegramBusinessMessageVersions`, `fetchTelegramBusinessMessageTombstones`, `fetchTelegramBusinessReplyChain`, `fetchTelegramBusinessForwardChain`, `fetchTelegramBusinessReactions`, `addTelegramBusinessReaction`, `removeTelegramBusinessReaction`. Указаны методы и шаблоны URL. |
| `frontend/src/domains/communications/api/telegramBusinessApi.test.ts` | Подтверждены: эндпоинты и параметры для `searchTelegramBusinessTopics` (не видна в обрезанном API-файле), `fetchTelegramBusinessMessages` (маппинг канонических сообщений в DTO Telegram), `pinTelegramBusinessMessage` (форма ответа), `sendTelegramBusinessMessage`, `replyToTelegramBusinessMessage`, `forwardTelegramBusinessMessage` (форма ответа команды). |
| `frontend/src/domains/communications/api/whatsappBusinessApi.ts` (обрезан) | Перечислены все видимые функции: `fetchWhatsappWebBusinessConversations` (с фильтрацией), `fetchWhatsappWebBusinessConversationDetail`, `fetchWhatsappWebBusinessConversationMembers` (возвращает `TelegramChatMemberListResponse`), `fetchWhatsappWebBusinessMessages` (маппинг в `WhatsappWebMessage`), `searchWhatsappWebBusinessMessages`, `searchWhatsappWebBusinessMedia`, `fetchWhatsappWebBusinessPinnedMessages`, `sendWhatsappBusinessMessage`, `replyToWhatsappBusinessMessage`, `forwardWhatsappBusinessMessage`, `editWhatsappBusinessMessage`, `deleteWhatsappBusinessMessage`, `pinWhatsappBusinessMessage`, `pinWhatsappBusinessConversation`, `unpinWhatsappBusinessConversation`, `archiveWhatsappBusinessConversation`, `unarchiveWhatsappBusinessConversation`, `muteWhatsappBusinessConversation`, `unmuteWhatsappBusinessConversation`, `markWhatsappBusinessConversationRead`, `markWhatsappBusinessConversationUnread`, реакции (fetch/add/remove). Указаны методы и шаблоны URL. |
| `frontend/src/domains/communications/api/whatsappBusinessApi.test.ts` (обрезан) | Подтверждены: фильтрация чатов (`isWhatsappConversation`), эндпоинты и параметры для `fetchWhatsappWebBusinessConversations`, `fetchWhatsappWebBusinessConversationDetail`, `fetchWhatsappWebBusinessConversationMembers`, `fetchWhatsappWebBusinessMessages` (маппинг), `searchWhatsappWebBusinessMessages` (с `channel_kind`), `searchWhatsappWebBusinessMedia`, `fetchWhatsappWebBusinessPinnedMessages`, `sendWhatsappBusinessMessage`, `replyToWhatsappBusinessMessage`, `forwardWhatsappBusinessMessage`, `editWhatsappBusinessMessage`, `deleteWhatsappBusinessMessage`, `pinWhatsappBusinessMessage`, `pinWhatsappBusinessConversation`, а также archive/unarchive/mute/unmute/read/unread. |
| `frontend/src/domains/communications/api/threadApi.ts` | Функции `fetchThreads`, `fetchThreadMessages`, `translateThread` с делегированием на connect-функции. |
| `frontend/src/domains/communications/components/AttachmentSearchPanel.boundary.test.ts` | Компонент использует Vee/Zod, TanStack Query, TanStack Table, TanStack Virtual; предзагрузка по mouseenter/focus; пропс `accountId`; отсутствие прямых вызовов API. |
| `frontend/src/domains/communications/components/BilingualReplyPanel.boundary.test.ts` | Компонент использует Vee/Zod и TanStack Mutation; поля и эмит `send-bilingual-reply`; отсутствие прямых вызовов API. |
| `frontend/src/domains/communications/components/BulkActionsBar.boundary.test.ts` | Компонент поддерживает действия `pin/unpin/important/not_important/add_label/remove_label/snooze`; типы `BulkActionCommand`/`BulkMessageActionRequest`; отложенное время `nextBusinessMorningIso()`. |
| `frontend/src/domains/communications/components/CommunicationFolderStrip.boundary.test.ts` | Компонент использует Vee/Zod, TanStack Virtual (горизонтально), мутации CRUD папок, реордеринг, drag-and-drop, иерархию, диалоги, эмит `select`, отсутствие прямых вызовов API. |
| `frontend/src/domains/communications/components/CommunicationList.boundary.test.ts` | Компонент поддерживает клавиатурное множественное выделение (Space, Ctrl+A, Escape, ArrowDown/Up с Shift), эмиты `toggleSelection`, `selectVisible`, `clearSelection`, атрибуты доступности; отсутствие прямых вызовов API. |
| `frontend/src/domains/communications/components/CommunicationListItem.boundary.test.ts` | Компонент использует `createCommunicationMessageDragPayload` с `selectedMessageIds`, атрибуты `role="option"` и `aria-selected`. |
| `frontend/src/domains/communications/components/CommunicationViewer.boundary.test.ts` | Компонент содержит `MessageBodyTab`, AI-состояния (`REVIEW_REQUIRED`, `FAILED`), действия с сообщением и соответствующие эмиты; отсутствие прямых вызовов API. |
| `frontend/src/domains/communications/components/CommunicationsActionBar.boundary.test.ts` | Компонент включает экспорт сообщений (download-ссылка), стрипы ресурсов/синхронизации/сертификатов, пагинацию для drafts/subscriptions/top-senders/blockers; отсутствие прямых вызовов API. |
| `frontend/src/domains/communications/components/CommunicationsCallsPanel.boundary.test.ts` | Компонент использует `useProviderCallsQuery`/`useProviderCallTranscriptQuery`, режимы calls/meetings, при meetings провайдер `zoom`, показывает participants/recording refs; отсутствие прямых вызовов API. |
| `frontend/src/domains/communications/components/CommunicationsConversationList.boundary.test.ts` | Компонент рендерит `threads`, предзагрузка `useThreadMessagesPrefetch` при mouseenter/focus, эмиты `selectThread`/`loadMoreThreads`; отсутствие прямых вызовов API. |
| `frontend/src/domains/communications/components/CommunicationsDetailPane.boundary.test.ts` | Компонент перенаправляет события `CommunicationViewer` и использует `ThreadConversationView` для тредов. |
| `frontend/src/domains/communications/components/CommunicationsListPane.boundary.test.ts` | Компонент имеет режим папок (`isFolderMode`), рендерит `CommunicationList`, пробрасывает эмиты множественного выделения. |
| `frontend/src/domains/communications/components/ComposeDrawer.boundary.test.ts` | Компонент использует мутации TanStack, `Sheet` из shared/ui, `RichComposeEditor`, пикеры шаблонов и подписей, staged attachments (без подключения к провайдеру), импорт CSS; отсутствие прямых вызовов API. |
| `frontend/src/domains/communications/components/ComposeSignaturePicker.boundary.test.ts` | Компонент использует `usePersonasQuery`, эмит `apply` с `persona.signature`; отсутствие прямых вызовов API. |
| `frontend/src/domains/communications/components/ComposeTemplatePicker.boundary.test.ts` | Компонент использует TanStack Query для операций с шаблонами, включает mail merge preview, диагностику, категории, маппинг получателей, диалог сохранения; отсутствие прямых вызовов API. |
| `frontend/src/domains/communications/components/DraftStrip.boundary.test.ts` | Компонент использует TanStack Virtual, эмит `loadMore`; отсутствие прямых вызовов API. |
| `frontend/src/domains/communications/components/MailCertificateStrip.boundary.test.ts` | Компонент использует запросы и мутацию для сертификатов, схему VeeValidation, UI для истекающих сертификатов; отсутствие прямых вызовов API. |

## Исходные файлы

- [`frontend/src/domains/communications/api/readReceipts.ts`](../../../../frontend/src/domains/communications/api/readReceipts.ts)
- [`frontend/src/domains/communications/api/savedSearchApi.ts`](../../../../frontend/src/domains/communications/api/savedSearchApi.ts)
- [`frontend/src/domains/communications/api/sendApi.ts`](../../../../frontend/src/domains/communications/api/sendApi.ts)
- [`frontend/src/domains/communications/api/telegramBusinessApi.test.ts`](../../../../frontend/src/domains/communications/api/telegramBusinessApi.test.ts)
- [`frontend/src/domains/communications/api/telegramBusinessApi.ts`](../../../../frontend/src/domains/communications/api/telegramBusinessApi.ts)
- [`frontend/src/domains/communications/api/threadApi.ts`](../../../../frontend/src/domains/communications/api/threadApi.ts)
- [`frontend/src/domains/communications/api/whatsappBusinessApi.test.ts`](../../../../frontend/src/domains/communications/api/whatsappBusinessApi.test.ts)
- [`frontend/src/domains/communications/api/whatsappBusinessApi.ts`](../../../../frontend/src/domains/communications/api/whatsappBusinessApi.ts)
- [`frontend/src/domains/communications/components/AttachmentSearchPanel.boundary.test.ts`](../../../../frontend/src/domains/communications/components/AttachmentSearchPanel.boundary.test.ts)
- [`frontend/src/domains/communications/components/BilingualReplyPanel.boundary.test.ts`](../../../../frontend/src/domains/communications/components/BilingualReplyPanel.boundary.test.ts)
- [`frontend/src/domains/communications/components/BulkActionsBar.boundary.test.ts`](../../../../frontend/src/domains/communications/components/BulkActionsBar.boundary.test.ts)
- [`frontend/src/domains/communications/components/CommunicationFolderStrip.boundary.test.ts`](../../../../frontend/src/domains/communications/components/CommunicationFolderStrip.boundary.test.ts)
- [`frontend/src/domains/communications/components/CommunicationList.boundary.test.ts`](../../../../frontend/src/domains/communications/components/CommunicationList.boundary.test.ts)
- [`frontend/src/domains/communications/components/CommunicationListItem.boundary.test.ts`](../../../../frontend/src/domains/communications/components/CommunicationListItem.boundary.test.ts)
- [`frontend/src/domains/communications/components/CommunicationViewer.boundary.test.ts`](../../../../frontend/src/domains/communications/components/CommunicationViewer.boundary.test.ts)
- [`frontend/src/domains/communications/components/CommunicationsActionBar.boundary.test.ts`](../../../../frontend/src/domains/communications/components/CommunicationsActionBar.boundary.test.ts)
- [`frontend/src/domains/communications/components/CommunicationsCallsPanel.boundary.test.ts`](../../../../frontend/src/domains/communications/components/CommunicationsCallsPanel.boundary.test.ts)
- [`frontend/src/domains/communications/components/CommunicationsConversationList.boundary.test.ts`](../../../../frontend/src/domains/communications/components/CommunicationsConversationList.boundary.test.ts)
- [`frontend/src/domains/communications/components/CommunicationsDetailPane.boundary.test.ts`](../../../../frontend/src/domains/communications/components/CommunicationsDetailPane.boundary.test.ts)
- [`frontend/src/domains/communications/components/CommunicationsListPane.boundary.test.ts`](../../../../frontend/src/domains/communications/components/CommunicationsListPane.boundary.test.ts)
- [`frontend/src/domains/communications/components/ComposeDrawer.boundary.test.ts`](../../../../frontend/src/domains/communications/components/ComposeDrawer.boundary.test.ts)
- [`frontend/src/domains/communications/components/ComposeSignaturePicker.boundary.test.ts`](../../../../frontend/src/domains/communications/components/ComposeSignaturePicker.boundary.test.ts)
- [`frontend/src/domains/communications/components/ComposeTemplatePicker.boundary.test.ts`](../../../../frontend/src/domains/communications/components/ComposeTemplatePicker.boundary.test.ts)
- [`frontend/src/domains/communications/components/DraftStrip.boundary.test.ts`](../../../../frontend/src/domains/communications/components/DraftStrip.boundary.test.ts)
- [`frontend/src/domains/communications/components/MailCertificateStrip.boundary.test.ts`](../../../../frontend/src/domains/communications/components/MailCertificateStrip.boundary.test.ts)

## Кандидаты на drift

Из предоставленного контекста виден потенциальный дрейф:

- В `frontend/src/domains/communications/api/whatsappBusinessApi.ts` функция `fetchWhatsappWebBusinessConversationMembers` возвращает тип `TelegramChatMemberListResponse` из `../../../shared/communications/types/telegramMembers`. Использование Telegram-типа в API для WhatsApp может быть осознанным (общий тип участников чата), либо указывать на незавершённый переход к независимым WhatsApp-типам. Других расхождений между кодом, тестами и документацией в данном чанке не обнаружено.
