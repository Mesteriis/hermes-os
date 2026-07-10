# ADR-0084 Persona Intelligence System

Status: Proposed

Supersedes:

- ADR-0019 Persona Identity Resolution
- ADR-0059 Person Communication DNA and Personas

Clarifies:

- ADR-0057 Person Memory and Provenance
- ADR-0058 Person Enrichment Engine
- ADR-0060 Person Timeline and Graph Integration
- ADR-0074 Persona Multi-Channel Identity Compatibility

## Context

Hermes Hub is a local-first Personal Memory System. The Personas domain was
previously documented as a partially renamed contact system: contacts became
persons, but the model still used CRM-shaped concepts such as contact merge,
roles, nested personas, favorites, watchlists, health status, fingerprints,
analytics and investigator flows.

The domain direction has changed. Hermes does not treat people as contacts.
Hermes treats subjects as Personas.

A Persona is a durable digital representation of a subject that can accumulate
identity, relationships, communication context, memory, timeline, knowledge and
a generated dossier.

## Decision

Use Persona Intelligence as the target architecture for the Personas domain.

The root domain entity is:

```yaml
Persona:
  id:
  is_self:
  persona_type:

  identity:
  communication:
  memory:
  timeline:
  relationships:
  dossier:
```

Exactly one Persona represents the owner:

```yaml
Persona:
  is_self: true
```

There is no separate `UserProfile` or Self domain. Local agents act through the
Owner Persona when operating for the system owner.

Supported Persona types:

```yaml
PersonaType:
  human
  ai_agent
  organization_proxy
  system
```

Relationships are first-class records:

```yaml
Relationship:
  source_persona:
  target_persona:
  type:
  trust_score:
  strength_score:
```

Trust and relationship strength must not be stored only as fields on a Persona.
Roles, organization links, relationship health and attention state are modeled
as Relationships, Timeline events, memory records or read models.

Persona memory contains facts, knowledge, preferences, memory cards and
conflicts with provenance, confidence and verification metadata. AI output may
produce observations and candidates, but it is not source of truth without
reviewed, cited storage.

Each Persona has a generated Dossier read model:

```yaml
Dossier:
  summary:
  interests:
  projects:
  organizations:
  skills:
  communication_patterns:
  ai_observations:
```

`fingerprint`, `communication profile`, `trust`, `analytics` and `investigator`
are consolidated under Persona Intelligence.

Identity Resolution operates on digital traces of a Persona:

- email;
- phone;
- Telegram;
- WhatsApp;
- GitHub;
- LinkedIn;
- documents;
- messages;
- provider-specific handles.

Ambiguous identity resolution remains reviewable. This preserves the safety
property from ADR-0019 while replacing its Contact framing.

ADR-0074 remains the implementation compatibility contract for existing
email-derived `person_id` values and internal `person_id` storage columns until
a separate physical identifier migration ADR is accepted. Active routes and
logical storage now use Persona terminology (`/personas`, `personas`,
`persona_*`); remaining `person_id` columns are a physical compatibility detail,
not a current domain name.

## Consequences

Positive:

- The domain aligns with Hermes as a Personal Memory System.
- People, agents, organization proxies and system actors can exist in one graph.
- Relationships become queryable, provenance-backed records.
- The Owner Persona gives agents a clear subject boundary.
- Dossiers become derived read models with citations instead of manually edited
  persona summaries.
- Identity resolution can unify communication and document traces without
  pretending they are address-book fields.

Migration constraints:

- Existing email-derived `person_id` values remain compatibility identifiers
  until a dedicated opaque Persona identifier migration is designed.
- Internal `person_id` primary/FK columns remain compatibility storage details
  even where table and API names are Persona-native.
- `person_personas` has been renamed to `persona_interaction_contexts`;
  compatibility writes now materialize interaction-context values into Persona
  Preferences.
- Existing health, watchlist, role and trust fields must be reclassified before
  deeper implementation work.
- UI and backend code will need a future migration plan to avoid breaking
  current projections when physical identifiers are renamed.

## Non-Goals

- Immediate physical identifier migration away from internal `person_id`
  columns.
- Removing current compatibility aliases for historical event payloads and
  request bodies.
- Fine-tuning models on private Persona data.
- Turning public enrichment into active OSINT or scraping beyond approved
  provider boundaries.

## Required Follow-Up

- Design a physical identifier migration ADR if implementation moves away from
  compatibility `person_id` columns and email-derived `person:v1:email:*`
  values.
- Add first-class Relationship records.
- Finish Owner Persona usage across agents, UI context assembly and user-owned
  actions.
- Finish PersonaType-aware graph and workflow semantics.
- Reframe existing intelligence, analytics and investigator code as Persona
  Intelligence services and read models.
