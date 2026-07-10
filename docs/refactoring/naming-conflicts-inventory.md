# Инвентаризация naming conflicts: Legacy Persons ↔ Personas

> Создано: 2026-06-14 в рамках Phase 1 (Foundation & Safety Net)
> Цель: Задокументировать все naming conflicts перед Phase 2 (Persona Naming Alignment)

## 1. Двойное именование: Legacy Persons ↔ Personas

### 1.1 API Routes

| Route | Файл | Статус |
|-------|------|--------|
| `/api/v1/personas` | [`backend/src/app/router/routes/personas.rs`](../../backend/src/app/router/routes/personas.rs) | Native |
| `/api/v1/personas/owner` | [`backend/src/app/router/routes/personas.rs`](../../backend/src/app/router/routes/personas.rs) | Native |
| `/api/v1/personas/{persona_id}` | [`backend/src/app/router/routes/personas.rs`](../../backend/src/app/router/routes/personas.rs) | Native |
| `/api/v1/personas/{persona_id}/interaction-contexts` | [`backend/src/app/router/routes/personas.rs`](../../backend/src/app/router/routes/personas.rs) | Native |

**Итог:** `/api/v1/personas/*` теперь покрывает активную Persona profile
surface, включая owner, dossier и profile subresources. `/api/v1/persons/*`
удалён из активного router surface; тесты закрепляют 404 для legacy route set.

### 1.2 Database Schema

| Таблица | Колонка | Статус |
|---------|---------|--------|
| `personas` | `person_id` | Transitional primary key; table renamed in `0180_rename_persons_table_to_personas.sql` |
| `personas` | `is_self` | Допустимо для Persona Owner |
| `personas` | `persona_id` API field | Native read-model/API identifier mapped from transitional storage |

### 1.3 Backend Module Names

| Модуль | Статус |
|--------|--------|
| `domains/personas/` | Native module path over Persona SQL storage |
| `domains/personas/core/` | Native — содержит `PersonaInteractionContext`, `PersonaIdentityStore` и `PersonaIdentityReviewStore` |
| `domains/personas/api/` | Native — `PersonaProjectionStore` over `personas` table |
| `app/handlers/personas/profile/` | Native app boundary over Persona storage |
| `domains/personas/api/store/persona_reads.rs` | Native |
| `domains/personas/api/store/persona_writes.rs` | Native |
| `domains/personas/api/store/persona_type.rs` | Native |

### 1.4 Rust Types

| Тип | Статус |
|-----|--------|
| `PersonaInteractionContext` | Native |
| `NewPersonaInteractionContext` | Native |
| `PersonaInteractionContextStore` | Native |
| `PersonaIdentityStore` | Native |
| `PersonaIdentityReviewStore` | Native |
| `PersonaProjectionStore` | Native |
| `PersonaIdentity` | Native |
| `PersonaRole` | Native |

`persona_interaction_contexts` is the physical storage for named interaction
contexts. It replaces the legacy `person_personas` table name; active routes use
`/api/v1/personas/{persona_id}/interaction-contexts`.

### 1.5 Runtime, consumer and event names

| Name | Статус |
|------|--------|
| `persona_derived_evidence` | Native runtime / event consumer name |
| `person_derived_evidence` | Legacy replay alias only |
| `persona_identity_review_inbox` | Native runtime / event consumer name |
| `person_identity_review_inbox` | Migrated legacy runtime name |
| `persona_identity.review_state_changed` | Native canonical event name |
| `person_identity.review_state_changed` | Append-only history compatibility only |
| `persona_identity.candidate.detected` | Native canonical event name |
| `person_identity.candidate.detected` | Append-only history compatibility only |

Migrations `0188_rename_persona_derived_evidence_runtime_names.sql` and
`0189_rename_persona_identity_review_inbox_runtime_names.sql` preserve consumer,
cursor, runtime and replay state while moving active runtime identifiers to the
Persona naming. Code may still read the legacy event names from append-only
history and may accept `person_derived_evidence` as a replay-target alias, but
new events and runtime rows use the `persona_*` names.

