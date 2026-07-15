# Refactoring Plan: handlers.rs Decomposition

Status: implementation decomposition note.

This document predates the foundation terminology cleanup and names historical
handler groups/routes. Treat `health`, `watchlist`, `promises`, `fingerprint`
and similar names below as compatibility labels until the code is renamed
through an explicit implementation task. The canonical domain model is defined
in `../foundation/`; the active people domain is Personas.

For the product-model migration from current modules and routes to
Communications, Persona, shared Engines, Decisions, Obligations and Polygraph,
use `../refactoring/implementation-alignment-plan.md`. This file remains a
handler decomposition note, not the product-domain migration plan.

## Goal

Eliminate `app/handlers.rs` (9019 lines). HTTP endpoint → one file under domain `api/`. DTOs in `dto.rs`, one per subdomain. Business logic stays — only HTTP layer moves.

## Non-goals

- Repository traits (Rule 12) — deferred
- Pure domain without framework types (Rule 13) — deferred
- Per-domain typed errors — deferred

---

## Phase 0: App Foundation (3 files)

### 0.1 `app/router.rs`
Move `build_router()` — pure route chain, no logic.

### 0.2 `app/auth.rs`
`verify_local_api_capability`, `local_api_actor`, `LocalApiActor`.

### 0.3 `app/shared.rs`
Shared store accessors: `event_store`, `message_store`, `api_audit_log`, etc.

---

## Phase 1: Small Domains (~30 files)

### 1.1 Graph ✅, 1.2 Projects ✅, 1.3 Documents ✅, 1.4 Settings ✅

### 1.5 AI (7 endpoints)
```
ai/api/
├── dto.rs
├── get_status.rs
├── list_agents.rs
├── list_runs.rs
├── get_run.rs
├── submit_answer.rs
├── refresh_task_candidates.rs
└── meeting_prep.rs
```

### 1.6 Integrations (12 endpoints)
```
integrations/
├── telegram/api/  (3 handlers + dto.rs)
├── whatsapp/api/  (3 handlers + dto.rs)
├── calls/api/     (3 handlers + dto.rs)
└── policies/api.rs (5 handlers + dto.rs)
```

### 1.7 Platform (5 endpoints)
```
platform/api/
├── dto.rs
├── audit_events.rs
├── post_event.rs
├── get_event.rs
├── status.rs
└── capabilities.rs
```

### 1.8 Email Setup (4 endpoints)
```
domains/communications/api/account_setup/
├── dto.rs
├── start_gmail_oauth.rs
├── complete_gmail_oauth.rs
├── gmail_callback.rs
└── setup_imap.rs
```

---

## Phase 2: Large Domains (~120 files)

### 2.1 Personas routes (45 handlers)
```
domains/personas/api/
├── dto.rs              # shared DTOs
├── list.rs             # GET /personas
├── get.rs              # GET /personas/:id
├── search.rs           # GET /personas/search
├── identity/           # 9 handlers + dto.rs
├── enrichment/         # 3 handlers + dto.rs
├── expertise/          # 2 handlers + dto.rs
├── memory/             # 6 handlers + dto.rs
├── timeline/           # 2 handlers + dto.rs
├── analytics/          # 4 handlers + dto.rs
└── watchlist/          # 2 handlers + dto.rs
```
Plus compatibility labels: fingerprint, favorite, notes, personas, health,
risks, promises, investigate, dossier, meeting-prep. Map these to canonical
Persona Intelligence, attention/risk read models, Obligations, Dossier and
context preparation before any implementation rename.

### 2.2 Calendar (47 handlers)
```
domains/calendar/api/
├── accounts/           # 8 handlers + dto.rs
├── events/             # 10 handlers + dto.rs
├── meetings/           # 15 handlers + dto.rs
├── scheduling/         # 3 handlers + dto.rs
├── analytics/          # 10 handlers + dto.rs
├── rules/              # 4 handlers + dto.rs
└── reminders/          # 3 handlers + dto.rs
```

### 2.3 Organizations (28 handlers)
```
domains/organizations/api/
├── dto.rs
├── list.rs, create.rs, get.rs, update.rs, search.rs, archive.rs
├── identities/         # 5 handlers + dto.rs
├── structure/          # 5 handlers + dto.rs
├── resources/          # 4 handlers + dto.rs
└── intelligence/       # 12 handlers + dto.rs
```

### 2.4 Tasks (26 handlers)
```
domains/tasks/api/
├── dto.rs
├── list.rs, create.rs, get.rs, update.rs, archive.rs, update_status.rs
├── context/            # 8 handlers + dto.rs
├── intelligence/       # 7 handlers + dto.rs
├── providers/          # 2 handlers + dto.rs
├── rules/              # 4 handlers + dto.rs
├── analytics/          # 3 handlers + dto.rs
└── candidates/         # 2 handlers + dto.rs
```

### 2.5 Mail V1 (50 handlers)
```
domains/communications/api/v1/
├── dto.rs
├── messages/           # 10 handlers + dto.rs
├── threads/            # 2 handlers + dto.rs
├── compose/            # 10 handlers + dto.rs
├── intelligence/       # 10 handlers + dto.rs
├── security/           # 6 handlers + dto.rs
└── admin/              # 17 handlers + dto.rs
```

---

## Phase 3: Rename core.rs → domain/*.rs (Rule 7)

| Current | New |
|---------|-----|
| `calendar/core.rs` | `calendar/domain/calendar.rs` |
| `personas/core.rs` | `personas/domain/persona.rs` |
| `tasks/core.rs` | `tasks/domain/task.rs` |
| `organizations/core.rs` | `organizations/domain/organization.rs` |
| `projects/core.rs` | `projects/domain/project.rs` |
| `documents/core.rs` | `documents/domain/document.rs` |
| `graph/core.rs` | `graph/domain/node.rs` |
| `mail/core.rs` | `mail/domain/message.rs` (split if >600 lines) |

---

## Phase 4: Cleanup

1. Delete `app/handlers.rs`
2. Update `app/mod.rs`
3. Run full validation

---

## Validation Gate (after each phase)

```sh
cargo check && cargo fmt && cargo clippy --all-targets --all-features -- -D warnings && cargo test --lib
```

---

## Handler Template

Every handler follows this exact shape — 10-30 lines:

```rust
// domains/personas/api/search.rs
// GET /api/v1/personas/search

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use serde::Deserialize;

use super::dto::PersonaSearchResponse;
use crate::app::auth::verify_local_api_capability;
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::domains::personas::enrichment::PersonaEnrichmentStore;

#[derive(Deserialize)]
struct PersonaSearchQuery {
    q: String,
    limit: Option<i64>,
}

pub async fn search_personas(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PersonaSearchQuery>,
) -> Result<Json<PersonaSearchResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;

    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let store = crate::app::api_support::app_store::<PersonaEnrichmentStore>(pool);
    let items = store
        .search_personas(&query.q, query.limit.unwrap_or(20))
        .await?;

    Ok(Json(PersonaSearchResponse { items }))
}
```

## DTO Template

```rust
// domains/personas/api/dto.rs
// Shared DTOs for personas list/search/get

use serde::Serialize;
use crate::domains::personas::enrichment::EnrichedPersona;

#[derive(Serialize)]
pub struct PersonaListResponse {
    pub items: Vec<EnrichedPersona>,
}

#[derive(Serialize)]
pub struct PersonaSearchResponse {
    pub items: Vec<EnrichedPersona>,
}
```
