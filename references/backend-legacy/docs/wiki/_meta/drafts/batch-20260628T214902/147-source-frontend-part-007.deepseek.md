### Summary / Резюме

Добавить страницу `components/frontend.md` в русскую Obsidian‑wiki. В ней описан слой запросов к данным коммуникаций (почта, Telegram, WhatsApp) на основе TanStack Vue Query: бесконечные списки, мутации, оптимистичные обновления кеша и функции для реактивного применения серверных событий (realtime‑patch). Страница строится исключительно по приложенным исходным файлам.

### Proposed pages / Предлагаемые страницы

`components/frontend.md`

```markdown
# Коммуникации: frontend‑слой запросов

## Обзор

Слой запросов построен на библиотеке **TanStack Vue Query** и предоставляет
единообразный доступ к данным коммуникаций из Vue‑компонентов. Все хуки
работают с реактивными ключами через `MaybeRefOrGetter` (типы `QueryParam<T>`,
`NullableQueryParam<T>` из `queryTypes.ts`).

Основные группы:

- **почтовые операции** – черновики, исходящие, отправка, отмена,
  билингвальные ответы, переадресация;
- **рабочие ресурсы** – папки, сохранённые поиски, шаблоны, сертификаты,
  вложения, подписки, топ‑отправители, блокировщики архитектуры;
- **оптимистичные обновления кеша** – чистые функции для немедленного
  отражения изменений в бесконечных списках (`InfiniteData`);
- **realtime‑патчи** – функции, обрабатывающие события сервера и
  обновляющие кешированные данные без полного перезапроса.

---

## Почтовые операции

Файл `mailOperationQueries.ts`.

### Бесконечные списки

- `useDraftsQuery(accountId?)` – черновики. Параметры: `fetchDrafts(accountId, undefined, 50, pageParam)`.
- `useOutboxQuery(accountId?, status?)` – исходящие. Параметры: `fetchOutboxItems(accountId, status, 100, pageParam)`.

Оба хука возвращают плоский массив элементов через `select: (data) => data.pages.flatMap(…)`.

### Мутации

| Хук | Назначение |
|-----|------------|
| `useSendMailMutation()` | Отправка письма. При наличии `draft_id` оптимистично удаляет черновик из кеша, при ошибке восстанавливает. При успехе инвалидирует списки сообщений, черновиков и исходящих. |
| `useSaveDraftMutation()` | Сохранение черновика. Оптимистично добавляет/обновляет черновик в кеше. При успехе замещает оптимистичную запись серверной. |
| `useDeleteDraftMutation()` | Удаление черновика. Оптимистично убирает из кеша, при ошибке восстанавливает. |
| `useUndoOutboxMutation()` | Отмена отправки. Оптимистично помечает элемент как `canceled` или удаляет его в зависимости от статуса запроса. При успехе обновляет запись серверными данными. |
| `usePrepareBilingualReplyFlowMutation()` | Подготовка билингвального ответа. |
| `useRedirectMessageMutation()` | Переадресация сообщения. |

Вспомогательные функции `restoreDraftLists`, `restoreOutboxLists` откатывают
оптимистичные изменения при ошибке.

---

## Рабочие ресурсы и папки

Файл `mailWorkspaceQueries.ts` (доступен частично, описаны только видимые хуки).

### Бесконечные списки

- `useSubscriptionsQuery(accountId?)` – подписки (`fetchSubscriptions`, размер страницы 25).
- `useTopSendersQuery(accountId?)` – топ‑отправители (`fetchTopSenders`, 25).
- `useSavedSearchesQuery(isSmartFolder?, accountId?)` – сохранённые поиски (`fetchSavedSearches`, 100).
- `useCommunicationFoldersQuery(accountId?)` – папки (`fetchCommunicationFolders`, 500).
- `useFolderMessagesQuery(folderId, enabled?)` – сообщения папки (`fetchFolderMessages`, 250).
- `useAttachmentSearchQuery(request, enabled?)` – поиск вложений.

### Обычные запросы и мутации

- `useRichTemplatesQuery()` – список rich‑шаблонов.
- `useCommunicationBlockersQuery()` – блокировщики архитектуры.
- `useMailCertificatesQuery()`, `useExpiringMailCertificatesQuery(days?)` – сертификаты.
- `useCreateMailCertificateMutation()` – создание сертификата.
- `useAttachmentPreviewQuery(attachmentId, enabled?)` – превью вложения.
- `useTranslateAttachmentMutation()` – перевод вложения.
- `useCreateRichTemplateMutation()`, `useDeleteRichTemplateMutation()` – управление rich‑шаблонами.
- `useRenderRichTemplateMutation()`, `usePreviewRichTemplateMailMergeMutation()` – рендеринг и предпросмотр слияния.
- `useCreateSavedSearchMutation()`, `useUpdateSavedSearchMutation()` – управление сохранёнными поисками.

---

## Оптимистичные обновления

Чистые функции, работающие с `InfiniteData<…>` (структура с `pages` и `pageParams`).

### Почтовые операции

Файл `optimisticMailUpdates.ts`.

- `applyBulkMessageActionToMailList(data, request, queryKey?)` – применяет массовое действие (`mark_read`, `mark_unread`, `archive`, `trash`, `restore`, `pin`, `unpin`, `important`, `not_important`, `add_label`, `remove_label`, `snooze`) к списку сообщений, удаляя элементы, которые перестали соответствовать фильтрам запроса (`workflowState`, `localState`).
- `applyBulkMessageActionToMailDetail(data, request)` – обновляет деталь сообщения.
- `upsertDraftInDraftList(drafts, draft)` – вставляет или замещает черновик.
- `removeDraftFromDraftList(drafts, draftId)` – удаляет черновик.
- `upsertOutboxItem(items, item)` – вставляет или замещает элемент исходящих.
- `markOutboxItemCanceled(items, outboxId)` – помечает элемент как `canceled`, сбрасывает `undo_deadline_at`.

Фильтры списка из ключа запроса разбираются через `parseMailListFilters` (позиции 2 и 5 в ключе).

### Папки

Файл `optimisticFolderUpdates.ts`.

- `upsertFolderInFolderList(data, queryKey, folder)` – вставляет или обновляет папку; если папка перестала соответствовать запросу, удаляет её.
- `removeFolderFromFolderList(data, folderId)` – удаляет папку по идентификатору.
- `optimisticFolderFromUpdate(existing, update, updatedAt)` – строит оптимистичный объект папки на основе частичного обновления.

Файл `optimisticFolderMessageUpdates.ts`.

- `upsertFolderMessageInFolderList(data, queryKey, folderMessage)` – вставляет или обновляет сообщение папки.
- `removeFolderMessageFromFolderList(data, messageId)` – удаляет сообщение.
- `findCachedFolderMessage(lists, messageId)` – ищет сообщение в снапшотах кешей папок.
- `optimisticFolderMessageForTarget(source, folderId, addedAt)` – создаёт оптимистичную запись для целевой папки.

Сортировка в папках: по `added_at` (по убыванию), затем по `message_id`.

---

## Realtime‑патчи

Функции, обрабатывающие `eventData` (строка JSON от сервера) и обновляющие
кеши запросов через `getQueriesData` / `setQueryData`.

### Почта

Файл `realtimeMailPatches.ts` (доступен частично).

Точка входа: `applyMailRealtimePatch(eventData, queryClient)`.
Последовательно обрабатывает следующие виды событий:

1. **AI‑состояние** – `mail.ai_state.changed`. Обновляет кеш по ключу `['communications-ai-state', messageId]`.
2. **Синхронизация** – `mail.sync.started`, `.progress`, `.completed`, `.failed`, `.skipped`. Обновляет список статусов синхронизации (`['communications', 'mail', 'sync-statuses']`).
3. **Сообщения папок** – `mail.folder_message.copied`, `.moved`. Добавляет/удаляет сообщения в кешах папок.
4. **Сохранённые поиски** – `mail.saved_search.created`, `.updated`, `.deleted`.
5. **Outbox / Draft** – (видна только диспетчеризация, детали обрезаны).
6. **Массовые действия** – если ни одно специализированное событие не сработало, пытается интерпретировать `eventData` как `BulkMessageActionRequest` и применить ко всем спискам `['communications-list']` и деталям `['communications-message', messageId]`.

Функции‑обработчики используют универсальные утилиты из `realtimePatchShared.ts`
(реэкспорт из `shared/communications/queries/realtimePatchShared`).

### Telegram

Файл `realtimeTelegramPatches.ts` (доступен частично).

Точка входа: `applyTelegramRealtimePatch(eventData, queryClient)`.
Покрывает кеши:

- списки сообщений (`['communications', 'telegram', 'messages']`);
- закреплённые сообщения;
- списки чатов (`['communications', 'telegram', 'chats']`);
- детали чата;
- папки‑фильтры;
- поиск сообщений и медиа;
- реакции;
- топики (через `patchTelegramTopicList`);
- статус runtime.

Поддерживаются события: `telegram.message.created`, `.updated`, `.edited`,
`.deleted`, `.visibility_restored`, `telegram.reaction.changed`,
`telegram.media.download.*` (прогресс загрузки медиа) и события синхронизации.

Для сообщений и чатов при вставке используется сортировка по времени
(утилиты `insertMessageByRecency`, `insertChatByRecency`, `messageRecencyKey`,
`chatRecencyKey` из `realtimeTelegramPatchShared.ts`).

Медиа‑патчи (`realtimeTelegramMediaPatches.ts`) обновляют состояние загрузки
вложений как в объектах сообщений, так и в результатах поиска медиа.

Участники чатов (`realtimeTelegramParticipantPatches.ts`) – только событие
`telegram.participant.updated`. Обновляет список участников, фильтруя
неактивных (статус `left`, `banned`, `absent_exhaustive` или роль `left`,
`banned`).

### WhatsApp

Файл `realtimeWhatsAppPatches.ts` (доступен частично).

Точка входа: `applyWhatsAppRealtimePatch(eventData, queryClient)`.
Обрабатывает события, начинающиеся с `whatsapp.`. Патчит:

- список диалогов (`['communications', 'whatsapp', 'conversations']`);
- деталь диалога;
- список сообщений.

События: `whatsapp.dialog.updated`, `whatsapp.message.created`,
`.updated`, `.deleted`, `whatsapp.reaction.changed`,
`whatsapp.receipt.changed`.

Обновление диалогов учитывает флаги `is_pinned`, `is_archived`, `is_muted`,
`is_unread`, `unread_count`, `participant_count`, `chat_kind`, `chat_title`.

При вставке нового сообщения применяется простой препендинг с учётом лимита
из ключа запроса.

---

## Подтверждающие тесты

- `messageLocalIntelligence.boundary.test.ts` – проверяет, что `mailActionQueries.ts` экспортирует `useExplainMessageMutation` и `useDetectMessageLanguageMutation`, а также использует `fetchMessageExplain` и `detectMessageLanguage`.
- `outboxInfiniteQuery.boundary.test.ts` – подтверждает бесконечную загрузку исходящих через `useInfiniteQuery` с курсором и наличие отдельного хука `useOutboxStatusStrip`, предоставляющего `hasMoreOutboxItems`, `loadMoreOutboxItems`, `prefetchMoreOutboxItems` для компонента `OutboxStatusStrip.vue`.
- `resourceOverviewInfiniteQuery.boundary.test.ts` – подтверждает бесконечные списки подписок и топ‑отправителей с курсором.
- `savedSearchInfiniteQuery.boundary.test.ts` – подтверждает бесконечную загрузку сохранённых поисков.
- Тесты оптимистичных обновлений (`optimisticMailUpdates.test.ts`, `optimisticFolderUpdates.test.ts`, `optimisticFolderMessageUpdates.test.ts`) покрывают логику вставки, удаления и пересортировки для почты, папок и сообщений папок.
```

