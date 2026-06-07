# Persons — Модель данных

## persons

Основная сущность. Бывшая таблица `contacts`, переименована миграцией 0034.

| Колонка | Тип | Описание |
|---|---|---|
| `person_id` | TEXT PK | `person:v1:email:{len}:{email}` |
| `display_name` | TEXT NOT NULL | Отображаемое имя |
| `email_address` | TEXT NOT NULL UNIQUE | Нормализованный email |
| `language` | TEXT | Предпочитаемый язык |
| `tone` | TEXT | Тон общения |
| `trust_score` | SMALLINT | 0–100 |
| `avg_response_hours` | FLOAT | Среднее время ответа |
| `preferred_channel` | TEXT | Предпочитаемый канал |
| `interaction_count` | INT | Количество взаимодействий |
| `frequent_topics` | JSONB | Частые темы |
| `writing_style` | TEXT | Стиль письма |
| `person_metadata` | JSONB | Произвольные метаданные |
| `is_favorite` | BOOL | Избранное |
| `notes` | TEXT | Заметки |
| `person_type` | TEXT | Тип персоны |
| `primary_role` | TEXT | Основная роль |
| `organization_reference` | TEXT | Ссылка на организацию |
| `timezone` | TEXT | Часовой пояс |
| `communication_style` | TEXT | DNA: formal/informal |
| `verbosity` | TEXT | DNA: verbose/concise/balanced |
| `technical_depth` | TEXT | DNA: technical/business/mixed |
| `question_frequency` | TEXT | DNA: high/medium/low |
| `call_preference` | TEXT | DNA: prefers_calls/avoids_calls/neutral |
| `response_pattern` | TEXT | DNA: fast/standard/slow |
| `active_hours` | JSONB | DNA: часы активности |
| `active_days` | JSONB | DNA: дни активности |
| `health_status` | TEXT | healthy/needs_attention/at_risk/dormant |
| `last_health_check` | TIMESTAMPTZ | Последняя проверка здоровья |
| `communication_gap_days` | INT | Дней без коммуникации |
| `watchlist` | BOOL | В списке наблюдения |
| `created_at` | TIMESTAMPTZ | |
| `updated_at` | TIMESTAMPTZ | |

## person_identities

Мультиканальные идентификаторы.

| Колонка | Тип | Описание |
|---|---|---|
| `id` | UUID PK | |
| `person_id` | TEXT FK → persons | |
| `identity_type` | TEXT | email, telegram, whatsapp, phone, github, linkedin, website, mastodon, x, stackoverflow, habr, medium, orcid, google_scholar |
| `identity_value` | TEXT | Значение идентификатора |
| `source` | TEXT | Источник |
| `confidence` | REAL | 0–1 |
| `last_verified_at` | TIMESTAMPTZ | |
| `status` | TEXT | active, outdated, unreachable, blocked |
| `metadata` | JSONB | |
| `created_at` | TIMESTAMPTZ | |
| `updated_at` | TIMESTAMPTZ | |

UNIQUE: `(identity_type, identity_value)` WHERE `status = 'active'`

## person_roles

Many-to-many роли.

| Колонка | Тип |
|---|---|
| `id` | UUID PK |
| `person_id` | TEXT FK → persons |
| `role` | TEXT |
| `assigned_by` | TEXT |
| `assigned_at` | TIMESTAMPTZ |

UNIQUE: `(person_id, role)`

## person_personas

Именованные контексты взаимодействия.

| Колонка | Тип |
|---|---|
| `persona_id` | TEXT PK |
| `person_id` | TEXT FK → persons |
| `name` | TEXT NOT NULL |
| `context` | TEXT |
| `default_tone` | TEXT |
| `default_language` | TEXT |
| `preferred_channel` | TEXT |
| `metadata` | JSONB |
| `created_at` | TIMESTAMPTZ |
| `updated_at` | TIMESTAMPTZ |

## person_facts

Извлечённые факты с provenance.

| Колонка | Тип |
|---|---|
| `id` | UUID PK |
| `person_id` | TEXT FK → persons |
| `fact_type` | TEXT |
| `value` | TEXT |
| `source` | TEXT |
| `confidence` | REAL 0–1 |
| `last_verified_at` | TIMESTAMPTZ |
| `valid_from` | TIMESTAMPTZ |
| `valid_to` | TIMESTAMPTZ |
| `is_active` | BOOL |
| `created_at` | TIMESTAMPTZ |
| `updated_at` | TIMESTAMPTZ |

## person_memory_cards

Карточки памяти.

| Колонка | Тип |
|---|---|
| `id` | UUID PK |
| `person_id` | TEXT FK → persons |
| `title` | TEXT NOT NULL |
| `description` | TEXT |
| `source` | TEXT |
| `confidence` | REAL 0–1 |
| `importance` | SMALLINT 1–10 |
| `created_at` | TIMESTAMPTZ |
| `last_verified_at` | TIMESTAMPTZ |

