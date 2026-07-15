### Summary / Резюме

В данной части исходников представлена основная бизнес-логика домена `communications` (работа с электронной почтой) – хранение секретов, уведомления о доставке, черновики, финансовые записи (инвойсы), связи с наблюдениями (evidence), объяснение важности, экспорт, извлечение задач/заметок, флаги сообщений и папки. Страница `components/backend.md` должна быть обновлена для документирования этих модулей на русском языке с акцентом на конкретные структуры, методы и поведения, видимые в исходном коде.

### Proposed pages / Предлагаемые страницы

`components/backend.md`

```markdown
# Backend (Communications)

## Обзор

Модуль `backend/src/domains/communications` содержит бизнес-логику работы с электронной почтой: управление учётными данными провайдеров, обработка уведомлений о доставке, черновики, инвойсы, анализ важности, экспорт, извлечение задач/заметок, флаги сообщений и папки.

## Ядро (Core)

### Хранилище (`store`)

`CommunicationIngestionStore` – обёртка над `PgPool` для инъекции зависимостей.

```rust
pub struct CommunicationIngestionStore {
    pub(super) pool: PgPool,
}
impl CommunicationIngestionStore {
    pub fn new(pool: PgPool) -> Self;
    pub(crate) fn pool(&self) -> PgPool;
}
```

### Секреты (`secrets`)

- `CommunicationIngestionStore` предоставляет методы для привязки секретов к аккаунтам провайдера:
  - `bind_provider_account_secret` – создаёт `ProviderAccountSecretBinding`.
  - `provider_account_secret_bindings` – список всех привязок для аккаунта.
  - `provider_account_secret_binding` – получение привязки по `secret_purpose`.

- `ProviderCredentialReader<R: SecretResolver>` – читает креденшл по `account_id` и `secret_purpose`:
  1. Проверяет непустоту `account_id`.
  2. Находит `ProviderAccountSecretBinding` для аккаунта и цели, иначе ошибка `MissingBinding`.
  3. Получает `SecretReference`, иначе ошибка `MissingSecretReference`.
  4. Проверяет совместимость `secret_kind` с `secret_purpose`, иначе `IncompatibleSecretKind`.
  5. Вызывает `resolver.resolve` для получения самого секрета.
  Результат: `ProviderCredential { binding, reference, secret }`.

### Валидация (`validation`)

- `validate_non_empty(field_name, value)` – возвращает `EmptyField`, если строка после `trim()` пуста.
- `validate_object(field_name, value)` – возвращает `NonObjectJson`, если `serde_json::Value` не является объектом.

## Уведомления о доставке (`delivery_notifications`)

### Типы

- `NewCommunicationDeliveryNotification` – входящее сырое уведомление (DSN/MDN) с полями `account_id`, `raw_message`, `received_at`, `provider_record_id`, `raw_record_id`.
- `ProviderDeliveryEventKind` – перечисление: `Delivered`, `Delayed`, `Failed`, `Read`. Метод `delivery_status()` отображает все кроме `Read` в `OutboxDeliveryStatus`.
- `NewProviderDeliveryEvent` – структурированное событие от провайдера: `account_id`, `provider_message_id`, `event_kind`, `recipient`, `occurred_at`, `source_kind`, `smtp_status` и др.
- `CommunicationDeliveryNotificationRecord` – объединённая запись о доставке или прочтении.

### Хранилище (`CommunicationDeliveryNotificationStore`)

- `record(notification)` – парсит `raw_message` и в зависимости от результата:
  - `DeliveryStatus` → записывает статус через `CommunicationOutboxStore::record_delivery_status`.
  - `ReadReceipt` → записывает подтверждение через `CommunicationReadReceiptStore::record`.
- `record_provider_event(event)` – на основе `event_kind`:
  - Если `delivery_status() != None` → статус доставки.
  - Иначе → подтверждение прочтения (требуется `recipient`).
- Функция `provider_event_from_delivery_notification` конвертирует сырое уведомление в `NewProviderDeliveryEvent`, сохраняя источник (`dsn` или `mdn`).
- Публичные функции `project_accepted_mail_delivery_signal_if_runtime_allows` и `consume_accepted_mail_delivery_signal` обрабатывают события из шины при условии активного консюмера (`COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER`). Если runtime разрешает, парсится `signal.accepted.mail.delivery_status` в `NewProviderDeliveryEvent` и записывается.

## Черновики (`drafts`)

### Модели

- `CommunicationDraft` – полная запись черновика: `draft_id`, `account_id`, `to_recipients` (и `cc`/`bcc`), `subject`, `body_text`, `body_html`, `in_reply_to`, `references`, `status` (`DraftStatus`), `scheduled_send_at`, `send_attempts`, `last_error`, `metadata`, временные метки.
- `DraftStatus` – `Draft`, `Scheduled`, `Sending`, `Sent`, `Failed`. Имеет методы `as_str` и `parse`.
- `NewCommunicationDraft` – структура для создания/обновления. Валидация требует непустые `draft_id` и `account_id`.

### Хранилище (`CommunicationDraftStore`)

- `upsert(draft)` – выполняет `INSERT ... ON CONFLICT UPDATE`, возвращает `CommunicationDraft`, эмитит доменное событие (`mail.draft.created` / `mail.draft.updated`).
- `upsert_with_observation(..., observation_id, ...)` – дополнительно связывает черновик с наблюдением через `link_mail_entity_in_transaction`.
- `list(account_id?, status?)` – выборка с фильтром, сортировка по `updated_at DESC, draft_id ASC`.
- `list_page(account_id?, status?, cursor?, limit)` – курсорная пагинация: cursor кодирует `updated_at` и `draft_id`, гарантирует стабильный порядок. Возвращает `CommunicationDraftListPage` с `next_cursor`.
- `get(draft_id)` – получение одного черновика (файл обрезан, детали неполные).
- В транзакциях вызывается `ensure_canonical_account_in_transaction` для проверки аккаунта.

## Связи с наблюдениями (`evidence`)

- `link_mail_entity_in_transaction` – привязывает почтовую сущность (draft, folder, …) к `observation_id` через платформенную функцию `link_domain_entity_in_transaction` с доменом `"communications"` и переданным `relationship_kind`.
- `merge_metadata` – объединяет базовый JSON с дополнительным опциональным JSON, переопределяя ключи.

## Объяснение важности (`explain`)

- `explain_importance(message: &ProjectedMessage) -> WhyImportantContext`:
  - Анализирует `importance_score`, ключевые слова в теме и теле письма: `urgent`, `asap`, `invoice`, `payment`, `contract`, `nda`, `deadline`, упоминания вложений, маркетинговые рассылки. Для каждого найденного маркера добавляет строку-причину в `reasons`.
- `smart_cc_suggestions(message) -> Vec<String>` – выдаёт рекомендации по CC на основе содержимого: бухгалтер для финансов, юрист для контрактов, стейкхолдеры для проектных апдейтов.

## Экспорт (`export`)

- `CommunicationExport` – результат экспорта: `format` (`ExportFormat`) и `content` (строка).
- `ExportFormat` – `Eml`, `Markdown`, `Json`. Каждый вариант задаёт `content_type` и расширение файла.
- `export_message(message_store, attachment_store, message_id, format)`:
  - Получает проекцию сообщения и список вложений.
  - Формирует вывод: Markdown с заголовками и полями, EML с заголовками RFC-2822, JSON с основными полями.
  - Ошибки: `NotFound`, `MessageProjection`, `CommunicationStorage`.

## Извлечение задач и заметок (`extract`)

### Сервис `EmailExtractService`

- Конструктор принимает опциональный `SharedAiRuntimePort`.
- `extract_tasks(message)`:
  1. Эвристически ищет строки, начинающиеся с `todo:`, `task:`, `- [ ]` или содержащие `action item`, `please … by …`.
  2. Для каждой строки пытается извлечь дату (`extract_due_date` по ключевым словам `by`, `due`, `deadline`, `before`).
  3. Если доступен LLM runtime, отправляет промпт с truncated телом письма и парсит JSON-ответ, добавляя задачи с источником `"llm"`.
- `extract_notes(message)` – эвристически определяет наличие финансовых, юридических, решающих или дедлайновых паттернов и создаёт `ExtractedNote` с тегами и первыми пятью строками тела.

### Вспомогательные функции

- `extract_due_date(text)` – ищет первое совпадение префикса и возвращает до трёх следующих слов.
- `truncate` – обрезает строку до заданной длины.

## Финансы (`finance`)

- `InvoiceRecord` – запись инвойса со статусом, суммами, контрагентом, проектами/персонами и метаданными.
- `InvoiceStatus` – `Received`, `Recognized`, `NeedsReview`, `Approved`, `Paid`, `Closed`, `Rejected`.
- `CommunicationFinanceStore`:
  - `upsert_invoice(invoice)` – вставка или обновление через `ON CONFLICT (invoice_id)`. Валидация требует непустой `invoice_id`.
  - `list(status?)` – выборка всех инвойсов с опциональным фильтром по статусу, сортировка по `COALESCE(due_date, created_at) DESC`.

## Экспорт фикстур (`fixtures/export`)

### Основной поток

- `export_fixture_messages_from_sync_batch(batch, options)`:
  - Для каждого сообщения из `EmailSyncBatch` извлекает `raw_rfc822_base64` из `payload` (base64, без пробелов), декодирует в бинарный вид.
  - Парсит RFC822 (`parse_rfc822_message`): разделение заголовков и тела, извлечение Subject, From, To, body_text.
  - При режиме `Redacted` заменяет идентифицирующую информацию на хэши (SHA-256, первые 6 байт в hex): адреса – `{prefix}-{hash}@example.invalid`, тема – `"Redacted subject {hash}"`, тело – шаблон с хэшем провайдера, фингерпринтом и длиной оригинала.
  - Возвращает `FixtureCommunicationSourceMessage` с обработанными полями.

### Вспомогательные модули

- `body` – извлечение текстовой части из MIME: multipart (по boundary ищет `text/plain` без вложения), text/html (вырезание тегов), декодирование transfer-encoding (base64, quoted-printable).
- `encoded_words` – декодирование RFC2047 (`=?charset?B?…?=` base64 или `?Q?` quoted-printable).
- `encoding` – декодирование transfer-encoding: `base64`, `quoted-printable` (включая мягкие переносы строк).
- `headers` – парсинг заголовков с учётом folding, декодирование RFC2047 в значениях.
- `redaction` – анонимизация с использованием `sha2::Sha256` и хэшей.
- `text` – нормализация строк и замена пустых значений на значения по умолчанию (например, `"(no subject)"`, `"recipient-unknown@example.invalid"`).

## Флаги сообщений (`flags`)

`MessageFlags` – операции над полем `message_metadata` (JSONB) через `MessageProjectionStore`:

- **Чтение флагов** (статические методы): `is_pinned`, `is_important`, `snooze_until`, `labels`, `is_muted`.
- **Мутации** (асинхронные, с вариантами `_with_observation`):
  - `toggle_pin` / `toggle_important` / `toggle_mute` – инвертируют булево значение в metadata.
  - `snooze(until)` – устанавливает `snooze_until` в RFC3339.
  - `add_label` / `remove_label` – изменяет массив `labels`.
  - Все мутации требуют существующего сообщения (ошибка `NotFound`) и уважают `observation`-линковку при наличии.

## Папки (`folders`)

### Модели

- `CommunicationFolder` – `folder_id`, `account_id`, `name`, `color`, `sort_order`, `message_count`, временные метки.
- `NewCommunicationFolder` – с опциональным `folder_id`.
- `UpdateCommunicationFolder` – все поля опциональны.
- `FolderMessage` – сообщение внутри папки: `folder_id`, `message_id`, `account_id`, `subject`, `sender`, `occurred_at`, `projected_at`, `workflow_state`, `local_state`, `added_at`, `attachment_count`.
- `FolderMessageOperation` – `Copy` / `Move`.
- `FolderMessageActionResponse` – результат операции с папкой.

### Хранилище (`CommunicationFolderStore`)

- `list(query)` – постраничный вывод с курсором (`FolderListCursor`): сортировка по `sort_order ASC, lower(name) ASC, folder_id ASC`. Курсор кодируется в base64 URL-safe.
- `create` / `update` / `delete` – с транзакционной проверкой аккаунта и эмиссией событий (`folder.created`, `folder.updated`, `folder.deleted`). Варианты `_with_observation` добавляют привязку к наблюдению.
- `list_messages(query)` – постраничный вывод сообщений папки с курсором (`FolderMessageCursor` – `added_at` + `message_id`), присоединяются данные сообщений и количество вложений. (Файл обрезан, полная реализация не включена).

### Курсоры

- `FolderListCursor` – `sort_order`, `name_lower`, `folder_id`.
- `FolderMessageCursor` – `added_at`, `message_id`.
- Кодирование/декодирование через `base64::engine::general_purpose::URL_SAFE_NO_PAD` и `serde_json`.

---

*Примечание*: некоторые исходные файлы в данном чанке были обрезаны (delivery_notifications, drafts, folders). Полная документация требует их полного рассмотрения.
```

