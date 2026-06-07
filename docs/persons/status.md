# Persons — Статус реализации

## Фаза 0: Переименование contact → person ✓

| Артефакт | Статус |
|---|---|
| Таблица `persons` (бывш. `contacts`) | ✓ |
| `person_identity_candidates` (бывш. `contact_identity_candidates`) | ✓ |
| 4 модуля переименованы | ✓ |
| 40+ бэкенд-файлов обновлены | ✓ |
| 8 фронтенд-файлов обновлены | ✓ |
| 20+ doc-файлов обновлены | ✓ |
| Миграция 0034 | ✓ |

## Фаза 1: Мультиканальная идентичность ✓

| Функция | Статус | Таблица/Модуль |
|---|---|---|
| §5 Multi-channel Identity | ✓ | `person_identities` |
| §6 Identity Resolution | ✓ | `person_identity_candidates` |
| §7 Contact Merge | ✓ | `person_identity.rs` |
| §8 Contact Roles | ✓ | `person_roles` |
| §9 Contact Personas | ✓ | `person_personas` |
| §10 Contact Types | ✓ | `person_type` column |

## Фаза 2: Память персоны ✓

| Функция | Статус | Таблица/Модуль |
|---|---|---|
| §20 Memory Cards | ✓ | `person_memory_cards` |
| §21 Personal Facts | ✓ | `person_facts` |
| §22 Memory Confidence | ✓ | `confidence` column on facts |
| §23 Contact Sources | ✓ | `source` column on all tables |
| §24 Knowledge Conflicts | ✓ | `person_knowledge_conflicts` |
| §25 Memory Decay | ✓ | `PersonFactStore::decay_unverified()` |
| §26 Contact Snapshots | ✓ | `person_snapshots` |
| §13 History Diff | ✓ | `PersonSnapshotStore::history_diff()` |

## Фаза 3: Таймлайн отношений ✓

| Функция | Статус | Таблица/Модуль |
|---|---|---|
| §11 Relationship Timeline | ✓ | `relationship_events` |
| §12 Memory Timeline | ✓ | `relationship_events` |
| §14 Milestones | ✓ | `event_type` filtering |

## Фаза 4: Communication DNA ✓

| Функция | Статус | Реализация |
|---|---|---|
| §15 Contact Inbox | — | `communication_messages` (вне модуля persons) |
| §16 Communication Profile | ✓ | `EnrichedPerson` поля |
| §17 Communication DNA | ✓ | DNA columns: `communication_style`, `verbosity`, `technical_depth`, `question_frequency`, `call_preference`, `response_pattern` |
| §18 Language Profile | ✓ | `language` column + `active_hours`/`active_days` JSONB |
| §19 Preferences | ✓ | `person_preferences` |

## Фаза 5: Enrichment Engine ✓

| Функция | Статус | Реализация |
|---|---|---|
| §67 Enrichment Engine | ✓ | `person_enrichment_engine.rs` |
| §68 Auto Discovery | ✓ | `EnrichmentResultStore` |
| §69 GitHub Intelligence | ✓ | через enrichment engine |
| §70 LinkedIn Intelligence | ✓ | через enrichment engine |
| §71 Profile Verification | ✓ | confidence scoring |

## Фаза 6: Экспертиза ✓

| Функция | Статус | Реализация |
|---|---|---|
| §28 Expertise | ✓ | `person_expertise` |
| §29 Skill Graph | ✓ | `search_by_skill()` |

## Фаза 7: Доверие ✓

| Функция | Статус | Реализация |
|---|---|---|
| §30-35 Trust & Reliability | ✓ | `person_promises` (fulfill/broken/forgiven), `person_risks` (low/medium/high/critical) |

## Фаза 8: Здоровье ✓

| Функция | Статус | Реализация |
|---|---|---|
| §36-40 Health & Monitoring | ✓ | `health_status`, `watchlist`, `communication_gap_days` columns + `PersonHealthStore` |

## Фаза 9: AI Investigator ✓

| Функция | Статус | Реализация |
|---|---|---|
| §64 Dossier Generator | ✓ | `PersonInvestigator::assemble_dossier()` |
| §65 AI Brief | ✓ | `PersonInvestigator::meeting_prep()` |
| §74 Investigator Agent | ✓ | `person_investigator.rs` |

## Фаза 10: Аналитика ✓

| Функция | Статус | Реализация |
|---|---|---|
| §27 Relationship Score | ✓ | `person_analytics.rs` |
| §41-57 Analytics | ✓ | heatmap, communication costs, shared context |
| §72 Intelligence Score | ✓ | `intelligence_score()` |
| §73 Knowledge Gaps | ✓ | через intelligence score |

## Фаза 11: Экспорт ✓

| Функция | Статус | Реализация |
|---|---|---|
| §60 Documents Hub | — | `documents` module (вне persons) |
| §61 Tasks | — | `task_candidates` module (вне persons) |
| §62 Decisions | — | `relationship_events` filtered by type |
| §63 Notes | ✓ | `notes` column + API |
| §66 Export | ✓ | Markdown/JSON export |

## Не реализовано (вне скоупа persons)

| Функция | Причина |
|---|---|
| Relationship Map (§75) | Зависит от работающей graph projection |
| Mutual Connections (§76) | Зависит от graph projection |
| Organization Module (§77) | Отдельный реализованный модуль; `organization_reference` остаётся compatibility/cache field |
| Digital Twin (§78) | Композитный read-side view, требует UI |
| Enrichment провайдеры (реальные API-вызовы) | Провайдеры спроектированы как pluggable traits, реализации — следующий шаг |

## Итого

| Метрика | Значение |
|---|---|
| Реализовано разделов спеки | 63 из 83 |
| Не в скоупе persons | 8 |
| Отложено (зависимости) | 12 |
