---
chunk_id: 145-source-frontend-part-005
batch_id: batch-20260628T214902
group: frontend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 145-source-frontend-part-005 — frontend/source

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Страница `components/frontend.md` дополняется документацией по утилитам, компонентам и формам из слоя `communications` фронтенда, предоставленным в этом чанке (`145-source-frontend-part-005`). Добавлены описания логики статусов отправки, расширений rich‑compose редактора, обработки HTML, представлений правил поиска, библиотеки шаблонов, представления цепочек сообщений, перетаскивания папок, табов секций и нескольких форм. Каждый описанный факт опирается исключительно на встроенные исходные файлы.

## Предложенные страницы

#### `components/frontend.md`

```markdown
# Фронтенд-компоненты и формы Communications

В этом документе собраны утилиты, компоненты и формы домена `communications`, реализованные на Vue 3 / TypeScript.

---

## Статус отправки (`outboxStatus`)

Файл: `frontend/src/domains/communications/components/outboxStatus.ts`

### `outboxStatusPresentation`

Возвращает `OutboxStatusPresentation` — унифицированное представление для отображения статуса элемента исходящих (`CommunicationOutboxItem`). Поля презентации:

- `title` — короткий заголовок статуса;
- `detail` — дополнительная строка с подробностями;
- `tone` — цветовой тон: `'neutral'`, `'success'`, `'warning'`, `'danger'`, `'muted'`;
- `icon` — идентификатор иконки (формат `tabler:*`);
- `canUndo` — доступна ли отмена отправки;
- `isVisible` — следует ли показывать элемент в сокращённом списке.

Приоритет при выборе статуса:

1. Если метаданные содержат `latest_read_receipt` с `receipt_kind === 'read'` — статус `'Read'`.
2. Если есть `delivery_status`:
   - `'failed'` → `'Delivery failed'` (в деталях SMTP‑статус, если передан);
   - `'delayed'` → `'Delivery delayed'`;
   - `'delivered'` → `'Delivered'`.
3. Если действует окно отмены (`canUndo`) — `'Undo available'`.
4. Если статус элемента `'scheduled'` и больше одной попытки с ошибкой — `'Retry scheduled'`.
5. Иначе по значению `item.status`:
   - `'scheduled'` → `'Scheduled'`;
   - `'queued'` → `'Queued'`;
   - `'sending'` → `'Sending'`;
   - `'failed'` → `'Send failed'`;
   - `'canceled'` → `'Canceled'`.
6. По умолчанию (остальные состояния) — `'Sent'` с `isVisible: false`.

`canUndo` вычисляется как: `item.status` равен `'queued'` или `'scheduled'`, `undo_deadline_at` задан и его timestamp больше или равен текущему времени (`now`). Временные метки форматируются через `Intl.DateTimeFormat` с часовым поясом UTC и форматом `en‑US` (месяц, день, часы, минуты).

### `visibleOutboxStatusItems`

Сортирует и фильтрует массив `CommunicationOutboxItem`:

- оставляет только элементы, для которых `outboxStatusPresentation` возвращает `isVisible: true`;
- сортирует по приоритету статуса (failed=0, sending=1, queued/scheduled=2, наличие read‑receipt/delivery‑status=3, canceled=4, остальные=5) и, при равных, по убыванию `updated_at`;
- возвращает не более `maxItems` (по умолчанию 6).

---

## Расширения Rich Compose (`richComposeExtensions`)

Файл: `frontend/src/domains/communications/components/richComposeExtensions.ts`

Массив `richComposeExtensions` содержит [TipTap](https://tiptap.dev/) ноды и марки для редактора rich‑текста в режиме создания письма.

**Поддерживаемые ноды (block):**
- `doc` (корневая);
- `paragraph` (с атрибутом `textAlign`, валидируется через `normalizeMailComposeTextAlign`);
- `heading` (только уровни 2 и 3, так же с `textAlign`);
- `bulletList` / `orderedList` / `listItem`;
- `blockquote`;
- `text` (inline).

**Марки (inline):**
- `bold` (парсит `strong`, `b`, `font-weight` со значением `bold`/`bolder` или ≥500; рендерит `strong`);
- `italic` (парсит `em`, `i`, `font-style: italic`; рендерит `em`);
- `link` (атрибут `href` нормализуется через `normalizeMailComposeLinkHref`; при рендере добавляет `rel="noopener noreferrer" target="_blank"`; если `href` невалидный, оборачивает в `span`).

---

## HTML‑помощники Rich Compose (`richComposeHtml`)

Файл: `frontend/src/domains/communications/components/richComposeHtml.ts`

### Конвертация plain text ↔ HTML

- `plainTextToComposeHtml(text)` — разбивает текст по двойным переводам строки, экранирует HTML‑символы и оборачивает каждый блок в `<p>`. Если вход пустой или состоит из пробелов, возвращает `<p></p>`.
- `htmlToComposePlainText(html)` — удаляет теги, заменяет некоторые элементы (`</p>`, `<br>`) на переводы строк, декодирует базовые HTML‑сущности и возвращает очищенный plain text.

### Добавление подписи

- `appendPlainTextSignature(body, signature)` — добавляет подпись через `\n\n`.
- `appendHtmlSignature(bodyHtml, signature)` — экранирует подпись и вставляет `<p></p>` перед ней, если тело не пустое.

### Нормализация ссылок

`normalizeMailComposeLinkHref(value)`:
- обрезает пробелы и отклоняет строки с пробелами внутри;
- если нет схемы, добавляет `https://`;
- парсит как `URL` и принимает только протоколы `http:`, `https:`, `mailto:`;
- отклоняет `javascript:`, `data:` и прочие;
- для mailto требует непустой `pathname`;
- для http/https отклоняет URL с именем пользователя или паролем;
- при любой ошибке парсинга возвращает `null`.

