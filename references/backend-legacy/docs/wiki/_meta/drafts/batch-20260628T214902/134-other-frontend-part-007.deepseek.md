### Summary / Резюме

Страница `components/frontend.md` дополнена документацией 12-ти Vue-компонентов из домена `communications` (и одного провайдерского представления для Telegram). Для каждого компонента перечислены входные параметры (props), генерируемые события (emits), ключевые вычисляемые свойства и особенности поведения, подтверждённые встроенным исходным кодом. Часть файлов была обрезана, что отмечено в примечаниях. Обновление позволяет разработчикам быстро ориентироваться в API компонентов и понимать их зоны ответственности.

### Proposed pages / Предлагаемые страницы

---

### `components/frontend.md`

```markdown
# Компоненты фронтенда

Данная страница описывает Vue-компоненты, расположенные в `frontend/src/domains/communications/components/` и `frontend/src/domains/communications/providers/telegram/views/`.  
Анализ выполнен по исходным файлам, предоставленным в контексте данного чанка. Для обрезанных файлов указано, что полная информация недоступна.

## MessageBodyTab

- **Файл:** `frontend/src/domains/communications/components/MessageBodyTab.vue` (обрезан)
- **Назначение:** отображает тело сообщения (HTML или текст), блокирует внешние изображения до явной загрузки через прокси, выводит карточку Message Intelligence, встраивает дочерние панели `MessageAiReplyPanel`, `MessageLocalIntelligencePanel`, `MessageTrustReviewPanel`, секции AI Summary Review, Knowledge Review, Extraction Review и панель действий (Reply, Task, Note, Translate, AI Reply, Security, Smart CC, Bilingual, Analyze).
- **Props:**
  - `detail: CommunicationMessageDetailResponse | null` – детальная информация о сообщении.
  - `insight: CommunicationMessageInsight | null` – результаты анализа сообщения.
- **Emits:**
  - `analyze`, `reply`, `createTask`, `createNote`, `translate`
  - `generateAiReply(payload: { tone: string; language: string })`
  - `applyAiReply(payload: AiReplyResponse)`
  - `reviewSecurity`, `reviewRecipients`
  - `sendBilingualReply(payload: BilingualReplyFlowResponse)`
- **Ключевые вычисляемые свойства:**
  - `message`, `attachments`, `messageId` – из `detail`.
  - `summaryContract` – результат вызова `aiSummaryContractFromMetadata`.
  - `summarySections` – секции Key points / Action items / Risks / Deadlines.
  - `extractionSections` – на основе `communicationExtractionSectionsFromInsight`.
  - `knowledgeSections` – на основе `communicationKnowledgeSectionsFromSummaryContract`.
  - `renderedBody` – вызов `renderMessageBody` для HTML/текста.
  - `remoteImageUrls` – ссылки на внешние изображения из HTML.
  - `displayHtml` – HTML с переписанными URL изображений, если загрузка через прокси активирована.
  - `originalSrcdoc` – полный HTML-документ для отображения в `<iframe>` с песочницей (`sandbox="allow-popups allow-popups-to-escape-sandbox"`).
- **Особенности:**
  - Переключение между «обычным» HTML-рендерингом (`v-html`) и просмотром исходного HTML в `<iframe>`.
  - Кнопка загрузки внешних изображений через прокси-эндпоинт `GET /api/v1/communications/messages/{message_id}/remote-image?url=...`.
  - При отсутствии HTML-тела используется `<pre>` с текстовой версией.
- **Примечание:** исходный файл обрезан после 12000 символов; стили и завершающая часть шаблона могут содержать дополнительные детали.

## MessageHeadersTab

- **Файл:** `frontend/src/domains/communications/components/MessageHeadersTab.vue` (полный)
- **Назначение:** таблица с основными заголовками сообщения.
- **Props:**
  - `detail: CommunicationMessageDetailResponse | null`
- **Вычисляемые свойства:**
  - `message` – `detail?.message` или `null`.
- **Шаблон:**
  - Если `message` отсутствует – надпись «No message selected».
  - Иначе – `<table>` со строками:
    - From (`message.sender`)
    - To (`message.recipients?.join(', ')`)
    - Subject (`message.subject`)
    - Date (`message.projected_at || message.occurred_at`)
    - Channel (`message.channel_kind`)
    - Message ID (`message.message_id`, моноширинный)
    - Account (`message.account_id`)
    - State (`message.workflow_state / message.local_state`)
    - Importance (`message.importance_score ?? 'N/A'`)
- **Стили:** scoped, таблица с рамками, label жирный, monospace для ID.

## MessageLocalIntelligencePanel

- **Файл:** `frontend/src/domains/communications/components/MessageLocalIntelligencePanel.vue` (полный)
- **Назначение:** запуск и отображение локального анализа важности и определения языка сообщения.
- **Props:**
  - `messageId: string | null`
  - `insight: CommunicationMessageInsight | null`
- **Используемые мутации:**
  - `useExplainMessageMutation` (запрос объяснения важности).
  - `useDetectMessageLanguageMutation` (определение языка).
- **Состояние:**
  - `explainResult`, `languageResult` – результаты запросов.
  - `errorMessage` – текст ошибки.
- **Вычисляемые свойства:**
  - `currentExplain` – `explainResult ?? props.insight?.explain`
  - `currentLanguage` – `languageResult ?? props.insight?.language`
  - `isRunning` – `explainMutation.isPending || languageMutation.isPending`
- **Методы:**
  - `explainMessage()` – вызывает `explainMutation.mutateAsync(props.messageId)`, при ошибке записывает сообщение.
  - `detectLanguage()` – вызывает `languageMutation.mutateAsync(props.messageId)`, аналогичная обработка ошибок.
- **Шаблон:**
  - Заголовок «Importance & Language», кнопки «Why this matters» и «Detect language».
  - При наличии `currentExplain` – список причин (`reasons`).
  - При наличии `currentLanguage` – название языка и процент уверенности.
  - При отсутствии результатов и не в процессе выполнения – «No local intelligence review has been run for this message.»
  - Ошибка отображается красным текстом.

## MessageRelatedTab

- **Файл:** `frontend/src/domains/communications/components/MessageRelatedTab.vue` (полный)
- **Назначение:** панель дополнительных действий с сообщением (управление состоянием, метки, экспорт, переадресация, snooze).
- **Props:**
  - `detail: CommunicationMessageDetailResponse | null`
- **Emits:**
  - `togglePin`, `toggleImportant`, `mute`, `replyAll`, `forwardMessage`
  - `redirectMessage(recipientsText: string)`
  - `exportMessage(format: MessageExportFormat)` – форматы: `'md'`, `'eml'`, `'json'`
  - `addLabel(label: string)`, `removeLabel(label: string)`
  - `markMessageRead`, `markMessageUnread`, `deleteFromProvider`
  - `snoozeMessage(until: string)`
- **Вычисляемые свойства:**
  - `labels` – метки из `communicationMessageLabelsFromMetadata`.
  - `snoozeUntil` – время snooze из `communicationMessageSnoozeUntilFromMetadata`.
- **Состояние:**
  - `redirectRecipientsText` – текст для переадресации.
  - Константа `quickLabels: ['Follow up', 'Finance', 'Legal']`.
- **Методы:**
  - `snoozePreset(days: number): string` – возвращает ISO-строку с 09:00 через указанное количество дней.
- **Шаблон:**
  - Группа «Read / Delete»: Mark as read, Mark as unread, Delete in provider.
  - Группа «Message Actions»: Pin, Important, Mute, Reply All, Forward, кнопки экспорта (md/eml/json).
  - Группа «Redirect»: поле ввода адресов и кнопка Redirect (disabled при пустом вводе).
  - Группа «Labels»: существующие метки (чипы с кнопкой удаления), быстрые метки (чипы с добавлением, disabled если уже присутствуют).
  - Группа «Snooze»: статус snooze и кнопки «Tomorrow» (1 день) / «Next week» (7 дней).

## MessageTimelineTab

- **Файл:** `frontend/src/domains/communications/components/MessageTimelineTab.vue` (полный)
- **Назначение:** временная шкала событий сообщения.
- **Props:**
  - `detail: CommunicationMessageDetailResponse | null`
- **Локальный интерфейс:** `TimelineEntry { label: string; time: string | null }`.
- **Вычисляемый список `entries`** формируется из полей:
  - Received – `occurred_at ?? projected_at`
  - Projected – `projected_at`
  - State changed – `local_state_changed_at`
  - AI analyzed – `ai_summary_generated_at`
  - Фильтруются записи с `time != null`.
- **Шаблон:**
  - Если записей нет – «No timeline data».
  - Иначе – вертикальный список с точкой (`timeline-dot`) и текстом (label + time).

## MessageTrustReviewPanel

- **Файл:** `frontend/src/domains/communications/components/MessageTrustReviewPanel.vue` (полный)
- **Назначение:** обзор безопасности и рекомендации по получателям.
- **Props:**
  - `messageId: string | null`
  - `insight: CommunicationMessageInsight | null`
- **Emits:**
  - `reviewSecurity`
  - `reviewRecipients`
- **Вычисляемые свойства:**
  - `smartCc` – `insight?.smartCc`
  - `authReview` – `insight?.auth`
  - `signatureReview` – `insight?.signature`
  - `authRisk` – `auth?.risk` (содержит `is_spoofed`, `risk_summary`, `spf_pass`, `dkim_pass`, `dmarc_pass`)
  - `authChecks` – массив объектов `{ label: 'SPF'|'DKIM'|'DMARC', result: string, passed: boolean }`.
- **Шаблон:**
  - Сетка из двух статей.
  - **Security Review:** кнопка «Check auth» (disabled без `messageId`), при наличии `authRisk` – описание риска и чипы SPF/DKIM/DMARC (passed/not passed), при наличии `signatureReview` – тип подписи, предупреждение об истечении сертификата.
  - **Recipient Suggestions:** кнопка «Smart CC», если `smartCc` есть – список предложений (или «No suggestions»).

## OutboxStatusStrip

- **Файл:** `frontend/src/domains/communications/components/OutboxStatusStrip.vue` (полный)
- **Назначение:** отображение статуса доставки исходящих сообщений (outbox).
- **Props:**
  - `items: CommunicationOutboxItem[]`
  - `isLoading: boolean`
  - `isLoadingMore: boolean`
  - `hasMore: boolean`
  - `isUndoing: boolean`
  - `errorMessage: string`
- **Emits:**
  - `undo(outboxId: string)` – отмена отправки.
  - `loadMore` – загрузка следующей страницы истории.
  - `prefetchMore` – предзагрузка (вызывается при наведении/фокусе).
- **Импортируемые утилиты:**
  - `visibleOutboxStatusItems` – фильтрация видимых элементов.
  - `outboxStatusPresentation` – формирует объект с `title`, `detail`, `icon`, `tone`, `canUndo`.
- **Шаблон:**
  - При `isLoading` – скелетон с анимацией.
  - При `errorMessage` – иконка ошибки и текст.
  - Иначе – горизонтальный скроллируемый список элементов. Каждый элемент:
    - Иконка с цветом тона (`tone-success`/`warning`/`danger`/`muted`).
    - Текст: `title` и `subject`, ниже `detail`.
    - Если `canUndo` – кнопка отмены (disabled при `isUndoing`).
  - При `hasMore` – кнопка «…» для дозагрузки, с атрибутами `@mouseenter`/`@focus` для предзагрузки.

## RichComposeEditor

- **Файл:** `frontend/src/domains/communications/components/RichComposeEditor.vue` (обрезан)
- **Назначение:** WYSIWYG-редактор для составления сообщений на базе TipTap.
- **Props:**
  - `modelValue: string` – HTML-содержимое.
  - `placeholder?: string`
- **Emits:**
  - `update:modelValue(value: string)`
  - `attachments-dropped(files: File[])`
  - `blur`
- **Внутреннее состояние:**
  - `isFocused` – фокус редактора.
  - `lastAppliedHtml` – последнее подтверждённое HTML, используется для избежания циклов обновления.
  - `linkHref` – значение для вставки ссылки.
- **Использование редактора:**
  - Создаётся через `useEditor` с расширениями `richComposeExtensions` (импортируются из `./richComposeExtensions`).
  - Обработчики `handlePaste` и `handleDrop`:
    - Для вставки HTML вызывается `sanitizeMailComposePastedHtml`.
    - При наличии файлов в drop-событии эмитируется `attachments-dropped`.
  - При `onUpdate` эмитится `update:modelValue`.
  - Watch на `modelValue` обновляет содержимое редактора, только если редактор не в фокусе и значение отличается.
- **Команды форматирования:**
  - Поддерживаются: `bold`, `italic`, `paragraph`, `heading2`, `heading3`, `alignLeft`, `alignCenter`, `alignRight`, `orderedList`, `bulletList`, `blockquote`, `link`, `unlink`.
  - Команда `link` нормализует href через `normalizeMailComposeLinkHref`.
  - Выравнивание текста обновляется для текущего узла (`heading` или `paragraph`).
- **Шаблон (частично):**
  - Панель инструментов с кнопками для каждого типа форматирования, включая поле ввода ссылки и кнопки Link/Unlink.
  - `<EditorContent>` – поверхность редактора.
- **Примечание:** файл обрезан; завершающая часть стилей и возможные дополнительные обработчики не видны.

## SavedSearchRuleGroupEditor

- **Файл:** `frontend/src/domains/communications/components/SavedSearchRuleGroupEditor.vue` (полный)
- **Назначение:** рекурсивный редактор групп правил для сохранённых поисков.
- **Props:**
  - `group: SavedSearchRuleGroup` – текущая группа.
  - `isRoot?: boolean` – признак корневой группы.
  - `depth?: number` – глубина вложенности.
- **Emits:**
  - `removeGroup` – удаление текущей группы (если не корневая).
- **Методы:**
  - `addRule()` – добавляет новое условие (`createSavedSearchRuleCondition`) в `group.children`.
  - `addGroup()` – добавляет вложенную группу (`createSavedSearchRuleGroup('all', [одно условие])`).
  - `removeNode(nodeId)` – удаляет дочерний узел по `id`.
  - `removeNestedGroup(node)` – удаляет вложенную группу, если узел является группой.
- **Шаблон:**
  - Заголовок с глубиной (`savedSearchRuleGroupDepthLabel`), кратким описанием (`savedSearchRuleGroupSummary`), выбором режима совпадения (all/any).
  - Кнопки: + Rule, + Group, Remove group (только для не-корневых).
  - Дочерние элементы:
    - Если `node.kind === 'rule'` – строка с выбором поля (subject/body/sender/all), оператора (contains/equals), полем ввода значения и кнопкой Remove.
    - Если `node.kind === 'group'` – рекурсивный `SavedSearchRuleGroupEditor`.
  - Если детей нет – «No rules yet».
- **Примечание:** стили вынесены в `SavedSearchStrip.css`.

## SavedSearchStrip

- **Файл:** `frontend/src/domains/communications/components/SavedSearchStrip.vue` (обрезан)
- **Назначение:** панель сохранённых поисков и «умных папок» (smart folders) с виртуальным скроллингом, созданием/редактированием/удалением.
- **Props:**
  - `accountId: string | null`
  - `activeId: string` – id активного поиска.
  - `currentQuery: string` – текущий текстовый запрос.
  - `currentWorkflowState: WorkflowState | ''`
  - `currentLocalState: LocalMessageState`
  - `currentChannelKind: string`
- **Emits:**
  - `select(savedSearch: CommunicationSavedSearch)`
  - `deleted(savedSearch: CommunicationSavedSearch)`
- **Используемые запросы/мутации:**
  - `useSavedSearchesQuery` – загрузка умных папок и сохранённых поисков (флаг `isSmartFolder`).
  - `useCreateSavedSearchMutation`, `useUpdateSavedSearchMutation`, `useDeleteSavedSearchMutation`.
  - `useSavedSearchCommunicationListPrefetch` – предзагрузка списка сообщений для поиска.
- **Состояние:**
  - `dialogOpen` / `deleteDialogOpen` – управление модальными окнами.
  - `editingSearch` / `deletingSearch` – текущий редактируемый/удаляемый поиск.
  - `searchRuleTree` – реактивное дерево правил (`SavedSearchRuleGroup`).
  - `deleteError` – ошибка удаления.
  - Форма через `vee-validate` с полями `SavedSearchFormValues`.
- **Виртуальный скроллинг:**
  - Для каждой группы (smart folders / saved searches) используется `useVirtualizer` с горизонтальной прокруткой, размер элемента 192px, overscan 6.
  - Обработчики `handleSmartFolderVirtualScroll` / `handleSavedSearchVirtualScroll` инициируют `fetchNextPage` при достижении края.
- **Методы:**
  - `openCreateDialog(isSmartFolder)` / `openEditDialog(savedSearch)` / `openDeleteDialog(savedSearch)`.
  - `currentSearchDefaults` – предзаполнение формы текущими фильтрами.
  - `handleSavedSearchPrefetch` – предзагрузка через `prefetchSavedSearchCommunicationList`.
  - `applyPreset(preset)` – применение пресета, сброс дерева правил.
  - `syncRuleTreeFromQuery(query)` – парсинг запроса через `parseSavedSearchQuery`, установка дерева и текстового запроса.
  - `normalizeQueryIntoBuilder` – нормализация запроса в дерево.
  - `submitSavedSearch` – отправка формы (создание/обновление).
  - `confirmDeleteSavedSearch` – удаление с обработкой ошибки.
- **Шаблон (частично):**
  - Скелетон при загрузке.
  - Группа «Smart» с виртуальным списком умных папок, каждая – чип с названием и счётчиком, кнопки Edit/Delete.
  - Группа «Saved» аналогично.
  - Модальное окно (Dialog) для создания/редактирования с формой, включая `SavedSearchRuleGroupEditor`.
  - Модальное окно подтверждения удаления.
- **Примечание:** файл обрезан; детали завершения шаблона и стилей не видны.

## TemplateRecipientMappingPanel

- **Файл:** `frontend/src/domains/communications/components/TemplateRecipientMappingPanel.vue` (полный)
- **Назначение:** связывание переменных шаблона с полями получателей (To/CC/BCC).
- **Props:**
  - `templateVariables: string[]` – список доступных переменных.
  - `mapping: TemplateRecipientVariableMapping` – текущее сопоставление (свойства `toVariable`, `ccVariable`, `bccVariable`).
  - `summary: string` – текстовое описание.
- **Emits:**
  - `update:mapping(mapping: TemplateRecipientVariableMapping)`
  - `fill` – заполнить сопоставленные переменные.
  - `buildPreview` – построить строки получателей из поля To.
- **Методы:**
  - `updateMappingField(key, value)` – обновляет одно поле в mapping.
- **Шаблон:**
  - Заголовок «Recipient mapping» и `summary`.
  - Три `<select>` для выбора переменной To/CC/BCC, опция «Not mapped».
  - Кнопки «Fill mapped variables» и «Build rows from To».

## TemplateSaveForm

- **Файл:** `frontend/src/domains/communications/components/TemplateSaveForm.vue` (полный)
- **Назначение:** форма сохранения текущего содержимого как шаблона.
- **Props:**
  - `name: string` – имя шаблона.
  - `nameError: string` – сообщение об ошибке имени.
  - `validationMessage: string` – сообщение валидации.
  - `canSave: boolean` – доступность сохранения.
  - `isSaving: boolean` – индикатор сохранения.
  - `saveMode: 'new' | 'duplicate'`
- **Emits:**
  - `cancel`, `submit`, `updateName(value: string)`
- **Шаблон:**
  - Описание режима сохранения.
  - Поле ввода имени с атрибутом `aria-invalid`.
  - Отображение ошибок (`template-error`).
  - Кнопки Cancel и «Save current» (с иконкой, `loading` при сохранении, disabled по `canSave`).

## ThreadAttachmentInsightPanel

- **Файл:** `frontend/src/domains/communications/components/ThreadAttachmentInsightPanel.vue` (обрезан)
- **Назначение:** предпросмотр вложений (изображений, аудио, видео, PDF, текста) и инспекция архивов с возможностью перевода текстовых превью.
- **Props:**
  - `attachment: CommunicationAttachment`
- **Состояние:**
  - `panelMode: 'preview' | 'archive' | ''`
  - `attachmentTranslationTarget` – язык перевода (по умолчанию `'en'`).
  - `attachmentTranslationResult`, `attachmentTranslationError`.
- **Используемые запросы:**
  - `useAttachmentArchiveInspectionQuery` – инспекция архива (активен при `panelMode === 'archive'`).
  - `useAttachmentPreviewQuery` – предпросмотр (активен при `panelMode === 'preview'`).
  - `useTranslateAttachmentMutation` – перевод текста превью.
- **Вычисляемые свойства:**
  - `canPreviewAttachment` – через `isPreviewableAttachment`.
  - `canInspectArchive` – через `isInspectableArchiveAttachment`.
  - `isAttachmentTranslationPending` – `translateAttachmentMutation.isPending`.
- **Методы:**
  - `openPreview()` / `openArchiveInspection()` – переключение режима.
  - `translateAttachmentPreview()` – запуск перевода, передаёт `attachment.attachment_id`, целевой язык и исходный текст из `attachmentPreview.text`.
- **Шаблон (частично):**
  - Кнопки переключения режимов с контекстными подписями (Preview image/PDF, Inspect archive).
  - Превью-панель: заголовок с именем файла и статусом сканирования, загрузка/ошибка/контент.
    - Для изображений – `<img>` с `data_url`.
    - Для аудио/видео – `<audio>`/`<video>`.
    - Для PDF – `<iframe>`.
    - Для текста – `<pre>` и блок перевода (выбор языка, кнопка Translate preview, результат или ошибка).
  - Панель инспекции архива: количество записей, общий размер, флаг вложенного архива, список записей с путём и размером.
- **Примечание:** файл обрезан; завершающая часть стилей и возможные дополнительные обработчики не видны.

## ThreadConversationView

- **Файл:** `frontend/src/domains/communications/components/ThreadConversationView.vue` (обрезан)
- **Назначение:** отображение цепочки сообщений (thread) с возможностью разворачивания, inline-ответов и перевода треда.
- **Props:**
  - `thread: CommunicationThreadSummary`
  - `messages: ThreadMessage[]`
  - `isLoading: boolean`
  - `errorMessage: string`
  - `isSendingReply: boolean`
- **Emits:**
  - `openMessage(messageId: string)`
  - `replyToMessage(message, bodyHtml, draftId)`
  - `saveReplyDraft(message, bodyHtml, draftId)`
  - `sendReply(message, bodyHtml, draftId)`
- **Состояние:**
  - `expandedMessageIds` – множество развёрнутых сообщений.
  - `activeReplyMessageId`, `activeReplyDraftId`, `inlineReplyHtml` – для inline-ответа.
  - `showQuotedContent` – показывать цитирование.
  - `threadTranslationTarget`, `threadTranslationResult`, `threadTranslationError`.
- **Вычисляемые свойства:**
  - `canTranslateThread` – есть сообщения и нет активного перевода.
  - `expansionSummary` (через `summarizeThreadExpansion`) – `expandedCount`, `canExpandAll`, `canCollapseAll`.
  - `hasQuotedMessages` – через `hasQuotedThreadMessages`.
  - `translatedMessages` – Map id → item перевода.
- **Методы:**
  - `toggleMessageExpanded`, `expandAllMessages`, `collapseAllMessages`.
  - `startInlineReply`, `cancelInlineReply`, `continueReplyInCompose`, `saveInlineReplyDraft`, `sendInlineReply`.
  - `handleTranslateThread` – вызывает `translateThreadMutation` с `accountId`, `subject`, `targetLanguage`, `limit`.
  - `previewBody`, `quotedBody`, `primaryBody` – формируют отображаемый текст на основе `previewThreadMessageBody` и `splitThreadMessageBody`.
- **Шаблон (частично):**
  - Заголовок с темой, количеством сообщений/участников/развёрнутых, кнопками Expand/Collapse all, Show/Hide quoted, выбор языка и кнопка Translate.
  - Панель результата перевода с количеством сообщений и числом переведённых.
  - Состояния ошибки/загрузки/пустоты.
  - `<ol class="thread-timeline">` – список сообщений:
    - Каждое сообщение: отправитель (label + email), время, кнопки Reply / Expand/Collapse / Open message.
    - Тело: `primaryBody`, при развороте – `quotedBody` в `<blockquote>`.
    - При развороте: список вложений (иконка, имя, размер, тип, статус сканирования), каждое вложение содержит `ThreadAttachmentInsightPanel`.
    - Если `activeReplyMessageId` совпадает – встроенный `ThreadInlineReplyComposer`.

## ThreadInlineReplyComposer

- **Файл:** `frontend/src/domains/communications/components/ThreadInlineReplyComposer.vue` (полный)
- **Назначение:** форма быстрого ответа на сообщение внутри треда с предпросмотром перед отправкой.
- **Props:**
  - `message: ThreadMessage`
  - `bodyHtml: string`
  - `isSendingReply: boolean`
- **Emits:**
  - `update:bodyHtml(bodyHtml: string)`
  - `cancel`
  - `saveDraft`
  - `continueInCompose`
  - `send`
- **Состояние:**
  - `reviewingReply` – флаг открытия панели предпросмотра.
- **Методы:**
  - `updateBodyHtml` – эмитит обновление, при пустом теле сбрасывает `reviewingReply`.
  - `openSendReview` / `closeSendReview` – управление предпросмотром.
  - `confirmSend` – закрывает предпросмотр и эмитит `send`.
  - `replyReviewRecipient` – формирует строку получателя из `senderLabel` и `senderEmail`.
  - `replyReviewSubject` – добавляет префикс «Re:» если отсутствует.
- **Шаблон:**
  - Заголовок «Replying to {senderLabel}» и кнопка Discard.
  - `<RichComposeEditor>` с `placeholder="Write a reply..."`.
  - Кнопки: Cancel, Save Draft, Continue in Compose, Review & Send.
  - При `reviewingReply` – панель предпросмотра с полями To, Subject, Delivery, Undo и кнопками Send / Edit.

## SavedSearchStrip.css

- **Файл:** `frontend/src/domains/communications/components/SavedSearchStrip.css` (полный)
- **Назначение:** глобальные стили для компонентов сохранённых поисков.
- **Содержит классы:**
  - `.saved-search-strip` – контейнер панели.
  - `.saved-search-group`, `.saved-search-virtual-scroll`, `.saved-search-virtual-track`, `.saved-search-virtual-row` – виртуальный скроллинг.
  - `.saved-search-chip` – чип поиска (название, счётчик, hover/active состояния).
  - `.saved-search-tool` – кнопки действий (edit/delete).
  - `.saved-search-form`, `.saved-search-field`, `.saved-search-effective-query` – элементы формы.
  - `.saved-search-group-builder`, `.saved-search-rule-row`, `.saved-search-rule-empty` – редактор групп правил.
  - `.saved-search-primary`, `.saved-search-secondary`, `.saved-search-danger` – кнопки действий формы.
  - `.saved-search-skeleton` – скелетон загрузки.

## TelegramCommunicationsPanel

- **Файл:** `frontend/src/domains/communications/providers/telegram/views/TelegramCommunicationsPanel.vue` (обрезан)
- **Назначение:** представление для работы с Telegram-чатами и сообщениями.
- **Зависимости:** `useI18n` для локализации.
- **Запросы/мутации:**
  - `useTelegramChatsQuery` – список чатов.
  - `useTelegramMessagesQuery` – сообщения выбранного чата (по `account_id` и `provider_chat_id`).
  - `useTelegramMessageSearchQuery` – поиск по сообщениям.
  - `useSendTelegramMessageMutation`, `useReplyTelegramMessageMutation`, `useEditTelegramMessageMutation`, `useDeleteTelegramMessageMutation`, `usePinTelegramMessageMutation`.
- **Состояние:**
  - `selectedConversationId`, `draftText`, `searchText`, `actionMessage`, `actionError`.
- **Вычисляемые свойства:**
  - `chats` – список чатов.
  - `selectedChat` – текущий чат (по `provider_chat_id`) или первый из списка.
  - `messages` – сообщения выбранного чата.
  - `visibleMessages` – результаты поиска, если введён текст, иначе все сообщения.
  - `isBusy` – активна любая мутация.
- **Методы:**
  - `sendMessage()` – отправка текста из `draftText` в текущий чат.
  - `replyToMessage(message)` – ответ на конкретное сообщение (текст из `draftText`).
  - `editMessage(message)` – редактирование через `window.prompt`.
  - `deleteMessage(message)` – локальное удаление.
  - `togglePin(message)` – переключение закрепления.
  - `messageTime(message)` – форматирование времени.
  - `messagePreview(chat)` – последнее сообщение чата или `sync_state`.
- **Шаблон (частично):**
  - Заголовок с поиском.
  - Трёхпанельная компоновка: список чатов, панель сообщений с кнопками действий (Reply, Edit, Pin, Delete), форма отправки нового сообщения.
  - Каждое сообщение отображается как «пузырь» с отправителем, временем, текстом.
- **Примечание:** файл обрезан; полные стили и завершающая часть шаблона не видны.
```

