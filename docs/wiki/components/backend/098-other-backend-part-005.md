---
chunk_id: 098-other-backend-part-005
batch_id: batch-20260628T214902
group: backend
role: other
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 098-other-backend-part-005 — backend/other

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Чанк содержит миграции базы данных backend-компонента, которые связывают различные таблицы с общей моделью наблюдений (`observations`), добавляют ссылочные ограничения и регистрируют новые типы (kind) наблюдений. Текущая wiki-страница `components/backend.md` (вероятно, обзорная) должна быть дополнена разделом, описывающим эти изменения: какие сущности теперь ссылаются на `observations`, какие гарантии целостности введены и какие новые категории доказательств (`observation_kind_definitions`) добавлены. Все утверждения подтверждаются исключительно предоставленным SQL-кодом миграций.

## Предложенные страницы

**Путь:** `components/backend.md`

```markdown
# Backend

## Привязка сущностей к наблюдениям (миграции 0101–0109, 0121)

Перечисленные ниже миграции внедряют сквозное связывание таблиц предметной области с реестром наблюдений (`observations`), а также добавляют контрольные ограничения, индексы и триггерные проверки.

### `graph_evidence` (0101)
- Добавлен столбец `observation_id TEXT`.
- Выполнен обратный перенос (backfill) `observation_id` для существующих записей:
  - если `source_kind = 'message'`, значение берётся из `communication_messages.observation_id`;
  - если `source_kind = 'raw_record'` – из `communication_raw_records.observation_id`;
  - если `source_kind = 'observation'` – из `observations.observation_id`.
- Обновлён перечень допустимых `source_kind`: `'contact', 'person', 'message', 'document', 'raw_record', 'relationship', 'decision', 'obligation', 'observation'`.
- Создано внешнее ключевое ограничение `graph_evidence_observation_fk` (`observation_id → observations(observation_id)`).
- Добавлены проверочные ограничения:
  - `graph_evidence_message_observation_required`: если `source_kind = 'message'`, то `observation_id IS NOT NULL`.
  - `graph_evidence_observation_source_check`: если `source_kind = 'observation'`, то `observation_id IS NOT NULL AND observation_id = source_id`.
- Создан частичный индекс `graph_evidence_observation_idx` на `observation_id WHERE observation_id IS NOT NULL`.

### `semantic_embeddings` (0102)
- Добавлен столбец `observation_id TEXT`.
- Выполнен backfill для `source_kind = 'message'` из `communication_messages.observation_id`.
- Внешний ключ `semantic_embeddings_observation_fk` → `observations(observation_id)`.
- Ограничение `semantic_embeddings_message_observation_required`: если `source_kind = 'message'`, то `observation_id IS NOT NULL`.
- Индекс `semantic_embeddings_observation_idx` (частичный, по `observation_id`).

### `documents` (0103)
- Добавлен столбец `observation_id TEXT`.
- Для строк без наблюдения создаются новые записи в `observations` с идентификатором, сформированным по формуле:
  `'observation:v1:legacy-document:' || md5(document_id | imported_at | source_fingerprint | extracted_text)`.
- Значения полей `origin_kind = 'file_import'`, `confidence = 1.0`, `provenance` включает `legacy_backfill: true`.
- Обратная связь (UPDATE) проставляет `observation_id` документам, для которых наблюдение уже существует.
- После миграции столбец `observation_id` получает `NOT NULL`.
- Внешний ключ `documents_observation_fk` → `observations(observation_id)`.
- Ограничение `documents_source_kind_observation_check` требует `observation_id IS NOT NULL`.
- Индекс `documents_observation_idx`.

### `tasks` (0104)
- Создана функция `enforce_task_provenance_target()` как триггерная процедура на PL/pgSQL.
- Она проверяет, что `provenance_id` задачи указывает на существующую запись в соответствующей таблице в зависимости от `provenance_kind`:
  - `'observation'` → `observations(observation_id)`;
  - `'review_item'` → `review_items(review_item_id)`;
  - `'decision'` → `decisions(decision_id)`;
  - `'obligation'` → `obligations(obligation_id)`.
- Если `provenance_kind` не из перечисленных, возбуждается исключение с SQLSTATE `23514`.
- Создан BEFORE-триггер `tasks_provenance_target_guard` на INSERT и UPDATE полей `provenance_kind, provenance_id` таблицы `tasks`.

### `task_candidates` (0105)
- Backfill `observation_id` для кандидатов задач:
  - при `source_kind = 'message'` из `communication_messages.observation_id`;
  - при `source_kind = 'document'` из `documents.observation_id`.
- После переноса `source_kind` и `source_id` перезаписываются на `'observation'` и `observation_id` соответственно (для тех, у кого `observation_id IS NOT NULL` и `source_kind IN ('message','document')`).
- Обновлено ограничение `task_candidates_source_kind_check` – теперь допустимо только `'observation'`.
- Введено `task_candidates_observation_required` – требует `observation_id IS NOT NULL AND source_id = observation_id`.

### `calendar_events` (0106)
- Добавлен столбец `observation_id TEXT`.
- Для событий без наблюдения генерируются новые записи `observations` с идентификатором:
  `'observation:v1:legacy-calendar-event:' || md5(event_id | start_at | end_at | title | description | location)`.
- `origin_kind = 'local_runtime'`, `confidence = 1.0`, `provenance` содержит `legacy_backfill: true` и `ingested_by: 'calendar_events_domain'`.
- Обратный UPDATE проставляет `observation_id` уже существующим событиям.
- `observation_id` становится `NOT NULL`.
- Внешний ключ `calendar_events_observation_fk` → `observations(observation_id)`.
- Индекс `calendar_events_observation_idx`.

### Добавление колонок `source` (0107, 0108, 0109)
- `event_participants.source TEXT NOT NULL DEFAULT 'manual'` (0107).
- `calendar_reminders.source TEXT NOT NULL DEFAULT 'manual'` (0108).
- `task_subtasks.source TEXT NOT NULL DEFAULT 'manual'` (0109).
- Для каждой таблицы выполнен UPDATE, гарантирующий, что существующие строки получат значение `'manual'`, если поле пустое.

### Привязка `communication_mail_sync_run` (0121)
- Добавлены definition-записи для двух новых типов наблюдений:
  - `'COMMUNICATION_MAIL_SYNC_RUN'` (почтовый sync run);
  - `'COMMUNICATION_MAIL_SYNC_RUN_STATUS'` (статус жизненного цикла sync run).
- Вставка использует `ON CONFLICT (code, version) DO NOTHING`.

## Новые типы наблюдений (observation kinds), зарегистрированные в миграциях 0110–0125

Ниже перечислены только те definition-записи, которые добавляются в таблицу `observation_kind_definitions` указанными миграциями. Для каждой записи приведены код (`code`), категория (`category`) и краткое описание (`description`), точно воспроизведённые из SQL.

| Код (`code`) | Категория | Назначение |
|---|---|---|
| `COMMUNICATION_DRAFT` (0110) | communication | Manual or provider-backed communication draft captured as evidence. |
| `COMMUNICATION_FOLDER` (0111) | communication | Manual or provider-backed communication folder captured as evidence. |
| `COMMUNICATION_SAVED_SEARCH` (0112) | communication | Saved search or smart folder definition captured as communication evidence. |
| `COMMUNICATION_OUTBOX` (0113) | communication | Queued, scheduled, or canceled outbound communication captured as evidence. |
| `COMMUNICATION_DELIVERY_STATUS` (0114) | communication | Observed provider or parser delivery status for an outbound communication. |
| `COMMUNICATION_READ_RECEIPT` (0115) | communication | Observed read receipt for an outbound communication. |
| `CONTRADICTION_OBSERVATION` (0116) | review | Consistency engine contradiction evidence captured for review. |
| `AI_PROVIDER_ACCOUNT` (0117) | ai | AI control center provider account configuration captured as evidence. |
| `AI_PROVIDER_SECRET_BINDING` (0117) | ai | AI control center provider secret binding captured as evidence. |
| `AI_MODEL_ROUTE` (0117) | ai | AI control center capability-slot routing captured as evidence. |
| `TELEGRAM_PROVIDER_WRITE_COMMAND` (0118) | telegram | Telegram provider write command queued as durable action evidence. |
| `TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS` (0118) | telegram | Telegram provider write command lifecycle or reconciliation state captured as evidence. |
| `AI_AGENT_RUN` (0119) | ai | AI agent run request captured as durable execution evidence. |
| `AI_AGENT_RUN_STATUS` (0119) | ai | AI agent run lifecycle state captured as durable execution evidence. |
| `DOCUMENT_PROCESSING_JOB` (0120) | document | Document processing job queued as durable execution evidence. |
| `DOCUMENT_PROCESSING_JOB_STATUS` (0120) | document | Document processing job lifecycle state captured as durable execution evidence. |
| `COMMUNICATION_MAIL_SYNC_RUN` (0121) | communications | Canonical evidence for mail background sync run creation. |
| `COMMUNICATION_MAIL_SYNC_RUN_STATUS` (0121) | communications | Canonical evidence for mail background sync run lifecycle transitions. |
| `AI_PROMPT_TEMPLATE` (0122) | ai | Canonical evidence for AI prompt template lifecycle mutations. |
| `AI_PROMPT_TEMPLATE_VERSION` (0122) | ai | Canonical evidence for AI prompt template version lifecycle mutations. |
| `AI_PROMPT_EVAL_RUN` (0122) | ai | Canonical evidence for AI prompt preview and evaluation runs. |
| `AI_MODEL_CATALOG_ITEM` (0123) | ai | Canonical evidence for AI curated model catalog materialization. |
| `AI_SEMANTIC_EMBEDDING` (0124) | ai | Canonical evidence for derived semantic embedding materialization. |
| `WHATSAPP_WEB_SESSION` (0125) | communications | Canonical evidence for WhatsApp Web session lifecycle materialization. |

Все перечисленные definition-записи версионированы (`version = 1`) и вставляются с семантикой upsert:
- Миграции 0110–0120 используют `ON CONFLICT (kind_definition_id) DO UPDATE ...`;
- Миграции 0121–0125 используют `ON CONFLICT (code, version) DO NOTHING`.
```

