# Organizations Data Model

## `organizations`

| Column | Type | Description |
|---|---|---|
| `organization_id` | TEXT PK | `org:v1:{nanos}` |
| `display_name` | TEXT NOT NULL | display label |
| `legal_name` | TEXT | legal name |
| `org_type` | TEXT | organization type |
| `status` | TEXT | lifecycle/status value |
| `country`, `city`, `address` | TEXT | location metadata |
| `website`, `industry`, `description` | TEXT | descriptive metadata |
| `primary_language`, `timezone` | TEXT | communication/context metadata |
| `tags` | JSONB | user/system tags |
| `org_metadata` | JSONB | structured metadata |
| `registration_number`, `vat`, `cif`, `nif`, `tax_id` | TEXT | legal identifiers |
| `communication_style`, `verbosity`, `formality` | TEXT | communication pattern hints |
| `secondary_languages` | JSONB | additional languages |
| `last_interaction_at`, `interaction_count` | | derived interaction hints |

## Relationship And Trust Boundary

Trust, risk, watchlist and health-like values are attention or engine outputs.
They must not be treated as Organization identity. Organization relationships to
Personas, Projects and other Organizations should be modeled as relationships
with provenance.

## Other Tables

The current schema includes identity, alias, domain, department, relationship,
memory, required document, timeline/workflow, portal, procedure, playbook,
finance, enrichment and risk/alert tables.

Full implementation schema lives in migrations `0038`-`0043`.