## person_preferences

Предпочтения коммуникации.

| Колонка | Тип |
|---|---|
| `id` | UUID PK |
| `person_id` | TEXT FK → persons |
| `preference_type` | TEXT |
| `value` | TEXT |
| `source` | TEXT |
| `confidence` | REAL 0–1 |
| `last_verified_at` | TIMESTAMPTZ |
| `created_at` | TIMESTAMPTZ |
| `updated_at` | TIMESTAMPTZ |

UNIQUE: `(person_id, preference_type)`

## person_snapshots

Снимки состояния для history diff.

| Колонка | Тип |
|---|---|
| `id` | UUID PK |
| `person_id` | TEXT FK → persons |
| `snapshot_date` | TIMESTAMPTZ |
| `data` | JSONB |
| `source` | TEXT |
| `created_at` | TIMESTAMPTZ |

## person_knowledge_conflicts

Обнаруженные противоречия.

| Колонка | Тип |
|---|---|
| `id` | UUID PK |
| `person_id` | TEXT FK → persons |
| `field` | TEXT |
| `value_a` | TEXT |
| `value_b` | TEXT |
| `source_a` | TEXT |
| `source_b` | TEXT |
| `detected_at` | TIMESTAMPTZ |
| `resolved_at` | TIMESTAMPTZ |
| `resolution` | TEXT |

## relationship_events

Таймлайн событий отношений.

| Колонка | Тип |
|---|---|
| `id` | UUID PK |
| `person_id` | TEXT FK → persons |
| `event_type` | TEXT |
| `title` | TEXT NOT NULL |
| `description` | TEXT |
| `occurred_at` | TIMESTAMPTZ |
| `source` | TEXT |
| `related_entity_id` | TEXT |
| `related_entity_kind` | TEXT |
| `confidence` | REAL 0–1 |
| `metadata` | JSONB |
| `created_at` | TIMESTAMPTZ |

## enrichment_results

Результаты enrichment из внешних источников.

| Колонка | Тип |
|---|---|
| `id` | UUID PK |
| `person_id` | TEXT FK → persons |
| `source` | TEXT |
| `url` | TEXT |
| `data` | JSONB |
| `confidence` | REAL 0–1 |
| `status` | TEXT: pending/applied/rejected/conflict |
| `last_checked_at` | TIMESTAMPTZ |
| `applied_at` | TIMESTAMPTZ |
| `created_at` | TIMESTAMPTZ |

## person_expertise

Навыки и домены.

| Колонка | Тип |
|---|---|
| `id` | UUID PK |
| `person_id` | TEXT FK → persons |
| `skill` | TEXT NOT NULL |
| `domain` | TEXT |
| `evidence` | TEXT |
| `source` | TEXT |
| `confidence` | REAL 0–1 |
| `last_verified_at` | TIMESTAMPTZ |
| `endorsed_by_person_id` | TEXT |
| `created_at` | TIMESTAMPTZ |
| `updated_at` | TIMESTAMPTZ |

## person_promises

Отслеживаемые обещания.

| Колонка | Тип |
|---|---|
| `id` | UUID PK |
| `person_id` | TEXT FK → persons |
| `description` | TEXT NOT NULL |
| `source_message_id` | TEXT |
| `promised_at` | TIMESTAMPTZ |
| `due_at` | TIMESTAMPTZ |
| `fulfilled_at` | TIMESTAMPTZ |
| `status` | TEXT: pending/fulfilled/broken/forgiven |
| `created_at` | TIMESTAMPTZ |
| `updated_at` | TIMESTAMPTZ |

## person_risks

Риски.

| Колонка | Тип |
|---|---|
| `id` | UUID PK |
| `person_id` | TEXT FK → persons |
| `risk_type` | TEXT |
| `description` | TEXT |
| `severity` | TEXT: low/medium/high/critical |
| `source` | TEXT |
| `confidence` | REAL 0–1 |
| `created_at` | TIMESTAMPTZ |
| `resolved_at` | TIMESTAMPTZ |
| `resolution` | TEXT |

## person_identity_candidates

Кандидаты на merge/split. Переименована из `contact_identity_candidates` миграцией 0034.

| Колонка | Тип |
|---|---|
| `identity_candidate_id` | TEXT PK |
| `candidate_kind` | TEXT: merge_persons/attach_email_address/split_person |
| `left_person_id` | TEXT |
| `right_person_id` | TEXT |
| `email_address` | TEXT |
| `evidence_summary` | TEXT |
| `confidence` | REAL |
| `review_state` | TEXT: suggested/user_confirmed/user_rejected |
| `event_id` | TEXT |
| `actor_id` | TEXT |
| `generated_at` | TIMESTAMPTZ |
| `reviewed_at` | TIMESTAMPTZ |
| `updated_at` | TIMESTAMPTZ |