## Покрытие источников

| Исходный файл | Факты, покрытые в предложенной странице |
|---|---|
| `backend/migrations/0101_...sql` | Добавление `observation_id` в `graph_evidence`, backfill, обновление допустимых `source_kind`, внешний ключ, проверочные ограничения, индекс. |
| `backend/migrations/0102_...sql` | Добавление `observation_id` в `semantic_embeddings`, backfill, внешний ключ, ограничение, индекс. |
| `backend/migrations/0103_...sql` | Добавление `observation_id` в `documents`, генерация новых наблюдений, backfill, `NOT NULL`, внешний ключ, ограничение, индекс. |
| `backend/migrations/0104_...sql` | Триггерная функция `enforce_task_provenance_target`, триггер `tasks_provenance_target_guard`, проверки целостности для `tasks.provenance_id`. |
| `backend/migrations/0105_...sql` | Backfill `observation_id` в `task_candidates`, перезапись `source_kind`/`source_id`, обновление ограничений. |
| `backend/migrations/0106_...sql` | Добавление `observation_id` в `calendar_events`, генерация новых наблюдений, backfill, `NOT NULL`, внешний ключ, индекс. |
| `backend/migrations/0107_...sql` | Добавление `source TEXT NOT NULL DEFAULT 'manual'` в `event_participants`, UPDATE существующих. |
| `backend/migrations/0108_...sql` | Добавление `source TEXT NOT NULL DEFAULT 'manual'` в `calendar_reminders`, UPDATE существующих. |
| `backend/migrations/0109_...sql` | Добавление `source TEXT NOT NULL DEFAULT 'manual'` в `task_subtasks`, UPDATE существующих. |
| `backend/migrations/0110_...sql` | Добавление observation kind `COMMUNICATION_DRAFT`. |
| `backend/migrations/0111_...sql` | Добавление observation kind `COMMUNICATION_FOLDER`. |
| `backend/migrations/0112_...sql` | Добавление observation kind `COMMUNICATION_SAVED_SEARCH`. |
| `backend/migrations/0113_...sql` | Добавление observation kind `COMMUNICATION_OUTBOX`. |
| `backend/migrations/0114_...sql` | Добавление observation kind `COMMUNICATION_DELIVERY_STATUS`. |
| `backend/migrations/0115_...sql` | Добавление observation kind `COMMUNICATION_READ_RECEIPT`. |
| `backend/migrations/0116_...sql` | Добавление observation kind `CONTRADICTION_OBSERVATION`. |
| `backend/migrations/0117_...sql` | Добавление observation kinds `AI_PROVIDER_ACCOUNT`, `AI_PROVIDER_SECRET_BINDING`, `AI_MODEL_ROUTE`. |
| `backend/migrations/0118_...sql` | Добавление observation kinds `TELEGRAM_PROVIDER_WRITE_COMMAND`, `TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS`. |
| `backend/migrations/0119_...sql` | Добавление observation kinds `AI_AGENT_RUN`, `AI_AGENT_RUN_STATUS`. |
| `backend/migrations/0120_...sql` | Добавление observation kinds `DOCUMENT_PROCESSING_JOB`, `DOCUMENT_PROCESSING_JOB_STATUS`. |
| `backend/migrations/0121_...sql` | Добавление observation kinds `COMMUNICATION_MAIL_SYNC_RUN`, `COMMUNICATION_MAIL_SYNC_RUN_STATUS`. |
| `backend/migrations/0122_...sql` | Добавление observation kinds `AI_PROMPT_TEMPLATE`, `AI_PROMPT_TEMPLATE_VERSION`, `AI_PROMPT_EVAL_RUN`. |
| `backend/migrations/0123_...sql` | Добавление observation kind `AI_MODEL_CATALOG_ITEM`. |
| `backend/migrations/0124_...sql` | Добавление observation kind `AI_SEMANTIC_EMBEDDING`. |
| `backend/migrations/0125_...sql` | Добавление observation kind `WHATSAPP_WEB_SESSION`. |

