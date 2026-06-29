---
chunk_id: 100-other-backend-part-007
batch_id: batch-20260628T214902
group: backend
role: other
source_status: pending
source_count: 16
generated_by: code-wiki-ru
---

# 100-other-backend-part-007 — backend/other

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `16`

## Резюме

Страница `components/backend.md` должна быть обновлена, чтобы отразить элементы схемы базы данных, введённые миграциями 0151–0166. Эти миграции добавляют канонические таблицы коммуникационной подсистемы (подтверждения прочтения, правила, шаблоны, персоны, счета, юридические документы, сертификаты), сигнальный хаб, исходящую очередь событий (`event_outbox`), а также расширения для провайдеров WhatsApp, Zoom, Yandex Telemost и структуры команд провайдера.

## Предложенные страницы

### `components/backend.md`

```markdown
# Бэкенд

На этой странице описаны компоненты схемы базы данных, введённые миграциями 0151–0166.

## Коммуникационная подсистема

### Подтверждения прочтения (`communication_read_receipts` — миграция 0151)

Таблица `communication_read_receipts` хранит факт прочтения сообщения получателем.

| Атрибут | Ограничение / деталь |
|---|---|
| `receipt_id` | `TEXT PRIMARY KEY` |
| `account_id` | `TEXT NOT NULL REFERENCES communication_accounts ON DELETE CASCADE` |
| `outbox_id` | `TEXT REFERENCES communication_outbox ON DELETE SET NULL` |
| `provider_message_id` | `TEXT NOT NULL` |
| `recipient` | `TEXT NOT NULL` |
| `receipt_kind` | `TEXT NOT NULL DEFAULT 'read'`; допустимо только `'read'` |
| `read_at` | `TIMESTAMPTZ NOT NULL` |
| `source_kind` | `TEXT NOT NULL DEFAULT 'mdn'` |
| `provider_record_id` | `TEXT`; уникальный индекс `(account_id, provider_record_id)` для не‑NULL |
| `raw_record_id` | `TEXT REFERENCES communication_raw_records ON DELETE SET NULL` |
| `metadata` | `JSONB NOT NULL DEFAULT '{}'`; constraint `jsonb_typeof(metadata) = 'object'` |
| `created_at` | `TIMESTAMPTZ NOT NULL DEFAULT now()` |

Дополнительные индексы — `outbox_id` + `read_at` и `account_id` + `provider_message_id` + `read_at`.

Первоначальное заполнение — миграция данных из `mail_read_receipts` с добавлением `source_table: 'mail_read_receipts'` в `metadata`. Конфликты по `receipt_id` игнорируются (`DO NOTHING`).

### Канонические вспомогательные таблицы (миграция 0152)

Миграция создаёт следующие таблицы и переносит данные из старых префиксов (`email_*`):

#### `communication_rules` (правила)

- `rule_id` (`TEXT PRIMARY KEY`), `name`, `description_nl`, `conditions_json` (массив `JSONB`, по умолчанию `'[]'`), `actions_json` (массив `JSONB`, по умолчанию `'[]'`), `mode` (`'suggest'|'ask_before_execute'|'auto_execute'|'dry_run'`, по умолчанию `'suggest'`), `enabled`, `match_count`, `last_matched_at`, `created_at`, `updated_at`.
- Данные перенесены из `email_rules`.

#### `communication_templates` (шаблоны)

- `template_id` (`TEXT PRIMARY KEY`), `name`, `subject_template`, `body_template`, `variables` (массив `JSONB`), `language`, `created_at`, `updated_at`.
- Данные перенесены из `email_templates`.

#### `communication_personas` (персоны)

- `persona_id` (`TEXT PRIMARY KEY`), `name`, `account_id` (`REFERENCES communication_accounts ON DELETE CASCADE`), `display_name`, `signature`, `default_language`, `default_tone`, `is_default` (`BOOLEAN DEFAULT false`), `metadata` (`JSONB`), `created_at`, `updated_at`.
- Частичный уникальный индекс `(account_id) WHERE is_default = true`.
- Данные перенесены из `email_personas` с добавлением `source_table: 'email_personas'` в `metadata`.

#### `communication_invoices` (счета)

- `invoice_id` (`TEXT PRIMARY KEY`), `message_id`, `amount`, `currency`, `invoice_number`, `issue_date`, `due_date`, `counterparty`, `tax_id`, `status` (`'received'|'recognized'|'needs_review'|'approved'|'paid'|'closed'|'rejected'`, по умолчанию `'received'`), `linked_project_id`, `linked_person_id`, `metadata`, `created_at`, `updated_at`.
- Индексы: `(status, due_date)`, `(linked_person_id)`.
- Данные перенесены из `email_invoices` с заменой `linked_contact_id` на `linked_person_id` и добавлением `source_table: 'email_invoices'`.

#### `communication_legal_documents` (юридические документы)

- `document_id` (`TEXT PRIMARY KEY`), `message_id`, `document_type` (`'contract'|'nda'|'msa'|'dpa'|'agreement'|'legal_notice'|'claim'|'court_document'|'tax_notice'|'government_doc'|'other'`), `title`, `parties` (массив `JSONB`), `effective_date`, `expiry_date`, `amount`, `currency`, `status` (`'active'|'expired'|'pending_review'|'signed'|'terminated'|'draft'`), `linked_project_id`, `risks` (массив `JSONB`), `metadata`, `created_at`, `updated_at`.
- Данные перенесены из `email_legal_documents` с `source_table: 'email_legal_documents'`.

#### `communication_certificates` (сертификаты)

- `cert_id` (`TEXT PRIMARY KEY`), `owner_name`, `issuer`, `serial_number`, `fingerprint_sha256`, `valid_from`, `valid_until`, `cert_type` (`'smime'|'pgp'|'pdf_sign'|'cades'|'xades'|'gost_sign'|'unknown'`), `provider` (`'fnmt'|'dnie'|'cryptopro'|'gost'|'apple_keychain'|'pkcs12'|'yubikey'|'usb_token'|'other'`), `storage_kind` (`'os_keychain'|'encrypted_vault'|'pkcs12_file'|'pfx_file'|'smart_card'|'usb_token'|'external_vault'`), `storage_ref`, `trust_status` (`'trusted'|'untrusted'|'expired'|'revoked'|'pending_verification'|'self_signed'`), `is_revoked`, `usage` (массив `JSONB`), `linked_message_id`, `metadata`, `created_at`, `updated_at`.
- Индекс `(valid_until) WHERE valid_until IS NOT NULL AND is_revoked = false`.
- Данные перенесены из `email_certificates` с `source_table: 'email_certificates'`.

Также миграция переносит аккаунты из `communication_provider_accounts` в `communication_accounts`, если для аккаунта существует связанная персона в `email_personas`.

### Изменения `communication_messages` (миграции 0161, 0163)

- **`delivery_state`**: допустимые значения расширены до `('received', 'sent', 'delivered', 'read', 'played', 'send_dry_run', 'send_blocked')`.
- **`channel_kind`**: допустимые значения расширены до `('email', 'telegram_user', 'telegram_bot', 'whatsapp_web', 'whatsapp_business_cloud')`.

### Изменения `communication_provider_commands` (миграция 0162)

Таблица дополнена колонками:

- `provider_state` (`JSONB NOT NULL DEFAULT '{}'`)
- `reconciliation_status` (`TEXT NOT NULL DEFAULT 'not_observed'`)
- `next_attempt_at` (`TIMESTAMPTZ`)
- `last_attempt_at` (`TIMESTAMPTZ`)
- `provider_observed_at` (`TIMESTAMPTZ`)
- `reconciled_at` (`TIMESTAMPTZ`)
- `dead_lettered_at` (`TIMESTAMPTZ`)

Наложены ограничения: `jsonb_typeof(provider_state) = 'object'` и `reconciliation_status` не пустое.

## Сигнальный хаб (миграция 0154)

Миграция создаёт таблицы, образующие ядро сигнального хаба:

### `signal_sources` (источники сигналов)

- `id` (`UUID PRIMARY KEY`), `code` (`TEXT NOT NULL UNIQUE`), `display_name`, `category`, `source_kind`, `default_enabled`, `supports_connections`, `supports_runtime`, `supports_replay`, `supports_pause`, `supports_mute`, `capability_schema_version` (целое, >0), `created_at`, `updated_at`.
- Индекс: `(category, code)`.

### `signal_connections` (подключения)

- `id` (`UUID PRIMARY KEY`), `source_code` (`REFERENCES signal_sources`), `display_name`, `status`, `profile`, `settings` (`JSONB`), `secret_ref`, `connected_at`, `last_seen_at`, `last_signal_at`, `last_sync_at`, `created_at`, `updated_at`.
- Индекс: `(source_code, status)`.

### `signal_capabilities` (возможности)

- `id` (`UUID PRIMARY KEY`), `source_code`, `connection_id`, `capability`, `state`, `reason`, `requires_confirmation`, `action_class`, `updated_at`.
- Уникальный индекс `(source_code, COALESCE(connection_id, '00000000-0000-0000-0000-000000000000'), capability)`.

### `signal_runtime_states` (состояния времени выполнения)

- `id` (`UUID PRIMARY KEY`), `source_code`, `connection_id`, `runtime_kind`, `state`, `last_started_at`, `last_stopped_at`, `last_heartbeat_at`, `last_error_at`, `last_error_code`, `last_error_message_redacted`, `metadata` (`JSONB`), `updated_at`.
- Индекс: `(source_code, state)`.

### `signal_health` (здоровье сигналов)

- `id` (`UUID PRIMARY KEY`), `source_code`, `connection_id`, `level`, `summary`, `last_ok_at`, `last_failure_at`, `failure_count` (≥0), `consecutive_failure_count` (≥0), `next_retry_at`, `evidence` (`JSONB`), `updated_at`.
- Индекс: `(source_code, level)`.

### `signal_policies` (политики)

- `id` (`UUID PRIMARY KEY`), `scope`, `source_code`, `connection_id`, `event_pattern`, `mode`, `reason`, `created_by`, `created_at`, `expires_at`, `metadata` (`JSONB`).
- Индекс: `(scope, source_code, connection_id, event_pattern, mode, expires_at)`.

### `signal_profiles` (профили)

- `id` (`UUID PRIMARY KEY`), `code` (`UNIQUE`), `display_name`, `description`, `source_policies` (массив `JSONB`), `is_system`, `created_at`, `updated_at`.
- Индекс: `(is_system, code)`.

### `signal_paused_events` (приостановленные события)

- `id` (`UUID PRIMARY KEY`), `event_id` (`UNIQUE`), `source_code`, `connection_id`, `raw_event_type`, `event_envelope` (`JSONB`), `reason`, `paused_at`, `released_at`.
- Индекс: `(source_code, paused_at) WHERE released_at IS NULL`.

### `signal_replay_requests` (запросы на повтор)

- `id` (`UUID PRIMARY KEY`), `source_code`, `connection_id`, `event_pattern`, `status`, `requested_by`, `requested_at`, `started_at`, `completed_at`, `last_error_redacted`, `replayed_count` (≥0), `metadata` (`JSONB`).
- Индекс: `(status, requested_at)`.

## Система событий (миграции 0155, 0156)

### `event_outbox` (исходящая очередь событий)

- `event_id` (`TEXT PRIMARY KEY REFERENCES event_log ON DELETE RESTRICT`), `subject` (не пустое), `status` (по умолчанию `'pending'`), `attempts` (≥0), `next_attempt_at`, `last_error_redacted`, `published_at`, `created_at`, `updated_at`.
- Индекс для pending: `(next_attempt_at, created_at) WHERE status = 'pending'`.

### Индексы `event_log`

Миграция 0155 добавляет:

- `event_log_source_code_idx` — `(source ->> 'source_code', occurred_at, position)` (частичный, где `source ? 'source_code'`).
- `event_log_subject_identity_idx` — `(subject ->> 'kind', subject ->> 'entity_id', occurred_at, position)` (частичный, где `subject ? 'kind'`).
- `event_log_source_gin_idx` — GIN по `source`.
- `event_log_subject_gin_idx` — GIN по `subject`.

Миграция 0156 добавляет еще два частичных индекса:

- `event_log_trace_position_idx` — `(correlation_id, position)` где `correlation_id IS NOT NULL`.
- `event_log_causation_id_position_idx` — `(causation_id, position)` где `causation_id IS NOT NULL`.

## Интеграции с провайдерами

### WhatsApp (миграции 0157, 0158, 0159)

#### `whatsapp_provider_write_commands` (миграция 0157)

Таблица реализует фундамент команд записи для WhatsApp. Согласно комментарию, **ADR‑0101** предписывает: «WhatsApp provider writes must be durable, capability-gated and completed only after provider-observed reconciliation».

Основные колонки:

- `command_id` (`TEXT PRIMARY KEY`)
- `account_id` (`REFERENCES communication_provider_accounts`)
- `command_kind` — одно из `('download_media', 'send_text', 'send_media', 'send_voice_note', 'reply', 'forward', 'edit', 'delete', 'react', 'unreact', 'mark_read', 'mark_unread', 'archive', 'unarchive', 'mute', 'unmute', 'pin', 'unpin', 'join_group', 'leave_group', 'publish_status')`
- `idempotency_key` (`TEXT NOT NULL`, уникальный constraint `(account_id, idempotency_key)`)
- `provider_chat_id`, `provider_message_id`
- `target_ref` (`JSONB`), `payload` (`JSONB`), `result_payload` (`JSONB`), `audit_metadata` (`JSONB`), `provider_state` (`JSONB`)
- `capability_state` (`'available'|'blocked'|'degraded'|'unsupported'`)
- `action_class` (`'read'|'local_write'|'provider_write'|'destructive'|'export'|'secret_access'|'automation'`)
- `confirmation_decision` (`'pending'|'confirmed'|'rejected'|'not_required'`)
- `status` (`'queued'|'confirmed'|'executing'|'retrying'|'completed'|'failed'|'dead_letter'|'cancelled'`)
- `retry_count` (≥0), `max_retries` (>0), `last_error`, `actor_id`
- `happened_at`, `next_attempt_at`, `last_attempt_at`, `locked_at`, `locked_by`, `provider_observed_at`, `reconciliation_status` (`'not_observed'|'awaiting_provider'|'observed'|'mismatch'|'not_required'`), `reconciled_at`, `dead_lettered_at`, `completed_at`, `created_at`, `updated_at`

Индексы: по аккаунту, чату, идемпотентности, просроченным задачам (`due_idx`) и статусу сверки.

Также миграция переносит WhatsApp‑аккаунты из `communication_provider_accounts` в `communication_accounts` (с `source_table: 'communication_provider_accounts'`) и команды в `communication_provider_commands`.

#### Расширение `whatsapp_web_sessions` (миграция 0158)

Допустимые значения `link_state` расширены с `'fixture', 'qr_pending', 'linked', 'degraded', 'revoked', 'blocked'` до добавления `'pair_code_pending'`.

#### WhatsApp Business Cloud (миграция 0159)

- `communication_provider_accounts.provider_kind` расширен значением `'whatsapp_business_cloud'`.
- `whatsapp_web_sessions.companion_runtime` теперь допускает `'api_credentials'` вместе с `'fixture'`, `'manual_webview'`, `'blocked'`.
- В `communication_provider_account_secret_refs.secret_purpose` добавлен `'whatsapp_business_cloud_access_token'`.

### Zoom (миграция 0160)

- `provider_kind` расширен значениями `'zoom_user'` и `'zoom_server_to_server'`.
- Добавлены `secret_purpose`: `'zoom_oauth_token'`, `'zoom_client_secret'`, `'zoom_webhook_secret'`.
- Создан индекс `telegram_calls_zoom_provider_idx` по `(account_id, provider_call_id, created_at DESC)` с фильтром `metadata->>'provider' = 'zoom'`.

### Yandex Telemost (миграция 0165)

- Добавлен `provider_kind` `'yandex_telemost_user'`.
- Добавлен `secret_purpose` `'yandex_telemost_oauth_token'`.

## Прочие изменения

### `semantic_embeddings.source_kind` (миграция 0153)

Предыдущее ограничение `source_kind` было удалено и заменено новым: допустимые значения `('message', 'document', 'project', 'task', 'contact', 'person')`.

### `event_relations.entity_type` (миграция 0164)

Расширен набор: к существующим значениям добавлены `'decision'`, `'obligation'`, `'recording'`, `'call'`.

### `observation_kind_definitions` (миграция 0166)

Добавлена новая запись (или обновлена существующая) с кодом `REALTIME_CONVERSATION_RADAR_SIGNAL`:

- `kind_definition_id`: `'observation_kind:v1:realtime_conversation_radar_signal'`
- `name`: «Realtime conversation radar signal»
- `version`: 1
- `category`: `'meeting'`
- `description`: «Provider‑neutral realtime conversation radar candidate captured from a local or provider runtime before owner review and promotion.»
```

