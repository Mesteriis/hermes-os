### Summary / Резюме

Создаётся обзорная страница `components/backend.md` для русской Obsidian‑wiki, описывающая backend-компоненты домена `communications` на основе предоставленных исходных файлов. Страница объединяет факты о хранилище сообщений, исходящей очереди (outbox), уведомлениях о прочтении, персонах, правилах обработки, многоязычности, rich‑шаблонах и сохранённых поисках. Все утверждения строго опираются на встроенный код.

### Proposed pages / Предлагаемые страницы

#### `components/backend.md`

# Компоненты бэкенда: домен communications

## Обзор домена

Корневой модуль `communications` реэкспортирует следующие подмодули:

```rust
pub mod actions;
pub mod ai_reply;
pub mod ai_state;
pub mod analytics;
pub mod archive_inspection;
pub mod attachment_dedup;
pub mod attachment_search;
pub mod blockers;
pub mod bulk_actions;
mod command_service;
pub mod core;
pub mod delivery_notifications;
pub mod drafts;
pub(crate) mod evidence;
pub mod explain;
pub mod export;
pub mod extract;
pub mod finance;
pub mod fixtures;
pub mod flags;
pub mod folders;
pub mod import;
pub mod ingestion;
pub mod legal;
pub mod messages;
pub mod multilingual;
pub mod outbox;
pub mod personas;
pub mod ports;
pub mod read_receipts;
pub mod rich_template;
pub mod rules;
pub mod saved_search_counts;
pub mod saved_searches;
pub mod search;
pub mod service;
pub mod signatures;
pub mod sources;
pub mod spf_dkim;
pub mod storage;
pub mod subscriptions;
pub mod templates;
pub mod threads;
```

Ниже раскрыты ключевые компоненты, полностью присутствующие в контексте.

## Хранилище проекций сообщений (`messages`)

`MessageProjectionStore` работает с таблицей `communication_messages`.

### Вставка/обновление сообщения (`upsert_message`)

- Вызывает `message.validate()` (детали валидации не входят в контекст, кроме упомянутой ниже `validate_non_empty` и `validate_limit`).
- Формирует канонический `message_id` через функцию `message_id(account_id, provider_record_id)`.
- Запрос `INSERT INTO communication_messages` … `SELECT … FROM communication_raw_records` при условии `record_kind = 'email_message'`.
- Вставляет фиксированные значения:
  - `channel_kind = 'email'`
  - `conversation_id = NULL`
  - `sender_display_name = $6` (совпадает с `sender`)
  - `delivery_state = 'received'`
  - `message_metadata = '{}'::jsonb`
- Конфликт по `(account_id, provider_record_id)` разрешается `DO UPDATE SET` всех полей, кроме первичного ключа, и обновлением `projected_at = now()`.
- Если строка не вставлена (не найдена raw‑запись), возвращается `MessageProjectionError::RawRecordTupleMismatch`.
- Возвращает проекцию через `row_to_projected_message`.

### Вставка/обновление сообщения произвольного канала

Методы:

- `upsert_channel_message` – запрещает пустой `body_text` (политика `false`).
- `upsert_channel_message_allowing_empty_body_text` – разрешает пустой `body_text` (политика `true`).
- Оба делегируют в `upsert_channel_message_with_body_policy`, которая:
  - Валидирует сообщение с учётом политики пустого тела.
  - Выполняет `INSERT … SELECT … FROM communication_raw_records`, но передаёт все поля из `NewProjectedMessage`: `channel_kind`, `conversation_id`, `sender_display_name`, `delivery_state`, `message_metadata`.
  - Так же разрешает конфликт по `(account_id, provider_record_id)`.
  - Возвращает проекцию.

Возвращаемая проекция включает поля `workflow_state`, `importance_score`, `ai_category`, `ai_summary`, `ai_summary_generated_at`, `local_state`, `local_state_changed_at`, `local_state_reason`.

### Переход состояния workflow (`transition_workflow_state`)