### Нормализация выравнивания текста

`normalizeMailComposeTextAlign(value)` возвращает `'left'`, `'center'` или `'right'` (регистронезависимо, с обрезкой пробелов), иначе `null`.

### Санитизация вставленного HTML

`sanitizeMailComposePastedHtml(html)`:
- пропускает содержимое опасных тегов (`head`, `iframe`, `link`, `meta`, `object`, `script`, `style`, `svg`);
- нормализует теги: `b` → `strong`, `i` → `em`, `h1` → `h2`, `h4`–`h6` → `h3`, `div` → `p`;
- для `a` проверяет `href` через `normalizeMailComposeLinkHref`; при успехе добавляет `rel="noopener noreferrer" target="_blank"`;
- для `p`, `h2`, `h3` сохраняет только допустимый `style="text-align: ..."`, остальные атрибуты/style удаляются;
- закрывает открытые теги в правильном порядке;
- если результат пуст, возвращает `<p></p>`.

---

## Представление дерева правил сохранённого поиска (`savedSearchRuleTreePresentation`)

Файл: `frontend/src/domains/communications/components/savedSearchRuleTreePresentation.ts`

- `savedSearchRuleGroupDepthLabel(depth)` — возвращает `'Root group'` для `depth <= 0`, иначе `'Group {depth + 1}'` (например, 1 → "Group 2").
- `savedSearchRuleGroupSummary(group)` — сводка группы:
  - режим: `'All conditions'` или `'Any condition'` (по полю `group.matchMode`);
  - количество правил (`rule`), только если > 0;
  - количество вложенных групп (`nested group`), с учётом множественного числа;
  - если нет ни правил, ни групп — `'Empty'`;
  - части соединяются через ` · `.

---

## Библиотека шаблонов (`templateLibrary`)

Файл: `frontend/src/domains/communications/components/templateLibrary.ts`

### Категории шаблонов

Определены категории:
- `'mail-merge'` — есть переменные;
- `'recipient-aware'` — среди переменных есть recipient‑алиасы;
- `'static-copy'` — нет переменных;
- `'needs-attention'` — имеются `malformed_placeholders` или `undeclared_variables`.

Функция `deriveTemplateLibraryCategories(template)` возвращает массив подходящих категорий. `templateMatchesLibraryCategory(template, category)` проверяет вхождение категории.

