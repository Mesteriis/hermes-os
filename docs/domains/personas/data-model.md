# Personas — Data Model

This document defines the target logical model for the Personas domain. Active
PostgreSQL storage uses `personas` / `persona_*` table names with `persona_id`
identifier columns. Active API routes use `/api/v1/personas/*`; legacy
`/api/v1/persons/*` routes are retired.

## Model Principles

- Persona is the root entity.
- The Owner Persona is represented by `is_self: true`; there is no separate
  `UserProfile`.
- Identity is a collection of digital traces, not a single email column.
- Relationships are first-class records, not fields on a Persona.
- Memory records are evidence-backed.
- Timeline is a shared engine view over dated records.
- Dossier is a generated read model.
- AI observations are derived, confidence-scored and cited.

## Persona

```yaml
Persona:
  persona_id: string
  is_self: boolean
  persona_type: human | ai_agent | organization_proxy | system
  display_name: string
  lifecycle_status: active | archived | merged

  identity:
    primary_label:
    traces:

  communication:
    preferred_channels:
    patterns:

  memory:
    facts:
    knowledge:
    preferences:
    memory_cards:
    conflicts:

  timeline_view:
    events:

  relationships:
    outgoing:
    incoming:

  dossier_read_model:
    current:

  created_at:
  updated_at:
```

Rules:

- Exactly one Persona may have `is_self = true`.
- `persona_type` is required.
- Email, phone and provider usernames are identities, not root columns.
- Organization membership is a Relationship, not a free-text field.
- Favorites, watchlists and relationship health are UI/read-model concerns, not
  Persona identity.

## PersonaType

| Value | Meaning |
|---|---|
| `human` | A real person represented in memory. |
| `ai_agent` | HESTIA or another local/future AI agent represented in the graph. Registry-backed AI agent Personas use stable Persona IDs and compatibility email/display identities such as `hestia@sh-inc.ru`. |
| `organization_proxy` | An organization-like actor when it must participate as a Persona in relationships. |
| `system` | Local system actor used for provenance and automation attribution. |

## PersonaIdentity

Identity Resolution works over traces:

```yaml
PersonaIdentity:
  identity_id: string
  persona_id: string | null
  trace_type:
    - email
    - phone
    - telegram
    - whatsapp
    - github
    - linkedin
    - document_mention
    - message_participant
    - provider_handle
  value: string
  normalized_value: string
  provider: string
  source_ref: string
  confidence: number
  status: active | outdated | unreachable | blocked | disputed
  first_seen_at:
  last_verified_at:
  metadata:
```

Rules:

- Active exact traces should be unique per trace type and normalized value.
- Ambiguous traces create identity resolution candidates.
- Provider-specific identity must be preserved for replay and audit.
- A trace may exist before it is attached to a Persona; compatibility storage
  now supports unattached traces and explicit later assignment.

## IdentityResolutionCandidate

```yaml
IdentityResolutionCandidate:
  candidate_id: string
  candidate_kind:
    - merge_personas
    - attach_trace
    - split_persona
  left_persona_id:
  right_persona_id:
  identity_id:
  evidence_summary:
  evidence_refs:
  confidence:
  review_state: suggested | user_confirmed | user_rejected
  actor_persona_id:
  generated_at:
  reviewed_at:
```

Rules:

- Ambiguous merge/split decisions require review.
- Confirming a merge must preserve enough evidence to support a later split.
- AI may rank candidates, but it must not silently merge ambiguous Personas.

## Relationship

Relationships are primary domain records:

```yaml
Relationship:
  relationship_id: string
  source_persona_id: string
  target_persona_id: string
  relationship_type: string
  trust_score: number
  strength_score: number
  confidence: number
  source_refs:
  valid_from:
  valid_to:
  status: active | inactive | disputed
  metadata:
  created_at:
  updated_at:
```

Rules:

- `source_persona_id` and `target_persona_id` are required.
- `trust_score` and `strength_score` are relationship attributes, not Persona
  root attributes.
- Relationship types must be explicit and queryable.
- Relationship evidence must point to events, messages, documents or reviewed
  user input.

Example relationship types:

- `knows`
- `collaborates_with`
- `works_with`
- `reports_to`
- `represents`
- `assists`
- `owns`
- `member_of`
- `introduced`

The list above is illustrative; a future implementation should control valid
values through a typed domain registry or migration.

## PersonaMemory

Memory is split into durable, cited record types.

### PersonaFact

```yaml
PersonaFact:
  fact_id: string
  persona_id: string
  fact_type: string
  value: string
  source_refs:
  confidence: number
  last_verified_at:
  valid_from:
  valid_to:
  status: active | superseded | rejected
```

### PersonaKnowledgeItem

```yaml
PersonaKnowledgeItem:
  knowledge_id: string
  persona_id: string
  topic: string
  summary: string
  source_refs:
  confidence: number
  updated_at:
```

### PersonaPreference

```yaml
PersonaPreference:
  preference_id: string
  persona_id: string
  preference_type: string
  value: string
  source_refs:
  confidence: number
  last_verified_at:
```

### PersonaMemoryCard

```yaml
PersonaMemoryCard:
  memory_card_id: string
  persona_id: string
  title: string
  body: string
  importance: 1..10
  source_refs:
  confidence: number
  created_at:
  last_verified_at:
```

### PersonaKnowledgeConflict