- Публичный метод `transition_workflow_state` принимает `message_id` и `WorkflowState`.
- Внутри вызывает `transition_workflow_state_with_observation(message_id, new_state, None, "workflow_state_transition", None)`.
- `transition_workflow_state_with_observation`:
  - Начинает транзакцию.
  - Вызывает внутренний `transition_workflow_state_in_transaction`.
  - Если передан непустой `observation_id`, связывает observation с сообщением через `link_mail_entity_in_transaction`.
  - Коммитит транзакцию.
- `transition_workflow_state_in_transaction`:
  - Проверяет `message_id` на непустоту через `validate_non_empty`.
  - Выполняет `UPDATE communication_messages SET workflow_state = $2, projected_at = now() WHERE message_id = $1`.
  - Возвращает обновлённую проекцию. При отсутствии строки возвращает `MessageProjectionError::MessageNotFound`.

### Утилиты валидации (`validation`)

- `validate_non_empty(field_name, value)`: обрезает строку, при пустом результате – `MessageProjectionError::EmptyField(field_name)`.
- `validate_limit(limit)`: требует значение в диапазоне `1..=5000`, иначе – `MessageProjectionError::InvalidLimit(limit)`.

## Исходящая очередь (`outbox`)

### Статусы (`CommunicationOutboxStatus`)

Перечисление со значениями:

- `Queued` → `"queued"`
- `Scheduled` → `"scheduled"`
- `Sending` → `"sending"`
- `Sent` → `"sent"`
- `Failed` → `"failed"`
- `Canceled` → `"canceled"`

Определены методы `as_str` и `parse`, выполняющие прямое преобразование из/в строки.

### Модели

- `CommunicationOutboxItem` – полная запись очереди со всеми полями, включая recipient‑списки (`to_recipients`, `cc_recipients`, `bcc_recipients`), `provider_message_id`, `send_attempts`, `claimed_at`, `sent_at`, `metadata` (JSON) и временные метки.
- `NewCommunicationOutboxItem` – входная структура для постановки в очередь.
- Валидация `NewCommunicationOutboxItem`:
  - `outbox_id` и `account_id` обязательны и непусты.
  - Хотя бы один непустой адрес среди `to_recipients`, `cc_recipients`, `bcc_recipients`.
  - `metadata` обязана быть JSON‑объектом.

### Хранилище (`CommunicationOutboxStore`)

- `get(outbox_id)` – получает элемент очереди, при необходимости объединяя с последним `read_receipt` (через `LEFT JOIN LATERAL`).
- `enqueue(item)` – вставляет запись; вызывает `enqueue_with_observation(item, None, "outbox_status_transition", None)`.
- `enqueue_with_observation`:
  - Валидирует `item`.
  - В транзакции вызывает `ensure_canonical_account_in_transaction`.
  - Вставляет строку с параметрами; возвращает созданный элемент.
  - При наличии непустого `observation_id` связывает observation с outbox‑элементом.
- `list` – возвращает список элементов (до `limit`).
- `list_page` – пагинированный список с курсором; декодирует курсор через `decode_outbox_list_cursor`; принимает фильтры по `account_id` и `status`.

### Доставка (`outbox/delivery`)

- Трейт `OutboxEmailSender` – определяет метод `send(&self, &CommunicationOutboxItem) -> Result<OutboxSendReceipt, OutboxDeliveryError>`.
- `OutboxSendReceipt` содержит `provider_message_id` и `accepted_recipients`.
- `OutboxRetryPolicy`:
  - Параметры: `max_attempts` (>=1), `base_delay`, `max_delay`.
  - Функция `new` корректирует задержки: `base_delay` >= 1 сек, `max_delay` >= `base_delay`.
  - `disabled()` – создаёт политику с `max_attempts=1`, задержками 1 сек.
  - `next_attempt_at(now, completed_attempts)`: если `completed_attempts >= max_attempts`, возвращает `None`; иначе вычисляет задержку как `base_delay * 2^(retry_index)`, ограниченную `max_delay`.
- `EmailOutboxDeliveryWorker`:
  - Конструируется с хранилищем, отправителем и политикой повторных попыток (по умолчанию: `max_attempts=3`, `base_delay=30 sec`, `max_delay=15 min`).
  - `deliver_due(now, limit)`:
    1. Вызывает `store.claim_due(now, limit)`.
    2. Для каждого полученного элемента вызывает `sender.send`.
    3. При успехе – `store.mark_sent`.
    4. При ошибке – если `retry_policy.next_attempt_at` возвращает время, вызывает `store.mark_retry_scheduled`, иначе `store.mark_failed`.
    5. Возвращает `OutboxDeliveryReport` с количеством `claimed`, `sent`, `failed`, `retried`.

