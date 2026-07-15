# ADR-0073 Backend Module Organization

Status: Accepted

## Context

На момент принятия этого ADR backend состоит из 106 файлов `.rs`, лежащих плоско в `backend/src/` без иерархии модулей. Файл `lib.rs` содержит 10 571 строку и смешивает: декларации модулей, HTTP routing, обработчики запросов, AppState, CORS, tracing, парсинг query-параметров и валидационные хелперы.

При росте системы такая структура становится неуправляемой:
- невозможно определить границы bounded context без чтения имён всех файлов;
- интеграции лежат вперемешку с доменами;
- общая платформа и AI не отделены от бизнес-логики;
- `lib.rs` невозможно рецензировать или тестировать изолированно.

## Decision

Вводится семислойная организация backend-крейта:

```text
backend/src/
├── app/            — HTTP-слой приложения (router, handlers, state, error)
├── domains/        — Bounded contexts (mail, personas, calendar, tasks, ...)
├── engines/        — Общие движки (search, automation)
├── integrations/   — Внешние адаптеры (gmail, telegram, whatsapp, ollama)
├── ai/             — AI-слой (семантические эмбеддинги, retrieval, AI-сервис)
├── workflows/      — Бизнес-процессы (email sync pipeline, email intelligence)
└── platform/       — Техническая платформа (config, events, secrets, db, audit, ...)
```

### Правила размещения

1. **app/** — HTTP-роутинг, обработчики, AppState, error types верхнего уровня. Не содержит бизнес-логики.
2. **domains/** — Каждый подкаталог — самостоятельный bounded context. Домены не импортируют друг друга напрямую. Для cross-domain коммуникации используют контракт из `ADR-architecture-communication-contract`.
3. **engines/** — Общие движки, не привязанные к конкретному домену.
4. **integrations/** — Адаптеры внешних систем. Не содержат бизнес-логики, только транспорт/протокол.
5. **ai/** — AI-компоненты изолированы от доменов. Домены используют AI через events или явные сервисные границы.
6. **workflows/** — Бизнес-процессы, координирующие несколько доменов.
7. **platform/** — Техническая платформа: конфигурация, события, БД, хранилище, секреты, аудит, capabilities.

### Domain Isolation

Домены не должны напрямую импортировать друг друга:

```rust
// ❌ Запрещено
use crate::domains::tasks::api::TaskStore;

// ✅ Разрешено — через events
use crate::platform::events::EventStore;

// ✅ Разрешено — через command/query/event contract владельца
use crate::platform::events::EventStore;
```

Прежнее исключение для `domains::graph` упразднено. Graph является доменом и
не может использоваться как shared spine через прямые импорты из других
доменов. Детальный контракт слоёв и допустимых способов взаимодействия описан в
`ADR-architecture-communication-contract`.

### Порог размера файлов

Файлы крупнее 700 строк требуют header-комментария в начале файла с объяснением, почему файл не разделён. Пример:

```rust
// This file exceeds 700 lines because it groups a single-responsibility
// store with its associated types (model, errors, queries) that share
// tight coupling through SQL query construction. Splitting would
// require either duplicating SQL fragments or introducing an
// abstraction layer that adds indirection without reducing complexity.
```

### Именование

- Имена файлов отражают доменную ответственность, а не техническую реализацию.
- `mod.rs` используется для реэкспорта публичного API модуля.
- Внутренние детали остаются `pub(crate)`.

## Consequences

- Структура проекта самодокументируется: по расположению файла понятна его роль.
- Рефакторинг `lib.rs` устраняет god file: routing, handlers, state, error разделены.
- Domain isolation предотвращает циклические зависимости при росте системы.
- Порог в 700 строк с обязательным обоснованием предотвращает повторное появление god files.
- Импорты становятся длиннее (`crate::domains::communications::core` вместо `crate::communications`), но это плата за явные границы.