```yaml
PersonaKnowledgeConflict:
  conflict_id: string
  persona_id: string
  field: string
  value_a: string
  value_b: string
  source_ref_a: string
  source_ref_b: string
  detected_at:
  resolved_at:
  resolution:
```

## Persona Dated Events

```yaml
PersonaDatedEvent:
  event_id: string
  persona_id: string
  relationship_id:
  event_type: string
  title: string
  description:
  occurred_at:
  source_refs:
  related_entity_refs:
  confidence: number
  metadata:
  created_at:
```

Dated events can describe first interaction, a project collaboration, an
obligation, an introduction, a conflict, a meeting, a document mention or a
system observation. They are not a substitute for Relationship records. The
Timeline Engine turns dated events into timeline views.

## PersonaCommunication

```yaml
PersonaCommunicationPattern:
  pattern_id: string
  persona_id: string
  channel:
  language:
  tone:
  verbosity:
  technical_depth:
  response_pattern:
  active_hours:
  active_days:
  source_refs:
  confidence:
  computed_at:
```

This replaces the old `CommunicationFingerprint` vocabulary. Patterns are
derived observations and may be recomputed from messages.

## PersonaDossier

```yaml
PersonaDossier:
  dossier_id: string
  persona_id: string
  summary:
  interests:
  projects:
  organizations:
  skills:
  communication_patterns:
  ai_observations:
  source_refs:
  generated_at:
```

Rules:

- Dossier is a read model.
- Dossier fields must cite source memory, relationships, messages, documents or
  graph records.
- AI observations must be labeled as observations, not facts.

## Compatibility Mapping

The current schema contains useful pieces but does not yet match the target
model.

| Current table/field | Target model | Migration note |
|---|---|---|
| `personas` | `Persona` projection | Active root Persona storage. |
| `personas.email_address` | `PersonaIdentity(trace_type=email)` | Root email is compatibility only. |
| `personas.person_type` | `Persona.persona_type` | Value set must become `human`, `ai_agent`, `organization_proxy`, `system`. |
| `personas.trust_score` | `Relationship.trust_score` | Compatibility cache only. Enrichment writes now materialize suggested Owner Persona -> Persona trust Relationships. |
| `personas.primary_role` | `Relationship.relationship_type` or memory fact | Do not model as Persona field. |
| `personas.organization_reference` | Relationship to organization proxy or organizations domain | Keep only as cached compatibility. |
| `personas.is_favorite` | `PersonaPreference(ui:favorite)` compatibility cache | Not domain identity. Writes now materialize a sourced UI preference. |
| `personas.notes` | `PersonaMemoryCard` | Compatibility cache only. Writes now materialize a sourced memory card. |
| `personas.health_status` | Risk/attention compatibility cache | Not source of truth. `PersonaRiskStore` writes now derive it from unresolved risks. |
| `personas.watchlist` | `PersonaPreference(ui:watchlist)` compatibility cache | Not domain identity. Writes now materialize a sourced UI preference. |
| `persona_identities` | `PersonaIdentity` | Compatibility schema supports account handles, `document_mention`, `message_participant`, `disputed` status and unattached trace assignment; Persona-native naming and review UI/API remain future work. |
| `persona_identity_candidates` | `IdentityResolutionCandidate` | Rename semantics from person/contact to Persona. |
| `persona_roles` | `Relationship` | Deprecated in target model. |
| `persona_interaction_contexts` | `PersonaPreference` interaction context compatibility | Renamed from legacy `person_personas`; compatibility writes materialize `interaction_context:*` preferences with source references. |
| `persona_facts` | `PersonaFact` | Persona-native memory fact storage; renamed from legacy `person_facts`. |
| `persona_memory_cards` | `PersonaMemoryCard` | Persona-native memory-card storage; evidence-backed semantics remain required. |
| `persona_preferences` | `PersonaPreference` | Persona-native preference storage; includes UI and interaction-context preferences. |
| `persona_snapshots` | Persona read-model snapshots | Keep only if used for diff/replay. |
| `persona_knowledge_conflicts` | `PersonaKnowledgeConflict` | Keep concept. |
| `relationship_events` | dated events consumed by Timeline Engine | Split from first-class Relationship records. |
| `enrichment_results` | Persona Intelligence observation candidates | Must be reviewed/cited. |
| `persona_expertise` | Persona skills/knowledge signals | Keep concept. |
| `persona_promises` | Obligation, commitment event or fact | Do not treat as CRM task tracking. |
| `persona_risks` | Evidence-backed AI/user observations | Avoid uncited risk labels. |

## Required Additions

Future implementation work needs explicit storage for:

- Owner Persona uniqueness (`is_self = true`).
- Persona type enum values.
- First-class Relationship records with `source_persona_id`,
  `target_persona_id`, `trust_score` and `strength_score`.
- Persona Dossier cache/read model with source references.
- Persona Intelligence observations with observation type, confidence and
  evidence.
- Digital traces from documents and messages, not only account handles.

## Required Removals From Domain Semantics

These concepts must not appear as target domain primitives:

- Legacy address-book row as the root durable entity.
- Address book as a separate domain object; it is a source/capability flag on a
  Persona projection, not a replacement for Persona.
- CRM profile.
- Address-book role as a Persona-owned primitive; organization/persona relationships
  own role context.
- Nested legacy address-book/person personas.
- Favorite/watchlist as identity.
- Relationship stored only as Persona fields.
- Trust score stored only on Persona.
- Email as required root identity.