### Запись статуса доставки (`outbox/delivery_status`)

- `OutboxDeliveryStatus`: `Delivered`, `Delayed`, `Failed`.
- `NewOutboxDeliveryStatus` – входные данные: `account_id`, `provider_message_id`, `delivery_status`, `smtp_status`, `source_kind`, опциональные `provider_record_id`/`raw_record_id`, `recorded_at`.
- `record_delivery_status`:
  1. Нормализует `account_id`, `provider_message_id`, `source_kind` (проверка непустоты, обрезка).
  2. Формирует терминальную ошибку, если статус `Failed` (с текстом SMTP‑статуса или без).
  3. В транзакции обновляет `communication_outbox.metadata` полем `delivery_status` и, при необходимости, `last_error`.
  4. Вызывает `capture_delivery_status_observation` (создаёт observation типа `COMMUNICATION_DELIVERY_STATUS`), связывает observation с `outbox_item` и `provider_message`.
  5. Добавляет событие `mail.outbox.delivery_status_changed` через `EventStore::append_in_transaction`.
  6. Коммитит транзакцию.

### Отправка через провайдеров (`provider_sender`, `smtp_sender`)

- `CommunicationOutboxEmailSender` реализует `OutboxEmailSender`.
  - При отправке загружает `ProviderAccount`.
  - Если `provider_kind == EmailProviderKind::Gmail` и в конфиге `gmail_send_enabled == true`, выполняет отправку через Gmail OAuth (читает `ProviderAccountSecretPurpose::OauthToken`, вызывает `gmail_transport.send`).
  - Иначе делегирует `smtp_sender.send`.
- `SmtpOutboxEmailSender`:
  - Загружает `ProviderAccount`, строит `SmtpConfig` через `smtp_config_for_provider_account`, читает SMTP‑пароль через `ProviderCredentialReader` (секрет с целью `SmtpPassword`), выполняет `transport.send`.
- `smtp_config_for_provider_account`:
  - Для `Gmail` возвращает ошибку «Gmail send is unavailable until OAuth send scopes are configured».
  - Для `Icloud` и `Imap` читает из `account.config` ключи: `smtp_host` (обязательный), `smtp_port` (обязательный, 1–65535), `smtp_username` (если отсутствует – `account.external_account_id`), `smtp_tls` (по умолчанию `true`), `smtp_starttls` (по умолчанию `false`).
  - Для остальных провайдеров – ошибка «provider does not support SMTP send».
- `outgoing_email_from_outbox_item` – собирает `OutgoingEmail`, извлекая `in_reply_to` и `references` из `metadata`.

### ProviderSendStore

`ProviderSendStore::record_sent_with_observation` проверяет непустоту `observation_id`, `provider_message_id`, `transport`, затем в транзакции вызывает `link_mail_entity_in_transaction` с entity‑типом `provider_send`.

## Уведомления о прочтении (`read_receipts`)

- `CommunicationReadReceiptKind` – только вариант `Read`.
- `CommunicationReadReceipt` – полная запись, включая `receipt_id`, `outbox_id`, `provider_message_id`, `recipient`, `read_at`, `source_kind`, `metadata` и т.д.
- `NewCommunicationReadReceipt` – входная структура; `receipt_id` опционален (при отсутствии генерируется из `account_id` и `provider_record_id`).
- `CommunicationReadReceiptStore::record`:
  1. Нормализует входные данные (проверка обязательных полей, `metadata` должен быть объектом).
  2. В транзакции вызывает `ensure_canonical_account_in_transaction`.
  3. Коррелирует `outbox_id` по `account_id` и `provider_message_id` (запрос к `communication_outbox`).
  4. Вставляет запись в `communication_read_receipts` с `receipt_kind = 'read'`.
  5. Создаёт observation `COMMUNICATION_READ_RECEIPT`, связывая его с `read_receipt`, `outbox_item` и `provider_message`.
  6. Добавляет событие `mail.read_receipt.recorded`.
  7. Коммитит транзакцию.