## Исходные файлы

- [`backend/migrations/0101_link_graph_evidence_to_observations.sql`](../../../../backend/migrations/0101_link_graph_evidence_to_observations.sql)
- [`backend/migrations/0102_link_semantic_embeddings_to_observations.sql`](../../../../backend/migrations/0102_link_semantic_embeddings_to_observations.sql)
- [`backend/migrations/0103_link_documents_to_observations.sql`](../../../../backend/migrations/0103_link_documents_to_observations.sql)
- [`backend/migrations/0104_add_task_provenance_reference_guard.sql`](../../../../backend/migrations/0104_add_task_provenance_reference_guard.sql)
- [`backend/migrations/0105_translate_task_candidates_to_observation_identity.sql`](../../../../backend/migrations/0105_translate_task_candidates_to_observation_identity.sql)
- [`backend/migrations/0106_link_calendar_events_to_observations.sql`](../../../../backend/migrations/0106_link_calendar_events_to_observations.sql)
- [`backend/migrations/0107_add_event_participant_source.sql`](../../../../backend/migrations/0107_add_event_participant_source.sql)
- [`backend/migrations/0108_add_calendar_reminder_source.sql`](../../../../backend/migrations/0108_add_calendar_reminder_source.sql)
- [`backend/migrations/0109_add_task_subtask_source.sql`](../../../../backend/migrations/0109_add_task_subtask_source.sql)
- [`backend/migrations/0110_add_communication_draft_observation_kind.sql`](../../../../backend/migrations/0110_add_communication_draft_observation_kind.sql)
- [`backend/migrations/0111_add_communication_folder_observation_kind.sql`](../../../../backend/migrations/0111_add_communication_folder_observation_kind.sql)
- [`backend/migrations/0112_add_communication_saved_search_observation_kind.sql`](../../../../backend/migrations/0112_add_communication_saved_search_observation_kind.sql)
- [`backend/migrations/0113_add_communication_outbox_observation_kind.sql`](../../../../backend/migrations/0113_add_communication_outbox_observation_kind.sql)
- [`backend/migrations/0114_add_communication_delivery_status_observation_kind.sql`](../../../../backend/migrations/0114_add_communication_delivery_status_observation_kind.sql)
- [`backend/migrations/0115_add_communication_read_receipt_observation_kind.sql`](../../../../backend/migrations/0115_add_communication_read_receipt_observation_kind.sql)
- [`backend/migrations/0116_add_contradiction_observation_kind.sql`](../../../../backend/migrations/0116_add_contradiction_observation_kind.sql)
- [`backend/migrations/0117_add_ai_control_center_observation_kinds.sql`](../../../../backend/migrations/0117_add_ai_control_center_observation_kinds.sql)
- [`backend/migrations/0118_add_telegram_command_observation_kinds.sql`](../../../../backend/migrations/0118_add_telegram_command_observation_kinds.sql)
- [`backend/migrations/0119_add_ai_agent_run_observation_kinds.sql`](../../../../backend/migrations/0119_add_ai_agent_run_observation_kinds.sql)
- [`backend/migrations/0120_add_document_processing_observation_kinds.sql`](../../../../backend/migrations/0120_add_document_processing_observation_kinds.sql)
- [`backend/migrations/0121_add_mail_sync_run_observation_kinds.sql`](../../../../backend/migrations/0121_add_mail_sync_run_observation_kinds.sql)
- [`backend/migrations/0122_add_ai_prompt_observation_kinds.sql`](../../../../backend/migrations/0122_add_ai_prompt_observation_kinds.sql)
- [`backend/migrations/0123_add_ai_model_catalog_observation_kind.sql`](../../../../backend/migrations/0123_add_ai_model_catalog_observation_kind.sql)
- [`backend/migrations/0124_add_ai_semantic_embedding_observation_kind.sql`](../../../../backend/migrations/0124_add_ai_semantic_embedding_observation_kind.sql)
- [`backend/migrations/0125_add_whatsapp_session_observation_kind.sql`](../../../../backend/migrations/0125_add_whatsapp_session_observation_kind.sql)

## Кандидаты на drift

Из предоставленного контекста (SQL-файлы миграций) расхождений между кодом, документацией и ADR не видно. Для оценки возможного drift необходимо иметь текущую схему базы данных, актуальное содержимое таблицы `observation_kind_definitions` и существующее содержимое wiki-страницы `components/backend.md`, которые в данном чанке не предоставлены.