### Source coverage / Покрытие источников

- **`core/secrets.rs`** – методы `bind_provider_account_secret`, `list`/`get` привязок; структура и логика `ProviderCredentialReader`: валидация `account_id`, поиск биндинга, получение `SecretReference`, проверка совместимости `secret_kind`, вызов resolver.
- **`core/store.rs`** – объявление `CommunicationIngestionStore` с `pool`, конструктор, геттер `pool()`.
- **`core/validation.rs`** – `validate_non_empty` и `validate_object`, их сигнатуры и возвращаемые ошибки (`EmptyField`, `NonObjectJson`).
- **`delivery_notifications.rs`** – структуры `NewCommunicationDeliveryNotification`, `NewProviderDeliveryEvent`, `ProviderDeliveryEventKind`, `CommunicationDeliveryNotificationRecord`; `CommunicationDeliveryNotificationStore::record` и `record_provider_event`; функция `provider_event_from_delivery_notification`; обработка сигнала `signal.accepted.mail.delivery_status` через `consume_accepted_mail_delivery_signal` и проверку runtime (`proect_accepted…`).
- **`drafts.rs`** – модели `CommunicationDraft`, `DraftStatus`, `NewCommunicationDraft` с валидацией; `CommunicationDraftStore::upsert` (INSERT ... ON CONFLICT, события, линковка observation) и `upsert_with_observation`, `list`, `list_page` с курсором на `updated_at`+`draft_id`, `get` (частично из-за обрезки).
- **`evidence.rs`** – `link_mail_entity_in_transaction` и `merge_metadata` для присоединения сущностей к observations.
- **`explain.rs`** – `explain_importance` с проверкой ключевых слов и score, `smart_cc_suggestions` (бухгалтер/юрист/стейкхолдеры); наличие модульных тестов.
- **`export.rs`** – `CommunicationExport`, `ExportFormat` (Eml, Markdown, Json) с `content_type`/`extension`; функция `export_message`, форматирование, ошибки.
- **`extract.rs`** – `EmailExtractService` с эвристическим и LLM-извлечением задач, эвристическое извлечение заметок; `extract_due_date`; тесты.
- **`finance.rs`** – `InvoiceRecord`, `InvoiceStatus`, `CommunicationFinanceStore::upsert_invoice` и `list`, `NewInvoiceRecord::validate`.
- **`fixtures/mod.rs`** – `pub mod export`.
- **`fixtures/export.rs`** – `export_fixture_messages_from_sync_batch`, использование submodules, redaction опции.
- **`fixtures/export/body.rs`** – `body_text_from_part`, `first_text_plain_multipart_part`, `strip_html_tags`.
- **`fixtures/export/encoded_words.rs`** – `decode_rfc2047_words` (поддержка base64 и quoted-printable внутри `=?…?=`).
- **`fixtures/export/encoding.rs`** – `decode_transfer_body` (base64, quoted-printable).
- **`fixtures/export/errors.rs`** – варианты ошибок (MissingRawRfc822, InvalidRawBase64, MalformedRfc822).
- **`fixtures/export/headers.rs`** – `parse_headers`, `header_value`, `content_type_parameter`.
- **`fixtures/export/models.rs`** – `EmailFixturePrivacyMode::Redacted`, `EmailFixtureExportOptions` со значением по умолчанию.
- **`fixtures/export/raw_payload.rs`** – извлечение base64-строки из JSON payload.
- **`fixtures/export/redaction.rs`** – `redact_message` с хэшированием (SHA-256, первые 6 байт), замена email, subject, body.
- **`fixtures/export/rfc822.rs`** – `parse_rfc822_message`, `split_headers_and_body`.
- **`fixtures/export/text.rs`** – `normalize_body_text`, `non_empty_or_default`, `non_empty_recipients`.
- **`flags.rs`** – `MessageFlags` с методами чтения (`is_pinned`, `is_important`, `snooze_until`, `labels`, `is_muted`) и мутации (`toggle_pin`, `snooze`, `add_label`…) с вариантами `_with_observation`; тесты.
- **`folders.rs`** – структуры `CommunicationFolder`, `NewCommunicationFolder`, `UpdateCommunicationFolder`, `FolderMessage`, `FolderMessageOperation`; `CommunicationFolderStore::list`, `create`, `update`, `delete`, `list_messages` (часть обрезана).
- **`folders/cursors.rs`** – `FolderListCursor`, `FolderMessageCursor`, их encode/decode через base64 URL-safe no pad.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений между кодом, документацией или ADR не обнаружено. Возможные риски (без подтверждения в данном чанке):

- Код в `delivery_notifications.rs` и `drafts.rs` обрезан; полные контракты методов могут отличаться от задокументированных.
- В `folders.rs` (обрезан) могут присутствовать дополнительные методы или изменения в семантике операций `Copy`/`Move`, не видимые в данном чанке.
- Поведение `consume_accepted_mail_delivery_signal` зависит от `payload` события `signal.accepted.mail.delivery_status`; формат payload может быть описан в другом месте (ADR или конфигурация), здесь не виден.
- Имена констант событий (например, `mail.draft.created`) не проверяются на соответствие конфигурации внешних подписчиков.

За отсутствием дополнительных артефактов (ADR, конфиги, другие wiki-страницы) подтвердить или опровергнуть дрифт невозможно.