## Персоны (`personas`)

- `CommunicationPersona` – содержит поля: `persona_id`, `name`, `account_id`, `display_name`, `signature`, `default_language`, `default_tone`, `is_default`, `metadata`, временные метки.
- `NewCommunicationPersona` – валидация: `persona_id`, `name`, `account_id` обязательны и непусты.
- `CommunicationPersonaStore`:
  - `upsert`: вызывает `ensure_canonical_account` (вставляет `communication_accounts` из `communication_provider_accounts`), затем выполняет `INSERT … ON CONFLICT (persona_id) DO UPDATE`.
  - `list` – возвращает все персоны, сортированные по `name`.
  - `get_default` – возвращает первую персону с `is_default = true`.

## Правила обработки (`rules`)

### Режимы (`RuleMode`)

Возможные значения:

- `suggest`
- `ask_before_execute`
- `auto_execute`
- `dry_run`

У каждого варианта определён метод `parse`, возвращающий `Option<RuleMode>` (неизвестная строка → `None`).

### Модели

- `EmailRule` – содержит `conditions_json`, `actions_json` (оба `Value`), `mode`, `enabled`, `match_count`, `last_matched_at` и пр.
- `NewEmailRule` – валидация: `rule_id` и `name` непусты, `conditions_json` и `actions_json` обязаны быть массивами.
- `RuleMatchResult` – `rule_id`, `matched`, `matched_conditions`, `suggested_actions`.
- `RuleAction` – `action_type` и `params`.

### Сопоставление (`evaluation`)

- `evaluate_conditions(conditions: &Value, message: &ProjectedMessage)`:
  - Перебирает массив условий. Каждое условие имеет поля `field`, `operator`, `value` и опционально `label`.
  - Поддерживаемые поля и операторы:
    - `subject` / `contains`, `equals`
    - `body` / `contains`
    - `sender` / `contains`, `equals`
    - `has_attachment` / `equals` (сравнивается с `"true"`)
  - Сравнение регистронезависимое (через `to_lowercase`).
  - При совпадении в результат добавляется `label` или `"condition matched"`.
- `parse_actions(actions: &Value)` – извлекает из массива действий поля `action_type` и `params`.

### Хранилище (`EmailRuleStore`)

- `upsert_rule` – вставляет или обновляет правило по `rule_id`.
- `list_rules` – возвращает все правила, сортированные по `created_at DESC`.
- `match_rules(message: &ProjectedMessage)`:
  1. Загружает все правила.
  2. Для каждого включённого правила вызывает `evaluate_conditions`.
  3. Если условия совпали, формирует `RuleMatchResult`.

## Многоязычность (`multilingual`)

### Сервис `MultilingualService`

- Опционально принимает `SharedAiRuntimePort`; если runtime отсутствует, перевод возвращает `None`.
- `detect_language(text: &str) -> LanguageDetection` – эвристическое определение языка:
  - Проверяет наличие кириллицы → различает русский (`ru`, confidence 0.90) и украинский (`uk`, confidence 0.85, если есть символы `ї`/`є`).
  - Наличие символа `ñ` → испанский (`es`, confidence 0.85).
  - Наличие CJK‑иероглифов → китайский (`zh`, confidence 0.70).
  - Подсчёт вхождений характерных слов:
    - Испанские (`hola`, `gracias`, `para`, …) → `es` (confidence 0.70).
    - Русская латиница (`privet`, `spasibo`, `pozhaluysta`) → `ru` (confidence 0.55).
    - Немецкие (`mit`, `und`, `der`, …) → `de` (confidence 0.65).
  - Если есть латиница, но не сработали другие правила → `en` (confidence 0.50).
  - Иначе → `unknown` (confidence 0.0).
- `translate(text, target_lang)` – отправляет LLM‑запрос с промптом «Translate the following text to {target_lang}. Return ONLY the translated text, no explanations:». Возвращает `Translation` с полями `original_language`, `target_language`, `translated_text`, `model`.
- Типы:
  - `LanguageDetection` – `language` (строка), `confidence` (f32), `script` (Option<String>).
  - `Translation` – `original_language`, `target_language`, `translated_text`, `model`.

