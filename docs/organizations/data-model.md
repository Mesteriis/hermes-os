# Organizations — Модель данных

## organizations

| Колонка | Тип | Описание |
|---|---|---|
| `organization_id` | TEXT PK | `org:v1:{nanos}` |
| `display_name` | TEXT NOT NULL | Отображаемое имя |
| `legal_name` | TEXT | Юридическое название |
| `org_type` | TEXT | 17 типов |
| `status` | TEXT | active/inactive/archived/watchlist/blocked/unknown |
| `country`, `city`, `address` | TEXT | |
| `website`, `industry`, `description` | TEXT | |
| `primary_language`, `timezone` | TEXT | |
| `trust_score` | SMALLINT | 0–100 |
| `health_status` | TEXT | healthy/needs_attention/at_risk/dormant |
| `priority` | TEXT | low/medium/high/critical |
| `tags` | JSONB | |
| `org_metadata` | JSONB | |
| `registration_number`, `vat`, `cif`, `nif`, `tax_id` | TEXT | Legal |
| `communication_style`, `verbosity`, `formality` | TEXT | DNA |
| `secondary_languages` | JSONB | |
| `watchlist` | BOOL | |
| `last_interaction_at`, `interaction_count` | | |

## Остальные 26 таблиц

Следуют тому же паттерну Persons: identity/alias/domain/department/contact_link/related, facts/memory_cards/preferences/required_docs/snapshots/conflicts, timeline_events/templates, portals/procedures/playbooks/quick_actions, financial_info/contracts/compliance/services/products, enrichment_results, risks/alerts.

Полная схема в миграциях `0038`–`0043`.
