## Summary

Обновить страницу `components/backend.md` русской Obsidian‑wiki — добавить описание домена **Persons** на основе предоставленных исходных файлов. Документируется структура модуля, ключевые сущности, поведение хранилищ, ручной командный сервис, обогащение, экспертиза, экспорт, здоровье и подсистема валидации/идентичности. Все утверждения опираются только на встроенный исходный код.

## Proposed pages

### `components/backend.md`

```markdown
# Компоненты бэкенда

## Домен Persons (Лица)

Домен Persons управляет информацией о людях, их идентичностях, ролях, экспертизе, состоянии здоровья и обогащении профиля.

### Структура модуля

- **`core`** — ядро: идентичности, роли, интерактивные контексты (персоны), предпочтения, связь с observation.
- **`command_service`** — сервис ручных мутаций, оборачивающий каждую операцию в observation.
- **`enrichment`** и **`enrichment_engine`** — обогащение профиля на основе коммуникаций, управление результатами обогащения.
- **`expertise`** — экспертиза персоны.
- **`export`** — экспорт досье в Markdown / JSON / PDF.
- **`health`** — мониторинг состояния (health_status, watchlist).
- **`identity`** — работа с кандидатами идентичности и подтверждением пользователем.
- **`validation`** — нормализация email, имён, идентификаторов AI‑агентов.

### Core

#### Идентичности (PersonIdentity)

`PersonIdentity` хранит:
- идентификатор, привязку к персоне (`person_id`),
- `identity_type`, `identity_value`,
- `source`, `confidence`, `last_verified_at`, `status`, `metadata`,
- `created_at`, `updated_at`.

**`PersonsIdentityStore`** поддерживает:
- `list_by_person` / `list_unattached` (лимит 1–200),
- `upsert` / `upsert_with_observation` — вставка с `ON CONFLICT (identity_type, identity_value) WHERE status = 'active'`,
- `create_unattached` / `create_unattached_with_metadata` / `create_unattached_with_observation` — создание непривязанной записи,
- `attach_to_persona` / `attach_to_persona_with_observation` — привязка к персоне через обновление `person_id` + `status = 'active'`,
- `delete` / `delete_with_observation` — удаление с привязкой observation.

При операциях с observation вызывается `link_persons_entity` (или транзакционный вариант) из модуля `core::evidence`, которая регистрирует связь с observation для домена `"persons"`.

#### Персоны (PersonPersona / NewPersonPersona)

Интерактивный контекст общения описывается полями:
- `persona_id`, `person_id`, `name`,
- `context`, `default_tone`, `default_language`, `preferred_channel`.

**`PersonPersonaStore`**:
- `list_by_person` — выборка из `person_personas`.
- `upsert` / `upsert_with_source` / `upsert_with_observation` — вставка с `ON CONFLICT (persona_id) DO UPDATE` в рамках транзакции.
  При upsert дополнительно вызывается `materialize_interaction_preferences_in_transaction` (модуль `core::preferences`), которая пишет/удаляет записи в `person_preferences` с ключами вида `interaction_context:<persona_id>:<field>` (поля: `name`, `context`, `default_tone`, `default_language`, `preferred_channel`).
- `delete` / `delete_with_source` / `delete_with_observation` — удаление с селектом `FOR UPDATE` и вызовом `delete_interaction_preferences_in_transaction` при успешном удалении.

#### Роли (PersonRole)

`PersonRole` содержит `id`, `person_id`, `role`, `assigned_by`, `assigned_at`.

**`PersonRoleStore`**:
- `assign` / `assign_with_observation` — `INSERT … ON CONFLICT (person_id, role) DO UPDATE SET assigned_by`.
  После назначения вызывается `append_role_assigned_event` — записывает событие `person.role.assigned`. Дублирующееся событие (по unique violation) игнорируется.
- `remove` / `remove_with_observation` — `DELETE … WHERE person_id = $1 AND role = $2` с предварительным `SELECT … FOR UPDATE`.
  При успешном удалении вызывается `append_role_removed_event` (тип `person.role.removed`).

Функция `person_role_knowledge_id` преобразует строку роли в snake_case‑идентификатор (`person_role:<slug>`). При пустом результате возвращается `"person_role:unspecified"`.

#### Связь с наблюдениями

`link_persons_entity` / `link_persons_entity_in_transaction` — обёртки над платформенным `link_domain_entity` для домена `"persons"`. Позволяют связать observation с сущностью (identity, persona, role и т.п.) и передать дополнительные метаданные.

### Command Service

**`PersonCommandService`** — слой ручных мутаций. Каждая операция начинается с создания observation через `capture_manual_at` (тип `"PERSON_RECORD_MUTATION"` или `"PERSON_MEMORY_CARD"`), после чего вызывается соответствующий store‑метод.

Обнаруженные в срезе методы:
- `create_identity_trace_manual` / `assign_identity_trace_manual` — создание непривязанной идентичности и её последующая привязка.
- `upsert_person_identity_manual` / `delete_person_identity_manual` — ручная вставка и удаление идентичности.
- `assign_role_manual` / `remove_role_manual` — назначение и удаление роли.
- `upsert_person_persona_manual` / `delete_person_persona_manual` — upsert и удаление персоны.
- `upsert_person_fact_manual` — запись факта с confidence.
- `upsert_person_memory_card_manual` — запись карточки памяти.

Детали остальных методов не подтверждены из‑за обрезки исходного файла.

### Enrichment

**`PersonEnrichmentStore`** обогащает профиль персоны:
- `enrich_person` / `enrich_person_with_observation` — обновляет поля `persons` (language, tone, trust_score, avg_response_hours, writing_style) значениями из `CommunicationFingerprint`. При изменении `trust_score` генерируется событие `person.enrichment.trust_score_changed`.
- `toggle_favorite` / `toggle_favorite_with_observation` — переключает `is_favorite` и синхронизирует предпочтение `ui:favorite` через `EnrichmentEngine::persona_favorite_preference`.
- `set_notes` / `set_notes_with_observation` — записывает заметки и синхронизирует `person_memory_cards` через `MemoryEngine::persona_notes_memory_card`.

**Модель `EnrichedPerson`** включает все основные поля персоны, а также:
- `frequent_topics`, `writing_style`,
- `linked_projects`, `linked_documents` (в текущем маппере заполняются пустыми векторами),
- `person_metadata`, `is_favorite`, `notes`.

Запросы:
- `get_enriched` — по `person_id`,
- `list_enriched` — с опциональным фильтром избранного, сортировка по trust_score / interaction_count,
- `search_persons` — поиск по `lower(display_name)` или `lower(email_address)` с LIKE.

### Enrichment Engine

**`EnrichmentResultStore`** управляет результатами обогащения:
- `upsert` — принимает `person_id`, `source`, `data`, `confidence`. Из `data` извлекает `extracted_claim` (проверяет ключи `"extracted_claim"`, `"claim"`, `"value"`) и создаёт кандидата через `EnrichmentEngine::persona_observation_candidate`. Вставка — `ON CONFLICT DO NOTHING`.
- `apply` / `apply_with_observation` — переводит статус в `"applied"`, материализует transition‑link с observation.
- `reject` / `reject_with_observation` — переводит статус в `"rejected"`, также материализует transition‑link.

Ошибки: `EnrichmentEngineError` (Sqlx, Shared, Observation, NotFound).

### Expertise

**`PersonExpertiseStore`** хранит навыки персоны (`PersonExpertise`):
- `list` — выборка по `person_id`, сортировка по `confidence DESC`.
- `search_by_skill` — поиск по `lower(skill) LIKE $1`, лимит 1–100.
- `upsert` — `INSERT … ON CONFLICT DO NOTHING`.

Ошибки: `PersonExpertiseError` (Sqlx).

### Export

**`PersonExportService`** экспортирует досье:
- `export` — собирает `PersonDossier` через `PersonInvestigator.assemble_dossier` и рендерит в выбранный формат.
- `ExportFormat` — `Markdown`, `Json`, `Pdf`; парсинг из строк `"markdown"`, `"md"`, `"json"`, `"pdf"`. Для `Pdf` пока возвращается Markdown.
- Markdown‑рендеринг включает: display_name, email, tone, language, trust_score, interaction_count, topics, memory cards, facts, timeline, notes, summary.

Ошибки: `ExportError` (Investigator, Sqlx, Serde, UnsupportedFormat).

### Health

**`PersonHealthStore`** предоставляет мониторинг:
- `get` — единичная запись с подзапросами количества `pending` обещаний (`person_promises`) и неразрешённых рисков (`person_risks`).
- `list_health` — персоны с `health_status != 'healthy'`.
- `list_watchlist` — персоны на наблюдении.
- `toggle_watchlist` / `toggle_watchlist_with_observation` — переключает `watchlist` и синхронизирует предпочтение `ui:watchlist` (вставка/удаление в `person_preferences`).

Поле `health_status` по умолчанию `"healthy"`. Ошибки: `PersonHealthError` (Sqlx, Observation).

### Identity

Подсистема identity управляет кандидатами идентичности и пользовательским подтверждением.

Основные константы (файл `identity/constants.rs`):
- `PERSON_IDENTITY_REVIEW_EVENT_TYPE = "person_identity.review_state_changed"`
- Префиксы: `PERSON_IDENTITY_REVIEW_PREFIX`, `PERSON_IDENTITY_ID_PREFIX`
- Лимиты: `DEFAULT_LIMIT = 50`, `MAX_LIMIT = 100`, `MIN_LIMIT = 1`

Ошибки: `PersonIdentityError` — варианты для невалидного лимита, пустых полей, неверного candidate kind, review state, отсутствующей записи, проблем payload и событий, а также прозрачные для Sqlx, EventStore, Observation.

События:
- `ReviewCommandEvent` формирует на основе команды событие `person_identity.review_state_changed` с полями `identity_candidate_id`, `review_state` и `actor_id`.
- `ReviewEvent` извлекает те же поля из payload события.

Полный перечень экспортируемых сущностей модуля `identity` не приводится, так как часть файлов не встроена.

### Validation

Функции нормализации и валидации:
- `normalize_email_address` — обрезает пробелы, приводит к нижнему регистру, требует наличие `@`; при пустом результате — `EmptyEmailAddress`, без `@` — `InvalidEmailAddress`.
- `email_addr_spec` — извлекает адрес из формата `"Name <email>"`, либо удаляет окружающие кавычки.
- `person_id_for_email` — детерминированный идентификатор `person:v1:email:<len>:<email>`.
- `normalize_ai_agent_id` — обрезает, приводит к верхнему регистру, разрешены только ASCII‑буквы, цифры, `_`, `-`; иначе `InvalidAiAgentId`. При пустом — `EmptyAiAgentId`.
- `validate_display_name` — непустая строка после trim; иначе `EmptyDisplayName`.
- `ai_agent_person_id` — `persona:v1:ai_agent:<agent_id>`.
- `ai_agent_email_address` — `"<agent_id_lowercase>@sh-inc.ru"`.
```