## Покрытие источников

| Source file | Covered facts |
|---|---|
| `0151_create_communication_read_receipts.sql` | Структура таблицы `communication_read_receipts`, её ограничения, индексы, миграция данных из `mail_read_receipts` |
| `0152_create_canonical_communication_aux_tables.sql` | Структура таблиц `communication_rules`, `communication_templates`, `communication_personas`, `communication_invoices`, `communication_legal_documents`, `communication_certificates`; миграция данных из `email_*` и `communication_provider_accounts` |
| `0153_allow_person_semantic_sources.sql` | Изменение `semantic_embeddings.source_kind` — разрешён `'person'` |
| `0154_create_signal_hub.sql` | Структура таблиц `signal_sources`, `signal_connections`, `signal_capabilities`, `signal_runtime_states`, `signal_health`, `signal_policies`, `signal_profiles`, `signal_paused_events`, `signal_replay_requests` |
| `0155_create_event_outbox.sql` | Структура `event_outbox`, новые индексы на `event_log` (source_code, subject, GIN) |
| `0156_add_event_trace_indexes.sql` | Индексы `event_log_trace_position_idx` и `event_log_causation_id_position_idx` |
| `0157_create_whatsapp_provider_write_commands.sql` | `whatsapp_provider_write_commands`, ADR‑0101, миграция в `communication_accounts` и `communication_provider_commands` |
| `0158_extend_whatsapp_session_link_state_for_pair_code.sql` | Добавление `'pair_code_pending'` в `link_state` |
| `0159_add_whatsapp_business_cloud_provider_kind.sql` | Добавление `'whatsapp_business_cloud'` в `provider_kind`, `'api_credentials'` в `companion_runtime`, `'whatsapp_business_cloud_access_token'` в `secret_purpose` |
| `0160_add_zoom_provider_kind.sql` | Добавление `'zoom_user'`, `'zoom_server_to_server'` в `provider_kind`, соответствующих `secret_purpose`, индекс `telegram_calls_zoom_provider_idx` |
| `0161_expand_communication_delivery_state.sql` | Расширение `delivery_state` значениями `'sent'`, `'delivered'`, `'read'`, `'played'`, `'send_dry_run'`, `'send_blocked'` |
| `0162_extend_canonical_provider_commands_runtime_state.sql` | Добавление колонок в `communication_provider_commands`: `provider_state`, `reconciliation_status`, `next_attempt_at`, `last_attempt_at`, `provider_observed_at`, `reconciled_at`, `dead_lettered_at` |
| `0163_expand_communication_message_channel_kind.sql` | Расширение `channel_kind` значениями `'whatsapp_web'`, `'whatsapp_business_cloud'` |
| `0164_expand_calendar_event_relation_entity_type.sql` | Расширение `entity_type` в `event_relations` значениями `'decision'`, `'obligation'`, `'recording'`, `'call'` |
| `0165_add_yandex_telemost_provider_kind.sql` | Добавление `'yandex_telemost_user'` в `provider_kind` и `'yandex_telemost_oauth_token'` в `secret_purpose` |
| `0166_add_realtime_conversation_radar_signal_observation_kind.sql` | Добавление observation‑kind `REALTIME_CONVERSATION_RADAR_SIGNAL` с категорией `'meeting'` |