### Фильтрация и сортировка

- `filterTemplateLibraryTemplates(templates, query, category)` — фильтрует по категории и поисковому запросу (по полям `name`, `subject_template`, `body_template`, `variables`).
- `orderTemplateLibraryTemplates(templates)` — сортирует по убыванию `updated_at`, затем по имени (регистронезависимо).
- `formatTemplateUpdatedLabel(timestamp)` — форматирует дату как `MMM DD` (en‑US).

### Работа с переменными получателей

Псевдонимы (алиасы) переменных получателей:
- `toVariable`: `['recipient', 'to', 'email', 'recipient_email']`
- `ccVariable`: `['cc', 'cc_email']`
- `bccVariable`: `['bcc', 'bcc_email']`

- `inferRecipientVariableMapping(variables)` — находит первые совпадения для to/cc/bcc.
- `applyTemplateRecipientMapping(currentValues, mapping, context)` — заполняет значения to/cc/bcc из контекста (поля `toText`, `ccText`, `bccText`).
- `buildTemplateRecipientPreviewRows(...)` — разбивает `toText` (через `splitComposeRecipients`) и для каждого адреса создаёт строку предпросмотра с `row_id = 'recipient-{index}'` и переменными.
- `recipientPreviewSummary(context)` — возвращает строку вида `"2 To · 1 CC · 0 BCC"`.

### Сохранение шаблона

- `suggestTemplateSaveName(subject, selectedTemplateName, { duplicate })` — предлагает имя: при дублировании добавляет `' copy'`, иначе берёт тему или имя выбранного шаблона.

---

## Представление цепочек сообщений (`threadConversationPresentation`, `threadMessageBody`)

Файлы:
- `frontend/src/domains/communications/components/threadConversationPresentation.ts`
- `frontend/src/domains/communications/components/threadMessageBody.ts`

### Разбивка тела сообщения

`splitThreadMessageBody(bodyText)` разделяет текст на:
- `mainText` — основная часть;
- `quotedText` — цитируемый «хвост».

Цитируемый блок определяется по первой строке, начинающейся с `>` или совпадающей с шаблоном `"On ... wrote:"` (регистронезависимо, не в первой строке). Если такого блока нет, весь текст считается основным.

`previewThreadMessageBody(message, expanded)` возвращает сжатый превью:
- если `expanded`, берёт основной текст (или полный, если основного нет), иначе полный `body_text`;
- заменяет последовательности пробелов на одиночный пробел, обрезает до 220 символов, добавляя `...` при превышении.

### Управление раскрытием

- `defaultExpandedThreadMessageIds(messages)` — возвращает `Set` с `message_id` последнего сообщения.
- `hasQuotedThreadMessages(messages)` — проверяет, есть ли хотя бы в одном сообщении цитируемый текст.
- `summarizeThreadExpansion(messages, expandedMessageIds)` — возвращает `{ expandedCount, canExpandAll, canCollapseAll }`.

---

## Drag‑and‑drop сортировка папок (`useCommunicationFolderReorder`)

Файл: `frontend/src/domains/communications/components/useCommunicationFolderReorder.ts`

Composable `useCommunicationFolderReorder(folders, updateFolder)` управляет перетаскиванием папок. Возвращает:
- реактивные `sourceId`, `targetId`, `status`, `error`, `isReordering`;
- `canHandleDragOver(event)` — проверяет наличие drag‑data нужного типа и отсутствие активного запроса;
- `handleDragStart(event, folder)` — устанавливает `sourceId` и drag‑data;
- `handleDragEnd()` — сбрасывает source/target;
- `handleDrop(event, folder)` — при возможности вычисляет обновления (`buildCommunicationFolderReorderUpdates`), последовательно вызывает `updateFolder` для каждого изменения, устанавливает `status` (через `mailFolderReorderStatus`) или `error`.

Подробности типов и payload’ов вынесены в модуль `mailFolderOrdering` (не включён в этот контекст).

---

## Табы секций (`sectionTabs`)

Файл: `frontend/src/domains/communications/constants/sectionTabs.ts`