### Source coverage / Покрытие источников

| Файл | Факты, отражённые в странице |
|------|-------------------------------|
| `mailOperationQueries.ts` | Состав и сигнатуры хуков для черновиков, исходящих, отправки, сохранения, удаления, отмены отправки, билингвального ответа, переадресации; оптимистичное удаление/восстановление черновиков; ключи запросов (`communications-drafts`, `communications-outbox`, `communications-list`); вспомогательные функции восстановления кеша. |
| `mailWorkspaceQueries.ts` | Видимые хуки: `useRichTemplatesQuery`, `useSubscriptionsQuery`, `useTopSendersQuery`, `useCommunicationBlockersQuery`, `useMailCertificatesQuery`, `useExpiringMailCertificatesQuery`, `useCreateMailCertificateMutation`, `useSavedSearchesQuery`, `useCommunicationFoldersQuery`, `useFolderMessagesQuery`, `useAttachmentSearchQuery`, `useAttachmentArchiveInspectionQuery`, `useAttachmentPreviewQuery`, `useTranslateAttachmentMutation`, `useCreateRichTemplateMutation`, `useDeleteRichTemplateMutation`, `useRenderRichTemplateMutation`, `usePreviewRichTemplateMailMergeMutation`, `useCreateSavedSearchMutation`, `useUpdateSavedSearchMutation`; параметры запросов (размеры страниц, ключи). |
| `optimisticMailUpdates.ts` | Экспортируемые функции: `applyBulkMessageActionToMailList`, `applyBulkMessageActionToMailDetail`, `upsertDraftInDraftList`, `removeDraftFromDraftList`, `upsertOutboxItem`, `markOutboxItemCanceled`; типы массовых действий; влияние фильтров `workflowState`/`localState` на видимость элементов; изменение `workflow_state` и `local_state`; работа с метаданными (пин, важность, метки, snooze). |
| `optimisticFolderUpdates.ts` | `upsertFolderInFolderList`, `removeFolderFromFolderList`, `optimisticFolderFromUpdate`; проверка соответствия папки запросу по `account_id`; сортировка по `sort_order` и `name`. |
| `optimisticFolderMessageUpdates.ts` | `upsertFolderMessageInFolderList`, `removeFolderMessageFromFolderList`, `findCachedFolderMessage`, `optimisticFolderMessageForTarget`; сортировка по `added_at` и `message_id`. |
| `realtimeMailPatches.ts` (частично) | `applyMailRealtimePatch` и порядок обработки событий: AI‑состояние, синхронизация, папки, сохранённые поиски, outbox/draft, массовые действия; работа с ключами `['communications-ai-state', …]`, `['communications', 'mail', 'sync-statuses']`, `['communications-folder-messages']`, `['communications-saved-searches']`; обновление `MailSyncStatus`. |
| `realtimePatchShared.ts` | Реэкспорт из `shared/communications/queries/realtimePatchShared`. |
| `realtimeTelegramPatches.ts` (частично) | `applyTelegramRealtimePatch` и перечень кешей: сообщения, закреплённые сообщения, чаты, деталь чата, папки‑фильтры, поиск сообщений/медиа, реакции, топики, статус runtime; события `telegram.message.*`, `telegram.reaction.changed`, `telegram.media.download.*`, события синхронизации. |
| `realtimeTelegramMediaPatches.ts` | `patchTelegramMediaSearch`, `patchTelegramMessageMediaDownloadState`; обновление состояния загрузки и вставка загруженного медиа в поиск. |
| `realtimeTelegramParticipantPatches.ts` | `applyTelegramParticipantRealtimePatch` для события `telegram.participant.updated`; фильтрация по статусу/роли, вставка новых участников. |
| `realtimeTelegramPatchShared.ts` | Вспомогательные утилиты: `telegramChatSnapshot`, `telegramMessageSnapshot`, `insertMessageByRecency`, `insertChatByRecency`, `matchesMessageScope`, `matchesChatScope`, `patchPinMetadata`; структура `TelegramEventPayload`. |
| `realtimeTelegramTopicPatches.ts` | `patchTelegramTopicList`; обработка обновления топика, поиск и лимит в ключе запроса. |
| `realtimeWhatsAppPatches.ts` (частично) | `applyWhatsAppRealtimePatch`; список кешей (conversations, conversation-detail, messages); события `whatsapp.dialog.updated`, `whatsapp.message.*`, `whatsapp.reaction.changed`, `whatsapp.receipt.changed`; обновление флагов диалогов; `whatsappMessageSnapshot`, `whatsappConversationSnapshot`. |
| `queryTypes.ts` | Типы `QueryParam<T>`, `NullableQueryParam<T>`. |
| `outboxStatusStrip.ts` | Хук `useOutboxStatusStrip` с полями `outboxItems`, `outboxErrorMessage`, `isOutboxLoading`, `isLoadingMoreOutbox`, `hasMoreOutboxItems`, `isUndoingOutbox` и методами `undoOutbox`, `loadMoreOutboxItems`, `prefetchMoreOutboxItems`. |
| `messageLocalIntelligence.boundary.test.ts` | Проверка экспорта `useExplainMessageMutation`/`useDetectMessageLanguageMutation` и вызова `fetchMessageExplain`/`detectMessageLanguage` в `mailActionQueries.ts` (сам файл в чанк не включён, но тест подтверждает наличие). |
| `outboxInfiniteQuery.boundary.test.ts` | Подтверждение структуры бесконечного запроса `useOutboxQuery` и использования хука `useOutboxStatusStrip` с ленивой загрузкой/предзагрузкой в `OutboxStatusStrip.vue`. |
| `resourceOverviewInfiniteQuery.boundary.test.ts` | Бесконечные списки подписок и топ‑отправителей. |
| `savedSearchInfiniteQuery.boundary.test.ts` | Бесконечная загрузка сохранённых поисков. |
| `optimisticMailUpdates.test.ts` | Логика `applyBulkMessageActionToMailList` (mark_read, archive, trash), `upsertDraftInDraftList`, `removeDraftFromDraftList`, `upsertOutboxItem`, `markOutboxItemCanceled`. |
| `optimisticFolderUpdates.test.ts` | Поведение `upsertFolderInFolderList`, `removeFolderFromFolderList`, `optimisticFolderFromUpdate`, учёт `account_id`. |
| `optimisticFolderMessageUpdates.test.ts` | Поведение `upsertFolderMessageInFolderList`, `removeFolderMessageFromFolderList`, `findCachedFolderMessage`, `optimisticFolderMessageForTarget`. |

### Drift candidates / Кандидаты на drift

На основе предоставленного контекста расхождений между кодом, документацией и ADR не видно. Текущее содержимое страницы `components/frontend.md` не входит в чанк, поэтому сравнить невозможно.