### Source coverage / Покрытие источников

| Source file | Covered facts |
|-------------|---------------|
| `frontend/src/domains/communications/components/MessageBodyTab.vue` | Props, emits, computed (body rendering, remote image proxy, summary/extraction/knowledge sections), template structure, sandboxed iframe logic. |
| `frontend/src/domains/communications/components/MessageHeadersTab.vue` | Props, table fields (from, to, subject, date, channel, id, account, state, importance), fallback state. |
| `frontend/src/domains/communications/components/MessageLocalIntelligencePanel.vue` | Props, mutations (`useExplainMessageMutation`, `useDetectMessageLanguageMutation`), reactive state, methods, template (importance reasons, language detection). |
| `frontend/src/domains/communications/components/MessageRelatedTab.vue` | Props, emits (toggle, export, redirect, labels, snooze), computed labels/snooze, quickLabels, snoozePreset logic, template sections. |
| `frontend/src/domains/communications/components/MessageTimelineTab.vue` | Props, entries derivation from `occurred_at`, `projected_at`, `local_state_changed_at`, `ai_summary_generated_at`, timeline rendering. |
| `frontend/src/domains/communications/components/MessageTrustReviewPanel.vue` | Props, emits, computed auth/signature/smartCc, authChecks derivation, security review and recipient suggestion template. |
| `frontend/src/domains/communications/components/OutboxStatusStrip.vue` | Props, emits (undo, loadMore, prefetchMore), imports (`visibleOutboxStatusItems`, `outboxStatusPresentation`), template with skeleton/error/items/load-more, tone classes. |
| `frontend/src/domains/communications/components/RichComposeEditor.vue` | Props, emits, TipTap integration, paste/drop sanitization, command runner for formatting, toolbar structure. Note: truncated, full styles and potential handlers not visible. |
| `frontend/src/domains/communications/components/SavedSearchRuleGroupEditor.vue` | Props, emits, recursive group/rule editing, addRule/addGroup/removeNode methods, template with rule rows and nested groups. |
| `frontend/src/domains/communications/components/SavedSearchStrip.css` | Global CSS classes for strip, virtual scroll, chips, forms, rule editor, buttons. |
| `frontend/src/domains/communications/components/SavedSearchStrip.vue` | Props, emits, queries/mutations, virtual scrolling, form state, dialog management, preset application, query-tree synchronization. Note: truncated. |
| `frontend/src/domains/communications/components/TemplateRecipientMappingPanel.vue` | Props, emits, mapping fields (to/cc/bcc Variable), updateMappingField method, template structure. |
| `frontend/src/domains/communications/components/TemplateSaveForm.vue` | Props, emits, form validation, save mode labeling, template structure. |
| `frontend/src/domains/communications/components/ThreadAttachmentInsightPanel.vue` | Props, panelMode state, preview/archive inspection queries, translation mutation, template for preview and archive panels. Note: truncated. |
| `frontend/src/domains/communications/components/ThreadConversationView.vue` | Props, emits, expansion management, inline reply state, thread translation, body splitting/preview, template with message list and inline composer. Note: truncated. |
| `frontend/src/domains/communications/components/ThreadInlineReplyComposer.vue` | Props, emits, reply review panel, recipient/subject formatting, RichComposeEditor integration, template. |
| `frontend/src/domains/communications/providers/telegram/views/TelegramCommunicationsPanel.vue` | Queries/mutations for Telegram, chat selection, message actions (send/reply/edit/delete/pin), three-pane layout. Note: truncated. |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений между кодом и документацией не видно – текущая wiki-страница и другие артефакты не были встроены в context pack. Потенциальный риск дрифта связан с внешними типами (`CommunicationMessageDetailResponse`, `CommunicationMessageInsight`, `SavedSearchRuleGroup` и др.) и хелперами (`aiSummaryContractFromMetadata`, `communicationExtractionSectionsFromInsight` и пр.), определения которых не включены в данный чанк. Без них невозможно подтвердить полную согласованность интерфейсов.
