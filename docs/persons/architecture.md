# Persons — Архитектура

## Модули

```
persons/
├── persons.rs                    Ядро: Person, PersonProjectionStore
├── person_core.rs                Мультиканальные identity, roles, personas
├── person_enrichment.rs          EnrichedPerson, fingerprint, избранное, заметки
├── person_intelligence.rs        CommunicationFingerprint (heuristic + LLM)
├── person_identity.rs            Merge/split candidates, review workflow
├── person_memory.rs              Facts, memory cards, snapshots, timeline, history diff, decay
├── person_enrichment_engine.rs   Enrichment results из внешних источников
├── person_expertise.rs           Навыки, поиск по навыкам
├── person_trust.rs               Обещания, риски
├── person_health.rs              Health monitoring, watchlist
├── person_investigator.rs        AI-ассистент: досье, meeting prep
├── person_analytics.rs           Relationship score, heatmap, intelligence score
└── person_export.rs              Markdown/JSON export
```

## Потоки данных

### Создание персоны из письма

```
incoming email
  → email_sync_pipeline.rs
    → PersonProjectionStore::upsert_email_person()
      → persons + person_identities (backfill)
```

### Identity Resolution

```
persons (same display_name)
  → PersonIdentityStore::refresh_candidates()
    → person_identity_candidates (merge_persons)
      → user review (confirm/reject)
        → confirmed merge materializes split_person candidate
```

### Communication Fingerprint

```
communication_messages
  → PersonIntelligenceService::heuristic_fingerprint()
    → CommunicationFingerprint
      → PersonEnrichmentStore::enrich_person()
        → persons (language, tone, trust_score, writing_style)
```

### Enrichment Engine

```
external sources (GitHub, LinkedIn, web)
  → EnrichmentResultStore::upsert()
    → enrichment_results (pending)
      → user review (apply/reject)
        → person_facts, person_expertise, person_memory_cards
```

### Dossier Assembly

```
PersonInvestigator::assemble_dossier()
  → PersonEnrichmentStore (profile)
  → PersonFactStore (facts)
  → PersonMemoryCardStore (memory cards)
  → RelationshipEventStore (timeline)
  → PersonDossier
    → PersonExportService (Markdown/JSON)
```

### Memory Decay

```
scheduled task
  → PersonFactStore::decay_unverified(threshold_days)
    → person_facts (confidence *= 0.5 for stale facts)
```

## Зависимости между модулями

```
persons.rs ← person_enrichment.rs ← person_intelligence.rs
persons.rs ← person_core.rs
persons.rs ← person_identity.rs
persons.rs ← person_memory.rs
persons.rs ← person_enrichment_engine.rs
persons.rs ← person_expertise.rs
persons.rs ← person_trust.rs
persons.rs ← person_health.rs
person_enrichment.rs + person_memory.rs + person_trust.rs ← person_investigator.rs
person_investigator.rs ← person_export.rs
persons.rs ← person_analytics.rs
```

Все модули используют `PgPool` через внедрение зависимостей (конструктор Store).

## Внешние связи

| Модуль persons | Внешний модуль | Связь |
|---|---|---|
| `person_intelligence.rs` | `ollama.rs` | LLM-based fingerprint |
| `person_intelligence.rs` | `email_multilingual.rs` | Детекция языка |
| `person_identity.rs` | `event_log.rs` | Event sourcing для review |
| `persons.rs` | `email_sync_pipeline.rs` | Автосоздание из писем |
| `persons.rs` | `graph_projection.rs` | Graph nodes для персон |
| `persons.rs` | `projects.rs` | Связь с проектами |

## ADR

| ADR | Тема |
|---|---|
| [0019](../adr/ADR-0019-contact-identity-resolution.md) | Identity resolution (частично superseded) |
| [0056](../adr/ADR-0056-person-multi-channel-identity-model.md) | Multi-channel identity model |
| [0057](../adr/ADR-0057-person-memory-and-provenance.md) | Memory and provenance system |
| [0058](../adr/ADR-0058-person-enrichment-engine.md) | Enrichment engine boundary |
| [0059](../adr/ADR-0059-person-communication-dna.md) | Communication DNA and personas |
| [0060](../adr/ADR-0060-person-timeline-and-graph-integration.md) | Timeline and graph integration |
