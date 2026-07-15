### Summary / Резюме

Создать страницу `operations/configuration.md` в русской Obsidian wiki, описывающую архитектурный контракт из файла `scripts/architecture-contract.json`. Страница должна задокументировать разрешённые и запрещённые взаимодействия для backend- и frontend-слоёв, принадлежащие домены, виды взаимодействий, модель бизнес-маршрутов и корни кэширования провайдеров. Все данные берутся исключительно из встроенного контекста.

### Proposed pages / Предлагаемые страницы

#### `operations/configuration.md`

```markdown
# Конфигурация архитектурного контракта

Файл `scripts/architecture-contract.json` определяет архитектурный контракт — набор правил допустимых и запрещённых взаимодействий между слоями backend и frontend, модель бизнес-маршрутов и корни кэширования провайдеров.

## Общая информация

- **Версия схемы:** `1`
- **Связанный ADR:** `docs/adr/ADR-0098-provider-neutral-communications-api-and-strict-boundaries.md` (содержание ADR не подтверждено данным контекстом)

## Виды взаимодействий (`interaction_kinds`)

- `direct_call`
- `command_port`
- `query_port`
- `event`
- `projection`
- `runtime_integration_api`

## Backend-слои (`backend.layers`)

### Принадлежащие домены (`domains.owned`)

Следующие домены входят в состав слоя `domains`:

`agents`, `calendar`, `communications`, `decisions`, `documents`, `graph`, `knowledge`, `mail`, `notes`, `obligations`, `organizations`, `personas`, `persons`, `projects`, `radar`, `relationships`, `signal_hub`, `tasks`, `timeline`

### Правила взаимодействий

| Слой | Разрешённые взаимодействия (`allow`) | Запрещённые взаимодействия (`deny`) |
|------|--------------------------------------|-------------------------------------|
| `app` | `domain_command_ports`, `domain_query_ports`, `integration_runtime_integration_api`, `platform` | `stores`, `business_orchestration` |
| `domains` | `own_modules`, `platform`, `pure_engines` | `other_domains`, `integrations`, `app`, `workflows`, `vault` |
| `integrations` | `own_modules`, `platform`, `vault`, `external_sdks` | `domains`, `app`, `workflows`, `business_truth` |
| `workflows` | `domain_command_ports`, `domain_query_ports`, `events`, `platform` | `stores`, `handlers`, `integration_clients` |
| `engines` | `own_projections`, `own_indexes`, `platform` | `business_domain_mutation`, `integrations` |
| `ai` | `candidates`, `summaries`, `classifications`, `embeddings` | `domain_stores`, `domain_mutation`, `source_of_truth` |
| `platform` | `neutral_contracts`, `technical_infrastructure` | `domains`, `integrations`, `workflows`, `business_table_sql` |
| `vault` | `secrets`, `sessions`, `runtime_state` | `business_truth` |

## Frontend-слои (`frontend.layers`)

| Слой | Разрешённые взаимодействия (`allow`) | Запрещённые взаимодействия (`deny`) |
|------|--------------------------------------|-------------------------------------|
| `app` | `compose_domain_views`, `compose_domain_stores`, `routing` | *(отсутствует в контракте)* |
| `domains` | `own_modules`, `platform`, `shared` | `other_frontend_domains`, `provider_runtime_state`, `integrations` |
| `integrations` | `provider_setup`, `provider_runtime`, `platform`, `shared` | `business_truth`, `domains` |

## Модель бизнес-маршрутов (`frontend.business_route_model`)

| Ключ | Значение |
|------|----------|
| `business_root` | `/api/v1/communications` |
| `forbidden_provider_business_roots` | `/api/v1/integrations/mail/*`, `/api/v1/integrations/telegram/*`, `/api/v1/integrations/whatsapp/*` |
| `runtime_root` | `/api/v1/integrations/{provider}` |

## Корни кэширования провайдеров (`frontend.provider_business_cache_roots`)

| Ключ | Значение |
|------|----------|
| `forbidden` | `["telegram", "whatsapp", "mail"]` |
| `business_query_key_root` | `"communications"` |
| `runtime_query_key_root` | `["integrations", "<provider>", "runtime"]` |
```

### Source coverage / Покрытие источников

- **`scripts/architecture-contract.json`** (полный файл, 4043 байта)
  - `schema_version` (1)
  - `adr` (путь к ADR)
  - `interaction_kinds` (6 видов)
  - `backend.layers`:
    - `domains.owned` (список из 19 доменов)
    - правила `allow`/`deny` для слоёв: `app`, `domains`, `integrations`, `workflows`, `engines`, `ai`, `platform`, `vault`
  - `frontend.layers`:
    - правила `allow`/`deny` для слоёв: `app`, `domains`, `integrations`
  - `frontend.business_route_model`:
    - `business_root`, `forbidden_provider_business_roots`, `runtime_root`
  - `frontend.provider_business_cache_roots`:
    - `forbidden`, `business_query_key_root`, `runtime_query_key_root`

### Drift candidates / Кандидаты на drift

Из представленного контекста (один файл `scripts/architecture-contract.json`) расхождения не видны. Согласованность с упомянутым ADR не может быть проверена, так как его содержание не встроено в контекст.
