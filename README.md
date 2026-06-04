# Hermes Hub

Hermes Hub - персональная локальная платформа коммуникаций, долговременной памяти и управления знаниями.

Проектируемая система объединяет email, Telegram, WhatsApp, документы, контакты, задачи, календарь, заметки, проекты, AI agents и knowledge graph в единую локальную платформу. Это не почтовый клиент, не мессенджер и не CRM. Центральная идея - долговременная, переносимая память пользователя, построенная на событиях, графе знаний, RAG, vector search и структурированных проекциях, без fine-tuning пользовательских данных.

## Статус

Репозиторий перешел от архитектурного фундамента к первому implementation slice.

Текущий результат:

- продуктовая и архитектурная документация
- базовая структура monorepo
- ADR по ключевым долгосрочным решениям
- roadmap до версии 5.0
- Rust backend foundation с конфигурацией и `GET /healthz`
- local API-token guard for event API reads and writes
- append-only audit log for authorized event API access attempts
- Docker Compose окружение для локальной разработки

## Принципы

- Local first: пользователь владеет данными, облако не является обязательной точкой отказа.
- Knowledge graph first: память живет в графе, индексах и событиях, а не в весах модели.
- Event driven: все значимые изменения представлены событиями.
- AI native: AI является частью всех подсистем, а не отдельным чат-виджетом.
- Long-term product: проектируется конечная система, не MVP.

## Целевая технологическая рамка

- Backend: Rust
- Frontend: SvelteKit
- Desktop: Tauri
- Database: PostgreSQL
- Full text search: Tantivy
- Local AI: Ollama
- Telemetry: OpenTelemetry

## Структура

- [docs/vision](docs/vision) - долгосрочное видение.
- [docs/product](docs/product) - charter, scope и продуктовые границы.
- [docs/architecture](docs/architecture) - системная архитектура и ключевые технические модели.
- [docs/domains](docs/domains) - доменные области.
- [docs/adr](docs/adr) - Architecture Decision Records.
- [docs/agents](docs/agents) - AI agent architecture.
- [docs/ui](docs/ui) - UI architecture и design system vision.
- [docs/roadmap](docs/roadmap) - план развития до версии 5.0.
- [docs/research](docs/research) - вопросы исследования и открытые риски.
- [backend](backend) - Rust backend.
- [frontend](frontend) - будущий SvelteKit frontend.
- [infrastructure](infrastructure) - self-hosted и локальная инфраструктура.
- [tools](tools) - будущие developer и data tools.
- [examples](examples) - будущие спецификации примеров и тестовых сценариев.

## Разработка

Проверить backend:

```sh
make backend-validate
```

Запустить backend локально:

```sh
make backend-run
```

Запустить PostgreSQL и backend с `DATABASE_URL` из `docker/.env`:

```sh
make db-up
make backend-run-dev
```

`/api/events` и `/api/audit/events` требуют локальный API token:

```sh
Authorization: Bearer <HERMES_LOCAL_API_TOKEN>
```

Выполнить smoke test backend + PostgreSQL:

```sh
make backend-smoke-dev
```

Проверить event log migration/store against PostgreSQL:

```sh
make backend-event-log-smoke-dev
```

Проверить replay/projection cursors against PostgreSQL:

```sh
make backend-projection-smoke-dev
```

Проверить projection runner checkpoint semantics against PostgreSQL:

```sh
make backend-projection-runner-smoke-dev
```

Проверить event HTTP API against PostgreSQL:

```sh
make backend-events-api-smoke-dev
```

Создать локальный Docker env и проверить Compose:

```sh
make docker-env
make compose-config
```

## Главные документы

- [Vision Document](docs/vision/vision-document.md)
- [Product Charter](docs/product/product-charter.md)
- [Product Scope](docs/product/product-scope.md)
- [Product Roadmap](docs/roadmap/product-roadmap.md)
- [Architecture Overview](docs/architecture/architecture-overview.md)
- [ADR Index](docs/adr/README.md)