Экспортируется константа `communicationSectionTabs` — массив объектов `{ id, label, icon }`:

| id            | label        | icon                    |
|---------------|--------------|-------------------------|
| `unified`     | Unified      | `tabler:inbox`          |
| `inbox`       | Inbox        | `tabler:mail`           |
| `needs_reply` | Need Reply   | `tabler:message-reply`  |
| `waiting`     | Waiting      | `tabler:clock`          |
| `done`        | Done         | `tabler:check`          |
| `archived`    | Archived     | `tabler:archive`        |

---

## Формы

### `attachmentSearchForm`

Файл: `frontend/src/domains/communications/forms/attachmentSearchForm.ts`

Схема `attachmentSearchFormSchema` (Zod):
- `query` — строка ≤500 символов;
- `content_type` — строка ≤120 символов;
- `scan_status` — пустая строка или одно из: `'not_scanned'`, `'clean'`, `'suspicious'`, `'malicious'`, `'failed'`.

`attachmentSearchFormToRequest(values, accountId)` — строит `AttachmentSearchRequest`:
- всегда `limit: 50`;
- добавляет `account_id`, если передан;
- добавляет непустые `query`, `content_type`, `scan_status`.

### `bilingualReplyFlowForm`

Файл: `frontend/src/domains/communications/forms/bilingualReplyFlowForm.ts`

Схема `bilingualReplyFlowFormSchema`:
- `replyTextRu` — обязательный русский текст ответа, 1–64 000 символов;
- `tone` — одно из `'formal'`, `'business'`, `'friendly'`, `'short'`, `'detailed'`.

`bilingualReplyFlowFormToRequest(values)` возвращает `{ reply_text_ru, tone }`. Значения по умолчанию: пустой текст, тон `'business'`.

### `certificateForm`

Файл: `frontend/src/domains/communications/forms/certificateForm.ts`

Схема `certificateFormSchema` описывает создание сертификата (`MailCertificateCreateRequest`). Обязательные поля: `cert_id`, `owner_name`, `issuer`. Опциональные: `fingerprint_sha256`, `valid_until` (преобразуется в ISO), `storage_ref`. Перечисления:
- `cert_type`: из `certificateTypeOptions` (по умолчанию `'smime'`);
- `provider`: из `certificateProviderOptions` (по умолчанию `'other'`);
- `storage_kind`: из `certificateStorageKindOptions` (по умолчанию `'encrypted_vault'`);
- `trust_status`: из `certificateTrustStatusOptions` (по умолчанию `'pending_verification'`);
- `usage` — строка, разделённая запятыми; преобразуется в массив строк функцией `splitUsage` (значение по умолчанию `'signing, encryption'`).

### `composeDraftAutosave`

Файл: `frontend/src/domains/communications/forms/composeDraftAutosave.ts`

- `buildComposeDraftPayload(form)` — формирует payload типа `ComposeDraftPayload` из `ComposeFormModel`: разбирает получателей через `splitComposeRecipients`, для HTML‑режима включает `body_html`, `scheduled_send_at` преобразует в ISO, в `metadata` сохраняет `compose_mode`.
- `composeDraftHasAutosaveContent(form)` — возвращает `true`, если заполнено хотя бы одно из полей: to/cc/bcc, subject, body, bodyHtml, scheduledSendAt.
- `useComposeDraftAutosave(options)` — Vue composable с отложенным автосохранением:
  - `schedule()` — ставит таймер на `delayMs` (по умолчанию 2000 мс);
  - `flush()` — отменяет таймер и немедленно сохраняет;
  - `cancel()` — сбрасывает таймер;
  - При наличии активной Vue‑области (`getCurrentScope`) автоматически вызывает `cancel` при dispose.

### `composeValidation`

Файл: `frontend/src/domains/communications/forms/composeValidation.ts`