## Исходные файлы

- [`backend/migrations/0151_create_communication_read_receipts.sql`](../../../../backend/migrations/0151_create_communication_read_receipts.sql)
- [`backend/migrations/0152_create_canonical_communication_aux_tables.sql`](../../../../backend/migrations/0152_create_canonical_communication_aux_tables.sql)
- [`backend/migrations/0153_allow_person_semantic_sources.sql`](../../../../backend/migrations/0153_allow_person_semantic_sources.sql)
- [`backend/migrations/0154_create_signal_hub.sql`](../../../../backend/migrations/0154_create_signal_hub.sql)
- [`backend/migrations/0155_create_event_outbox.sql`](../../../../backend/migrations/0155_create_event_outbox.sql)
- [`backend/migrations/0156_add_event_trace_indexes.sql`](../../../../backend/migrations/0156_add_event_trace_indexes.sql)
- [`backend/migrations/0157_create_whatsapp_provider_write_commands.sql`](../../../../backend/migrations/0157_create_whatsapp_provider_write_commands.sql)
- [`backend/migrations/0158_extend_whatsapp_session_link_state_for_pair_code.sql`](../../../../backend/migrations/0158_extend_whatsapp_session_link_state_for_pair_code.sql)
- [`backend/migrations/0159_add_whatsapp_business_cloud_provider_kind.sql`](../../../../backend/migrations/0159_add_whatsapp_business_cloud_provider_kind.sql)
- [`backend/migrations/0160_add_zoom_provider_kind.sql`](../../../../backend/migrations/0160_add_zoom_provider_kind.sql)
- [`backend/migrations/0161_expand_communication_delivery_state.sql`](../../../../backend/migrations/0161_expand_communication_delivery_state.sql)
- [`backend/migrations/0162_extend_canonical_provider_commands_runtime_state.sql`](../../../../backend/migrations/0162_extend_canonical_provider_commands_runtime_state.sql)
- [`backend/migrations/0163_expand_communication_message_channel_kind.sql`](../../../../backend/migrations/0163_expand_communication_message_channel_kind.sql)
- [`backend/migrations/0164_expand_calendar_event_relation_entity_type.sql`](../../../../backend/migrations/0164_expand_calendar_event_relation_entity_type.sql)
- [`backend/migrations/0165_add_yandex_telemost_provider_kind.sql`](../../../../backend/migrations/0165_add_yandex_telemost_provider_kind.sql)
- [`backend/migrations/0166_add_realtime_conversation_radar_signal_observation_kind.sql`](../../../../backend/migrations/0166_add_realtime_conversation_radar_signal_observation_kind.sql)

## Кандидаты на drift

Из предоставленного контекста дрейф не виден. Все приведённые миграции являются аддитивными изменениями схемы; противоречий между ними или с уже задокументированной архитектурой в рамках данного чанка не обнаружено.