## Rich‑шаблоны (`rich_template`)

- `RichTemplate` – содержит `name`, `subject` и `blocks: Vec<TemplateBlock>`.
- `TemplateBlock` – варианты:
  - `Text { content }`
  - `Variable { key, default }`
  - `Conditional { condition, then_blocks, else_blocks }`
  - `Table { headers, row_variable, columns }`
  - `Button { text, url_template }`
  - `Divider`
- `Condition` – `variable`, `operator`, `value`.
- `RichTemplateEngine::render(template, vars: &HashMap<String, String>)`:
  - Обрабатывает `subject` подстановкой `{{key}}` → значение.
  - Рекурсивно рендерит блоки:
    - `Text` – подстановка переменных.
    - `Variable` – подстановка значения или default.
    - `Conditional` – проверяет условие: операторы `equals`, `not_empty`, `contains`. При истине рендерит `then_blocks`, иначе `else_blocks`.
    - `Table` – выводит только заголовки и разделитель Markdown‑таблицы (колонки не раскрыты в доступном коде).
    - `Button` – формирует Markdown‑ссылку `[text](url)`.
    - `Divider` – `---`.

## Сохранённые поиски и подсчёт сообщений (`saved_search_counts`)

- `count_messages_for_saved_search` – строит `SELECT count(*) FROM communication_messages` с фильтрами из `SavedSearchRecord`:
  - `account_id`, `workflow_state` (если задан), `channel_kind`, `local_state` (через `persisted_filter`), а также полнотекстовый поиск через `append_message_search_filter`.
- `load_message_counts_for_saved_searches` – аналогично, но выполняет один UNION ALL‑запрос для пакета записей, возвращая `HashMap<saved_search_id, count>`.

## Порты (`ports`)

Модуль `ports` реэкспортирует следующие хранилища как порты для внедрения зависимостей:

```rust
pub use super::core::CommunicationIngestionStore as CommunicationIngestionPort;
pub use super::core::CommunicationProviderAccountStore as CommunicationProviderAccountPort;
pub use super::messages::MessageProjectionStore as CommunicationMessageProjectionPort;
pub use super::storage::CommunicationStorageStore as CommunicationBlobMetadataPort;
pub use super::storage::LocalCommunicationBlobStore as LocalCommunicationBlobPort;
```

Назначение каждого порта не раскрыто в контексте, кроме факта реэкспорта.

### Source coverage / Покрытие источников

