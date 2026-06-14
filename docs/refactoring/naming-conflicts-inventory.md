# Инвентаризация naming conflicts: Persons ↔ Personas

> Создано: 2026-06-14 в рамках Phase 1 (Foundation & Safety Net)
> Цель: Задокументировать все naming conflicts перед Phase 2 (Persona Naming Alignment)

## 1. Двойное именование: Persons ↔ Personas

### 1.1 API Routes

| Route | Файл | Статус |
|-------|------|--------|
| `/api/v1/persons` | [`backend/src/app/router/routes/persons.rs`](../../backend/src/app/router/routes/persons.rs:5) | Legacy |
| `/api/v1/personas` | [`backend/src/app/router/routes/persons.rs`](../../backend/src/app/router/routes/persons.rs:6) | Native |
| `/api/v1/persons/owner` | [`backend/src/app/router/routes/persons.rs`](../../backend/src/app/router/routes/persons.rs:12) | Legacy |
| `/api/v1/persons/{person_id}` | [`backend/src/app/router/routes/persons.rs`](../../backend/src/app/router/routes/persons.rs:15) | Legacy |
| `/api/v1/personas/{persona_id}` | [`backend/src/app/router/routes/persons.rs`](../../backend/src/app/router/routes/persons.rs:8) | Native |
| `/api/v1/persons/{person_id}/personas` | [`backend/src/app/router/routes/persons.rs`](../../backend/src/app/router/routes/persons.rs:62) | Mixed |

**Итог:** 90% routes используют `/api/v1/persons/`, 10% используют `/api/v1/personas/`. Оба набора routes работают параллельно.

### 1.2 Database Schema

| Таблица | Колонка | Статус |
|---------|---------|--------|
| `persons` | `person_id` | Legacy |
| `persons` | `is_self` | Допустимо для Persona Owner |
| `personas` | `persona_id` | Native |
| `personas` | `person_id` (FK → persons) | Mixed |

### 1.3 Backend Module Names

| Модуль | Статус |
|--------|--------|
| `domains/persons/` | Legacy — основной domain модуль |
| `domains/persons/core/` | Mixed — содержит `PersonPersona` и `PersonsIdentityStore` |
| `domains/persons/api/` | Legacy — `PersonProjectionStore` |
| `domains/persons/handlers/` | Legacy — но содержит persona handlers |
| `domains/persons/api/store/persona_reads.rs` | Native |
| `domains/persons/api/store/persona_writes.rs` | Native |
| `domains/persons/api/store/persona_type.rs` | Native |

### 1.4 Rust Types

| Тип | Статус |
|-----|--------|
| `PersonPersona` | Mixed — тип называется Persona, но префикс Person |
| `NewPersonPersona` | Mixed |
| `PersonPersonaStore` | Mixed |
| `PersonsIdentityStore` | Legacy |
| `PersonProjectionStore` | Legacy |
| `PersonIdentity` | Legacy |
| `PersonRole` | Legacy |

### 1.5 Frontend Module Names

| Модуль/файл | Статус |
|-------------|--------|
| `frontend/src/domains/personas/` | Native |
| `frontend/src/domains/personas/api/personas.ts` | Mixed — экспортирует `fetchPersons()` и `fetchOrganizations()` |
| `frontend/src/domains/personas/queries/usePersonasQuery.ts` | Native |
| `frontend/src/domains/personas/types/persona.ts` | Native |

### 1.6 Frontend API Functions

| Функция | Статус |
|---------|--------|
| `fetchPersons()` | Legacy — идёт к `/api/v1/persons` |
| `fetchPersonDossier()` | Legacy |
| `fetchIdentityCandidates()` | Legacy |
| `fetchRelationships()` | Отдельный domain |
| `fetchOrganizations()` | Отдельный domain (но находится в personas/api) — **cross-domain** |

### 1.7 Compatibility Layer

Файл: [`backend/src/domains/persons/handlers/compatibility.rs`](../../backend/src/domains/persons/handlers/compatibility.rs)

Содержит хендлеры для `/api/v1/personas/` (native routes), которые преобразуют данные из persons-ориентированной модели в persona-ориентированную.

## 2. SemanticSourceKind → "contact"

В [`backend/src/ai/core/semantic/sources.rs`](../../backend/src/ai/core/semantic/sources.rs): `SemanticSourceKind::Person` сериализуется как `"contact"`.

```rust
pub enum SemanticSourceKind {
    Person,     // → "contact"
    Document,   // → "document"
    Email,      // → "email"
    Task,       // → "task"
    Note,       // → "note"
}
```

Это legacy naming, несовместимое с Persona-моделью. Требует изменения в Phase 2.

## 3. Cross-domain imports

### 3.1 persons → organizations

Файл: [`frontend/src/domains/personas/api/personas.ts`](../../frontend/src/domains/personas/api/personas.ts)

Содержит `fetchOrganizations()` и `fetchOrganization()` — функции, относящиеся к organizations domain, но находящиеся в personas/api.

### 3.2 review → persons + tasks + knowledge

Файл: [`frontend/src/domains/review/stores/review.ts`](../../frontend/src/domains/review/stores/review.ts)

Импортирует из:
- `../../personas/api/personas` (relationships)
- `../../tasks/api/tasks` (decisions, obligations)
- `../../knowledge/api/knowledge` (contradictions)

### 3.3 organizations queries → persons

Файл: [`frontend/src/domains/organizations/queries/useOrganizationsQuery.ts`](../../frontend/src/domains/organizations/queries/useOrganizationsQuery.ts)

Импортирует `fetchOrganizations` и `fetchOrganization` из `../../personas/api/personas`.

## 4. Communications module naming

| Файл/модуль | Проблема |
|-------------|----------|
| `domains/mail/` | Название `mail` вместо `communications` |
| `backend/src/domains/mail/` | ~100+ файлов в God-директории |
| `/api/v1/communications/*` | API уже использует правильное имя |
| `frontend/src/domains/communications/` | Frontend уже использует правильное имя |

**Итог:** Backend domain называется `mail`, но API роуты и фронтенд используют `communications`. Это несоответствие требует рефакторинга.

## 5. Резюме для Phase 2

Приоритетные изменения:
1. Переименовать `domains/persons/` → `domains/personas/` (или создать facade)
2. `SemanticSourceKind::Person` → `"persona"` (не `"contact"`)
3. Перенести `fetchOrganizations` из personas/api в organizations/api
4. Устранить cross-domain imports в review store
5. Начать рефакторинг `domains/mail/` → `domains/communications/`