### 1.6 Frontend Module Names

| Модуль/файл | Статус |
|-------------|--------|
| `frontend/src/domains/personas/` | Native |
| `frontend/src/domains/personas/api/personas.ts` | Native — активные Persona calls идут в `/api/v1/personas`; organization helpers вынесены в organizations domain |
| `frontend/src/domains/personas/queries/usePersonasQuery.ts` | Native |
| `frontend/src/domains/personas/types/persona.ts` | Native |

### 1.7 Frontend API Functions

| Функция | Статус |
|---------|--------|
| `fetchPersonas()` | Native compatibility bridge — идёт к `/api/v1/personas` |
| `fetchPersonDossier()` | Native compatibility bridge — идёт к `/api/v1/personas/{persona_id}/dossier` |
| `fetchIdentityCandidates()` | Legacy |
| `fetchRelationships()` | Отдельный domain |
| `fetchOrganizations()` | Отдельный organizations domain |

### 1.8 Compatibility Layer

Файл: [`backend/src/app/handlers/personas/profile/personas.rs`](../../backend/src/app/handlers/personas/profile/personas.rs)

Содержит хендлеры для `/api/v1/personas/` routes, которые отдают
Persona-native read model поверх `personas` storage. `/api/v1/persons/*`
retired и не должен возвращаться в router surface.

## 2. SemanticSourceKind::Persona → persisted "person"

В [`backend/src/ai/core/semantic/models.rs`](../../backend/src/ai/core/semantic/models.rs):
`SemanticSourceKind::Persona` сериализуется как persisted source kind
`"person"`. Legacy `"contact"` остаётся только входным parser-alias для старых
записей; new code uses the Persona-native Rust variant.

```rust
pub enum SemanticSourceKind {
    Persona,    // → "person" compatibility storage string
    Document,   // → "document"
    Message,    // → "message"
    Task,       // → "task"
    Project,    // → "project"
}
```

Это уже выровнено для новых записей; `"contact"` остаётся только для чтения
старых строк.

## 3. Cross-domain imports

### 3.1 personas → organizations

Статус: resolved for frontend API helpers.

`fetchOrganizations()` и `fetchOrganization()` живут в
[`frontend/src/domains/organizations/api/organizations.ts`](../../frontend/src/domains/organizations/api/organizations.ts).

### 3.2 review → personas + tasks + knowledge

Файл: [`frontend/src/domains/review/stores/review.ts`](../../frontend/src/domains/review/stores/review.ts)

Статус: partially resolved. Relationship review helpers live in
[`frontend/src/domains/review/api/workspace.ts`](../../frontend/src/domains/review/api/workspace.ts).
The remaining cross-domain imports to tasks/knowledge are tracked outside the
Persona cleanup slice.

### 3.3 organizations queries → personas

Файл: [`frontend/src/domains/organizations/queries/useOrganizationsQuery.ts`](../../frontend/src/domains/organizations/queries/useOrganizationsQuery.ts)

Статус: resolved. Imports now come from `../api/organizations`.

## 4. Communications module naming

| Файл/модуль | Проблема |
|-------------|----------|
| `domains/communications/` | Название `mail` вместо `communications` |
| `backend/src/domains/communications/` | ~100+ файлов в God-директории |
| `/api/v1/communications/*` | API уже использует правильное имя |
| `frontend/src/domains/communications/` | Frontend уже использует правильное имя |

**Итог:** Backend domain называется `mail`, но API роуты и фронтенд используют `communications`. Это несоответствие требует рефакторинга.

## 5. Резюме для Phase 2

Приоритетные изменения:
1. Завершено: физический модуль переименован в `domains/personas/`; таблица
   переименована в `personas`; `/api/v1/persons/*` удалён из active route set.
2. Завершено: `SemanticSourceKind::Persona` пишет persisted `"person"`;
   `"contact"` сохранён только для чтения legacy строк.
3. Завершено: `fetchOrganizations` перенесён из personas API в organizations API
4. Устранить cross-domain imports в review store
5. Продолжить рефакторинг Communications backend naming в отдельном slice.