## Source coverage

| Source file | Covered facts |
|---|---|
| `backend/src/domains/persons/api/validation.rs` | Normalization rules for email, agent id, display name; deterministic ID generation; AI agent ID/email patterns |
| `backend/src/domains/persons/command_service.rs` (truncated) | Structure of `PersonCommandService`, manual mutation methods (`create_identity_trace_manual`, `assign_identity_trace_manual`, `upsert_person_identity_manual`, `delete_person_identity_manual`, `assign_role_manual`, `remove_role_manual`, `upsert_person_persona_manual`, `delete_person_persona_manual`, `upsert_person_fact_manual`, `upsert_person_memory_card_manual`), observation recording pattern |
| `backend/src/domains/persons/core.rs` | Re-exported public items from core submodules |
| `backend/src/domains/persons/core/errors.rs` | `PersonCoreError` variants (Sqlx, Observation, Event, IdentityNotFound, PersonaNotFound) |
| `backend/src/domains/persons/core/evidence.rs` | `link_persons_entity` and `link_persons_entity_in_transaction` wrappers around platform observation linking for domain `"persons"` |
| `backend/src/domains/persons/core/identities.rs` | `PersonIdentity` struct, `PersonsIdentityStore` methods (list, upsert, unattached creation, attach, delete) and their SQL behaviour, observation linking |
| `backend/src/domains/persons/core/interaction_contexts.rs` | `PersonPersona` / `NewPersonPersona`, `PersonPersonaStore` upsert/delete with preference materialization, transaction management |
| `backend/src/domains/persons/core/preferences.rs` | `materialize_interaction_preferences_in_transaction` and `delete_interaction_preferences_in_transaction`, preference type naming (`interaction_context:<persona_id>:<field>`), upsert/delete logic |
| `backend/src/domains/persons/core/roles.rs` | `PersonRole` struct, `PersonRoleStore` assign/remove with events, `person_role_knowledge_id` slug generation, `append_role_assigned_event` / `append_role_removed_event` (unique violation handling for role assigned) |
| `backend/src/domains/persons/enrichment.rs` | Public API re-exports for enrichment submodule |
| `backend/src/domains/persons/enrichment/commands.rs` | `enrich_person`, `enrich_person_with_observation` (fingerprint‑based enrichment, trust_score event), `toggle_favorite`/`set_notes` variants with preference/memory card sync, `append_trust_score_changed_event` |
| `backend/src/domains/persons/enrichment/errors.rs` | `PersonEnrichmentError` variants (Sqlx, Trust, Observation, Event, NotFound) |
| `backend/src/domains/persons/enrichment/materialization.rs` | `sync_notes_memory_card_in_transaction` (deletes + inserts via `MemoryEngine`), `sync_favorite_preference_in_transaction` (upserts/deletes `ui:favorite` via `EnrichmentEngine`) |
| `backend/src/domains/persons/enrichment/models.rs` | `EnrichedPerson` struct with all fields including `linked_projects`, `linked_documents` (default empty vec in mapper) |
| `backend/src/domains/persons/enrichment/queries.rs` | `get_enriched`, `list_enriched` (favorites filter, ordering), `search_persons` (LIKE on lower name/email) |
| `backend/src/domains/persons/enrichment/rows.rs` | `ENRICHED_PERSON_COLUMNS` constant, `row_to_enriched` mapping (including default vec for linked_*) |
| `backend/src/domains/persons/enrichment/store.rs` | `PersonEnrichmentStore` struct and constructor |
| `backend/src/domains/persons/enrichment_engine.rs` | `EnrichmentResult` / `EnrichmentResultStore` upsert (claim extraction, candidate creation, `ON CONFLICT DO NOTHING`), apply/reject with review link, `EnrichmentEngineError` |
| `backend/src/domains/persons/expertise.rs` | `PersonExpertise` struct, `PersonExpertiseStore` list, search_by_skill (LIKE), upsert (`ON CONFLICT DO NOTHING`), error |
| `backend/src/domains/persons/export.rs` | `ExportFormat` enum (parse, content_type, extension), `PersonExportService.export` (dossier assembly, markdown rendering with full structure), PDF fallback to markdown, `ExportError` |
| `backend/src/domains/persons/health.rs` | `PersonHealth` struct, `PersonHealthStore` get (subqueries for promises/risks), list_health, list_watchlist, toggle_watchlist (with preference sync `ui:watchlist`), error |
| `backend/src/domains/persons/identity.rs` | Module re-exports (candidate, review, store, etc.) |
| `backend/src/domains/persons/identity/constants.rs` | Event type, prefixes, limit constants |
| `backend/src/domains/persons/identity/errors.rs` | `PersonIdentityError` variants |
| `backend/src/domains/persons/identity/events.rs` | `ReviewCommandEvent` (to event envelope) and `ReviewEvent` (from payload) |

## Drift candidates

Из предоставленного контекста расхождения между кодом, документацией и ADR не видны — текущее содержимое wiki‑страницы `components/backend.md` не встроено, другие артефакты отсутствуют.