| Исходный файл | Покрытые факты |
|---|---|
| `mod.rs` | Полный список публичных подмодулей домена `communications`. |
| `multilingual.rs` | Структура `MultilingualService`, эвристика `detect_language` (правила для русского, украинского, испанского, немецкого, английского, китайского, пороговые значения confidence), метод `translate`, типы `LanguageDetection` и `Translation`, `MultilingualError`. |
| `outbox.rs` | Перечисление `CommunicationOutboxStatus`, его `as_str`/`parse`; структуры `CommunicationOutboxItem`, `NewCommunicationOutboxItem`, `EmailOutboxListPage`; валидация нового элемента (обязательность непустых `outbox_id`, `account_id`, наличие хотя бы одного адресата, metadata-объект); хранилище `CommunicationOutboxStore` с методами `get`, `enqueue`, `enqueue_with_observation`, `list`, `list_page`. |
| `outbox/delivery.rs` | Трейт `OutboxEmailSender`, `OutboxSendReceipt`, `OutboxDeliveryError`, `OutboxRetryPolicy` (параметры, вычисление `next_attempt_at`, disabled), `OutboxDeliveryReport`, `EmailOutboxDeliveryWorker::deliver_due` (логика отправки, retry, отметки sent/failed). |
| `outbox/delivery_status.rs` | Перечисление `OutboxDeliveryStatus`; структуры `NewOutboxDeliveryStatus`, `OutboxDeliveryStatusRecord`; метод `record_delivery_status` с обновлением `communication_outbox`, созданием observation и события. |
| `outbox/provider_send_store.rs` | `ProviderSendStore::record_sent_with_observation` – валидация и вызов `link_mail_entity_in_transaction`. |
| `outbox/provider_sender.rs` | `CommunicationOutboxEmailSender` – маршрутизация между Gmail и SMTP; условия Gmail‑отправки (наличие `gmail_send_enabled` в конфиге); использование `ProviderAccountSecretPurpose::OauthToken`. |
| `outbox/smtp_sender.rs` | `SmtpOutboxEmailSender::send` (загрузка аккаунта, SMTP‑конфигурация, чтение секрета); функция `smtp_config_for_provider_account` – правила для Gmail, Icloud/Imap (обязательные `smtp_host`/`smtp_port`, опциональные `smtp_username`, `smtp_tls`, `smtp_starttls`); `outgoing_email_from_outbox_item` (сборка `OutgoingEmail`, извлечение `in_reply_to`/`references` из metadata). |
| `personas.rs` | Модели `CommunicationPersona`, `NewCommunicationPersona`; валидация (`persona_id`, `name`, `account_id` непусты); `CommunicationPersonaStore::upsert`, `list`, `get_default`; обеспечение канонического аккаунта. |
| `ports.rs` | Список реэкспортированных портов. |
| `read_receipts.rs` (первые 12000 символов) | `CommunicationReadReceiptKind` (`Read`); структуры `CommunicationReadReceipt`, `NewCommunicationReadReceipt`; `CommunicationReadReceiptStore::record` – корреляция outbox, вставка, observation, событие. |
| `rich_template.rs` | Модели `RichTemplate`, `TemplateBlock`, `Condition`; `RichTemplateEngine::render` – подстановка переменных, условные блоки с операторами `equals`, `not_empty`, `contains`, рендеринг `Button` (ссылка) и `Divider`; тесты демонстрируют поведение. |
| `rules.rs` (реэкспорт) | Перечисление экспортируемых элементов модуля `rules`. |
| `rules/errors.rs` | `EmailRuleError`. |
| `rules/evaluation.rs` | Функции `evaluate_conditions` (поля `subject`, `body`, `sender`, `has_attachment`; операторы `contains`, `equals`) и `parse_actions`. |
| `rules/mode.rs` | `RuleMode` (варианты, `as_str`, `parse`, `format_mode`). |
| `rules/models.rs` | `EmailRule`, `RuleMatchResult`, `RuleAction`, `NewEmailRule`. |
| `rules/rows.rs` | Константа `EMAIL_RULE_COLUMNS`, `row_to_email_rule` (преобразование с дефолтом `RuleMode::Suggest` для неизвестной строки). |
| `rules/store.rs` | `EmailRuleStore::upsert_rule`, `list_rules`, `match_rules` (обход включённых правил, вызов evaluate). |
| `rules/validation.rs` | Валидация `NewEmailRule`: `rule_id`/`name` непусты, `conditions_json` и `actions_json` – массивы. |
| `rules/tests.rs` | Тестовые примеры, подтверждающие поведение `evaluate_conditions` и `RuleMode::parse`. |
| `saved_search_counts.rs` | Функции `count_messages_for_saved_search` и `load_message_counts_for_saved_searches` с фильтрацией по полям `SavedSearchRecord`. |
| `messages/store/upsert.rs` | `upsert_message` (email‑сообщения с фиксированными значениями), `upsert_channel_message`, `upsert_channel_message_allowing_empty_body_text` (политика пустого тела), возвращаемые столбцы. |
| `messages/store/workflow.rs` | `transition_workflow_state`, `transition_workflow_state_with_observation` (связывание observation), `transition_workflow_state_in_transaction` (валидация `message_id`). |
| `messages/validation.rs` | `validate_non_empty` и `validate_limit` (диапазон 1..=5000). |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений кода, документации или ADR не видно, поскольку отсутствует предыдущая версия wiki‑страницы. Файлы `outbox.rs` и `read_receipts.rs` были обрезаны до 12000 символов; их полное поведение в контексте не раскрыто, что потенциально может привести к неполноте описания методов, работающих с пагинацией, курсорами и дополнительными операциями (`claim_due`, `mark_sent`, `mark_failed` и т.п.), но это не является drift‑расхождением на основании доступных данных.
