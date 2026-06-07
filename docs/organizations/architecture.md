# Organizations — Архитектура

## Модули

```
organizations/
├── organizations.rs              Ядро: Organization, OrganizationStore
├── organization_core.rs          Identities, aliases, domains, departments, contacts, related orgs
├── organization_memory.rs        Facts, memory cards, preferences, required docs, memory decay
├── organization_workflows.rs     Timeline, templates, portals, procedures, playbooks
├── organization_finance.rs       Financial info, contracts, compliance, services, products
├── organization_enrichment.rs    Enrichment results (pending/applied/rejected)
├── organization_health.rs        Health status, risks, watchlist
└── organization_investigator.rs  Dossier, brief, context pack
```

## Потоки данных

### Создание организации из email

```
incoming email (domain)
  → email_sync_pipeline.rs
    → OrganizationStore::create()
    → organization_domains::add()
    → organization_contact_links::link() (per sender)
```

### Identity Resolution

```
organizations (similar names/domains/VAT)
  → merge/split candidates
    → user confirm/reject
```

### Enrichment

```
external sources (VIES, GitHub, LinkedIn)
  → EnrichmentProvider::enrich()
    → organization_enrichment_results (pending)
      → user apply/reject
        → organization_facts, organization_identities
```

## Зависимости

Все модули используют `PgPool` через DI (конструктор Store). Связи:
- `organizations.rs` ← все остальные модули
- `organization_investigator.rs` ← `organizations.rs` + `organization_health.rs`

## ADR

| ADR | Тема |
|---|---|
| 0061 | Organization as first-class entity |
| 0062 | Identity and resolution |
| 0063 | Passive OSINT boundary |
| 0064 | Memory and provenance |
| 0065 | Portals, procedures, playbooks |
| 0066 | Graph integration |