- `splitComposeRecipients(value)` — разбивает строку по запятым, извлекает email из угловых скобок (`<email>`).
- `toComposeValidationValues(form)` — маппит `ComposeFormModel` в плоский объект для валидации.
- `composeSendSchema` (Zod):
  - `accountId` — непустая строка;
  - `toText` — обязательные получатели, каждый должен проходить проверку по шаблону `EMAIL_ADDRESS_PATTERN` (упрощённый email);
  - `ccText`, `bccText` — опциональные, но если заполнены — аналогичная проверка;
  - `subject` — ≤ 998 символов;
  - `body` — ≤ 1 000 000 символов;
  - `inReplyTo` — допускает `null`.
- `useComposeValidation(formSource)` — composable на основе vee‑validate: отслеживает изменения формы, предоставляет `errors` и `validateForSend()`.

### `mailFolderForm`

Файл: `frontend/src/domains/communications/forms/mailFolderForm.ts`

Схема `mailFolderFormSchema`:
- `name` — обязательная, ≤ 120 символов;
- `description` — ≤ 500 символов, необязательная;
- `color` — hex‑строка (`#RRGGBB`) или пустая;
- `sort_order` — целое неотрицательное число.

Дополнительные функции:
- `splitCommunicationFolderName(name)` — разделяет путь (через `/`) на `parentPath` и `leafName`.
- `composeCommunicationFolderName(parentPath, leafName)` — собирает путь обратно.
- `mailFolderParentPathOptions(folders, editingFolder)` — возвращает список возможных родительских путей, исключая собственный и дочерние.
- `validateCommunicationFolderParentPath(...)` — возвращает сообщение об ошибке, если путь приводит к циклической иерархии.
- `mailFolderFormToInput(values, accountId)` — строит `CommunicationFolderInput`, обнуляя пустые опциональные поля до `null`.
- `mailFolderDeleteDialogCopy(folder)` — возвращает текст диалога удаления (заголовок, сообщение, кнопка).
- `mailFolderMessageCountLabel(folder)` — возвращает строку с неотрицательным целым количеством сообщений.
```

## Покрытие источников

| Файл | Факты, покрытые в предлагаемой странице |
|------|------------------------------------------|
| `outboxStatus.ts` | Функция `outboxStatusPresentation`, её состояния, приоритеты, формат `OutboxStatusPresentation`, `visibleOutboxStatusItems`, вспомогательные функции `canUndo`, `timestampDetail`, приоритет `statusPriority` |
| `richComposeExtensions.ts` | Состав `richComposeExtensions` (ноды `doc`, `paragraph`, `heading`, списки, `blockquote`, `text`; марки `bold`, `italic`, `link`), атрибуты (`textAlign`, `href`), привязка к `normalizeMailComposeTextAlign` и `normalizeMailComposeLinkHref` |
| `richComposeHtml.test.ts` | Ожидаемое поведение функций `plainTextToComposeHtml`, `htmlToComposePlainText`, `append*Signature`, `normalizeMailComposeLinkHref`, `normalizeMailComposeTextAlign`, `sanitizeMailComposePastedHtml` (использованы для подтверждения описаний в разделе `richComposeHtml`) |
| `richComposeHtml.ts` | Все перечисленные функции преобразования текста, добавления подписей, нормализации ссылок и выравнивания, санитизации вставленного HTML, внутренние детали (`SKIPPED_PASTE_CONTENT_TAGS`, `NORMALIZED_PASTE_TAGS`, разбор тегов) |
| `savedSearchRuleTreePresentation.test.ts` | Примеры работы `savedSearchRuleGroupDepthLabel` и `savedSearchRuleGroupSummary` |
| `savedSearchRuleTreePresentation.ts` | Функции `savedSearchRuleGroupDepthLabel`, `savedSearchRuleGroupSummary`, формат сводки группы |
| `templateLibrary.test.ts` | Факты о категориях, маппинге получателей, preview‑строках, summary, предложении имени |
| `templateLibrary.ts` | Все экспортируемые функции и типы (`deriveTemplateLibraryCategories`, маппинг, preview‑строки, `recipientPreviewSummary`, `suggestTemplateSaveName`, алиасы, фильтрация/сортировка) |
| `threadConversationPresentation.test.ts` | Поведение `defaultExpandedThreadMessageIds`, `hasQuotedThreadMessages`, `summarizeThreadExpansion` |
| `threadConversationPresentation.ts` | Функции `defaultExpandedThreadMessageIds`, `hasQuotedThreadMessages`, `summarizeThreadExpansion` |
| `threadMessageBody.test.ts` | Примеры разбивки тела и превью |
| `threadMessageBody.ts` | Функции `splitThreadMessageBody` (логика определения цитируемого блока), `previewThreadMessageBody` (логика превью) |
| `useCommunicationFolderReorder.ts` | Интерфейс composable: реактивные поля, методы `canHandleDragOver`, `handleDragStart`, `handleDragEnd`, `handleDrop`, использование `mailFolderOrdering` (без деталей, т.к. модуль не предоставлен) |
| `sectionTabs.ts` | Состав массива `communicationSectionTabs` |
| `attachmentSearchForm.test.ts` | Преобразование формы в запрос |
| `attachmentSearchForm.ts` | Схема, значения по умолчанию, `attachmentSearchFormToRequest` |
| `bilingualReplyFlowForm.test.ts` | Валидация и преобразование |
| `bilingualReplyFlowForm.ts` | Схема, значения по умолчанию, `bilingualReplyFlowFormToRequest`, перечень тонов |
| `certificateForm.ts` | Схема, поля, значения по умолчанию, `certificateFormToCreateRequest`, `splitUsage`, `localDateTimeToIso` |
| `composeDraftAutosave.test.ts` | Структура payload, условие наличия контента, debounce и flush |
| `composeDraftAutosave.ts` | `buildComposeDraftPayload`, `composeDraftHasAutosaveContent`, `useComposeDraftAutosave` (schedule, flush, cancel, safe, onScopeDispose), `datetimeLocalToIso` |
| `composeValidation.test.ts` | Разбор получателей, валидация |
| `composeValidation.ts` | `splitComposeRecipients`, `toComposeValidationValues`, `composeSendSchema` (Zod), `useComposeValidation` |
| `mailFolderForm.test.ts` | Иерархия имён, валидация, текст диалога удаления, подсчёт сообщений |
| `mailFolderForm.ts` | Схема, утилиты для работы с иерархическими именами, `mailFolderParentPathOptions`, `validateCommunicationFolderParentPath`, `mailFolderFormToInput`, `mailFolderDeleteDialogCopy`, `mailFolderMessageCountLabel` |

## Исходные файлы

- [`frontend/src/domains/communications/components/outboxStatus.ts`](../../../../frontend/src/domains/communications/components/outboxStatus.ts)
- [`frontend/src/domains/communications/components/richComposeExtensions.ts`](../../../../frontend/src/domains/communications/components/richComposeExtensions.ts)
- [`frontend/src/domains/communications/components/richComposeHtml.test.ts`](../../../../frontend/src/domains/communications/components/richComposeHtml.test.ts)
- [`frontend/src/domains/communications/components/richComposeHtml.ts`](../../../../frontend/src/domains/communications/components/richComposeHtml.ts)
- [`frontend/src/domains/communications/components/savedSearchRuleTreePresentation.test.ts`](../../../../frontend/src/domains/communications/components/savedSearchRuleTreePresentation.test.ts)
- [`frontend/src/domains/communications/components/savedSearchRuleTreePresentation.ts`](../../../../frontend/src/domains/communications/components/savedSearchRuleTreePresentation.ts)
- [`frontend/src/domains/communications/components/templateLibrary.test.ts`](../../../../frontend/src/domains/communications/components/templateLibrary.test.ts)
- [`frontend/src/domains/communications/components/templateLibrary.ts`](../../../../frontend/src/domains/communications/components/templateLibrary.ts)
- [`frontend/src/domains/communications/components/threadConversationPresentation.test.ts`](../../../../frontend/src/domains/communications/components/threadConversationPresentation.test.ts)
- [`frontend/src/domains/communications/components/threadConversationPresentation.ts`](../../../../frontend/src/domains/communications/components/threadConversationPresentation.ts)
- [`frontend/src/domains/communications/components/threadMessageBody.test.ts`](../../../../frontend/src/domains/communications/components/threadMessageBody.test.ts)
- [`frontend/src/domains/communications/components/threadMessageBody.ts`](../../../../frontend/src/domains/communications/components/threadMessageBody.ts)
- [`frontend/src/domains/communications/components/useCommunicationFolderReorder.ts`](../../../../frontend/src/domains/communications/components/useCommunicationFolderReorder.ts)
- [`frontend/src/domains/communications/constants/sectionTabs.ts`](../../../../frontend/src/domains/communications/constants/sectionTabs.ts)
- [`frontend/src/domains/communications/forms/attachmentSearchForm.test.ts`](../../../../frontend/src/domains/communications/forms/attachmentSearchForm.test.ts)
- [`frontend/src/domains/communications/forms/attachmentSearchForm.ts`](../../../../frontend/src/domains/communications/forms/attachmentSearchForm.ts)
- [`frontend/src/domains/communications/forms/bilingualReplyFlowForm.test.ts`](../../../../frontend/src/domains/communications/forms/bilingualReplyFlowForm.test.ts)
- [`frontend/src/domains/communications/forms/bilingualReplyFlowForm.ts`](../../../../frontend/src/domains/communications/forms/bilingualReplyFlowForm.ts)
- [`frontend/src/domains/communications/forms/certificateForm.ts`](../../../../frontend/src/domains/communications/forms/certificateForm.ts)
- [`frontend/src/domains/communications/forms/composeDraftAutosave.test.ts`](../../../../frontend/src/domains/communications/forms/composeDraftAutosave.test.ts)
- [`frontend/src/domains/communications/forms/composeDraftAutosave.ts`](../../../../frontend/src/domains/communications/forms/composeDraftAutosave.ts)
- [`frontend/src/domains/communications/forms/composeValidation.test.ts`](../../../../frontend/src/domains/communications/forms/composeValidation.test.ts)
- [`frontend/src/domains/communications/forms/composeValidation.ts`](../../../../frontend/src/domains/communications/forms/composeValidation.ts)
- [`frontend/src/domains/communications/forms/mailFolderForm.test.ts`](../../../../frontend/src/domains/communications/forms/mailFolderForm.test.ts)
- [`frontend/src/domains/communications/forms/mailFolderForm.ts`](../../../../frontend/src/domains/communications/forms/mailFolderForm.ts)

## Кандидаты на drift

- Модуль `mailFolderOrdering` (импортируется из `./mailFolderOrdering`) не включён в данный context pack. Невозможно подтвердить, что типы и функции `MAIL_FOLDER_REORDER_DRAG_TYPE`, `buildCommunicationFolderReorderUpdates`, `createCommunicationFolderReorderPayload`, `hasCommunicationFolderReorderDragType`, `mailFolderReorderStatus`, `parseCommunicationFolderReorderPayload` существуют в актуальной кодовой базе в том виде, в каком они используются в `useCommunicationFolderReorder.ts`.
- Типы `CommunicationOutboxItem`, `CommunicationTemplate`, `ThreadMessage`, `CommunicationFolder`, `CommunicationFolderUpdate`, `ComposeFormModel`, `SavedSearchRuleGroup`, `BilingualReplyFlowRequest`, `BilingualReplyTone`, `CertificateProvider`, `CertificateStorageKind`, `CertificateTrustStatus`, `CertificateType`, `MailCertificateCreateRequest`, `AttachmentScanStatus`, `AttachmentSearchRequest`, `CommunicationSectionId`, `CommunicationFolderInput`, а также перечисления `certificateTypeOptions`, `certificateProviderOptions`, `certificateStorageKindOptions`, `certificateTrustStatusOptions` импортируются из файлов типов, не предоставленных в чанке. Их точная структура и возможные расхождения с текущей реализацией не могут быть оценены.
- Функции‑фабрики `createSavedSearchRuleCondition`, `createSavedSearchRuleGroup` из `../forms/savedSearchForm` не включены; в тестах `savedSearchRuleTreePresentation.test.ts` они используются для построения тестовых данных, но их актуальность не подтверждена.
- Никаких других расхождений между кодом, документацией и ADR в данном чанке не выявлено.
