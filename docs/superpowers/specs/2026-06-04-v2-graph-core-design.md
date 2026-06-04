# V2 Graph Core Design

## Purpose

Version 2 starts with a graph-first core. The goal is to make Hermes Hub's knowledge graph a real backend projection before expanding the dashboard graph UI, document linking, identity resolution, task candidates or AI workflows.

The first V2 slice builds a deterministic, read-only graph from existing V1 data:

- contacts
- communication messages
- imported documents

This keeps the graph grounded in already persisted evidence and avoids introducing fuzzy identity merging, OCR, AI extraction or project inference before there is a reliable graph substrate.

## Relevant ADRs

- `ADR-0008 Knowledge Graph First`: relationships are durable records with provenance and confidence.
- `ADR-0017 Document Processing Pipeline`: richer document extraction is asynchronous and not part of the first graph slice.
- `ADR-0019 Contact Identity Resolution`: ambiguous merges require review; automatic fuzzy merge is out of scope.
- `ADR-0020 Task Candidate Lifecycle`: task extraction creates candidates that require confirmation; task candidates are not part of the first graph projection.
- `ADR-0023 Rebuildable Projections`: graph state is derived and must be rebuildable from canonical/raw storage.
- `ADR-0031 Temporary Desktop Only UI Scope`: V2 graph UI remains desktop/laptop only.
- `ADR-0038`, `ADR-0039`, `ADR-0040`: local graph APIs require local API token auth, actor identity and audit coverage when appropriate.

## Non-Goals

The first V2 graph slice does not implement:

- fuzzy identity merge
- contact merge/split workflows
- OCR
- entity extraction from document text
- task candidate extraction
- AI summaries
- graph editing
- GraphQL
- a separate graph database
- mobile graph UI

## Recommended Approach

Use PostgreSQL relational graph tables:

- `graph_nodes`
- `graph_edges`
- `graph_evidence`

PostgreSQL remains the only persistence system for V2 graph core. The graph tables are a rebuildable projection, not source of truth. Source of truth remains existing V1 storage:

- `contacts`
- `communication_messages`
- `documents`
- later, event log and document artifacts

This matches the current Rust, SQLx, PostgreSQL and migration pattern, keeps operational risk low and allows live PostgreSQL smoke tests.

## Data Model

### Graph Nodes

`graph_nodes` stores normalized graph entities.

Fields:

- `node_id TEXT PRIMARY KEY`
- `node_kind TEXT NOT NULL`
- `stable_key TEXT NOT NULL`
- `label TEXT NOT NULL`
- `properties JSONB NOT NULL DEFAULT '{}'::jsonb`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`

Initial `node_kind` values:

- `person`
- `email_address`
- `message`
- `document`

Reserved later values:

- `project`
- `organization`
- `task_candidate`

Rules:

- Node identity is deterministic and idempotent.
- `node_kind + stable_key` is unique.
- `properties` stores display metadata only; it must not store secrets.
- A node can exist without edges.

Initial deterministic IDs:

- person: `graph:node:v1:person:<contacts.contact_id>`
- email address: `graph:node:v1:email:<normalized_email>`
- message: `graph:node:v1:message:<communication_messages.message_id>`
- document: `graph:node:v1:document:<documents.document_id>`

### Graph Edges

`graph_edges` stores first-class relationships.

Fields:

- `edge_id TEXT PRIMARY KEY`
- `source_node_id TEXT NOT NULL REFERENCES graph_nodes(node_id)`
- `target_node_id TEXT NOT NULL REFERENCES graph_nodes(node_id)`
- `relationship_type TEXT NOT NULL`
- `confidence NUMERIC(5,4) NOT NULL`
- `review_state TEXT NOT NULL`
- `properties JSONB NOT NULL DEFAULT '{}'::jsonb`
- `valid_from TIMESTAMPTZ`
- `valid_to TIMESTAMPTZ`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`

Initial `relationship_type` values:

- `person_has_email_address`
- `person_sent_message`
- `person_received_message`
- `email_address_sent_message`
- `email_address_received_message`

Reserved later values:

- `message_related_to_document`
- `message_related_to_project`
- `document_related_to_project`
- `document_mentions_person`
- `task_created_from_message`
- `task_created_from_document`

Initial `review_state` values:

- `system_accepted`
- `suggested`
- `user_confirmed`
- `user_rejected`

Rules:

- First-slice deterministic edges use `review_state = 'system_accepted'`.
- `confidence` must be between `0.0` and `1.0`.
- Deterministic V1-derived edges use confidence `1.0`.
- An edge must have evidence unless it is explicitly user-created in a later slice.
- `source_node_id + target_node_id + relationship_type` is unique for active first-slice edges.

### Graph Evidence

`graph_evidence` links relationships to source evidence.

Fields:

- `evidence_id TEXT PRIMARY KEY`
- `edge_id TEXT NOT NULL REFERENCES graph_edges(edge_id)`
- `source_kind TEXT NOT NULL`
- `source_id TEXT NOT NULL`
- `excerpt TEXT`
- `metadata JSONB NOT NULL DEFAULT '{}'::jsonb`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`

Initial `source_kind` values:

- `contact`
- `message`
- `document`
- `raw_record`

Reserved later values:

- `event`
- `document_artifact`
- `projection_run`
- `agent_run`
- `manual_action`

Rules:

- Evidence is separate from edge properties so the UI and APIs can expose provenance without unpacking arbitrary JSON.
- Evidence can include short excerpts but must not include secrets.
- Multiple evidence records may support one edge.

## Projection Rules

The first graph projection reads already materialized V1 tables. It does not parse raw MIME, infer entities, OCR documents or call AI.

### Contacts

For each row in `contacts`:

1. Create a `person` node:
   - stable key: `contacts.contact_id`
   - label: `contacts.display_name`
   - properties: `{ "email_address": contacts.email_address }`

2. Create an `email_address` node:
   - stable key: normalized `contacts.email_address`
   - label: normalized email address
   - properties: `{}`

3. Create `person_has_email_address` edge:
   - source: person node
   - target: email address node
   - confidence: `1.0`
   - review state: `system_accepted`
   - evidence: source kind `contact`, source id `contacts.contact_id`

### Messages

For each row in `communication_messages`:

1. Create a `message` node:
   - stable key: `communication_messages.message_id`
   - label: message subject
   - properties:
     - `account_id`
     - `provider_record_id`
     - `occurred_at`

2. Resolve sender through exact email match:
   - if a contact exists for sender email, use its person node
   - otherwise create only an email address node for sender

3. Create `person_sent_message` when the sender maps to a contact, otherwise create `email_address_sent_message`.

For the first slice, sender/recipient message edges use email address nodes as stable endpoints when no person exists. This avoids hidden person creation with weak identity.

Initial edge endpoint rules:

- If exact contact exists: `person_sent_message` / `person_received_message`
- If no exact contact exists: create `email_address` node and use `email_address_sent_message` / `email_address_received_message`

Evidence:

- source kind: `message`
- source id: `communication_messages.message_id`
- excerpt: subject or a short body prefix
- metadata includes `raw_record_id` and `provider_record_id`

### Documents

For each row in `documents`:

1. Create a `document` node:
   - stable key: `documents.document_id`
   - label: `documents.title`
   - properties:
     - `document_kind`
     - `source_fingerprint`
     - `imported_at`

The first slice does not create document-person or document-project edges because entity extraction is not implemented yet.

## Read APIs

All V2 graph endpoints are read-only and protected by the existing local API token and actor header.

### `GET /api/v2/graph/summary`

Returns:

- node counts grouped by `node_kind`
- edge counts grouped by `relationship_type`
- total evidence count
- latest graph projection timestamp when available
- whether graph tables are empty

### `GET /api/v2/graph/neighborhood?node_id=<id>&depth=1`

Returns:

- selected node
- neighboring nodes
- edges touching the selected node
- evidence summaries for returned edges

Initial constraints:

- `depth` supports only `1`
- missing node returns `404`
- unsupported depth returns `400`
- results are capped to a conservative limit to protect dashboard usage

### `GET /api/v2/graph/search?q=<query>&limit=20`

Returns matching nodes by:

- `label ILIKE`
- `stable_key ILIKE`

Initial constraints:

- no Tantivy dependency in the first slice
- default limit: `20`
- max limit: `50`
- empty query returns `400`

## UI Behavior

The existing Knowledge Graph dashboard card must stop depending on hardcoded mock graph data once the V2 graph API exists.

First UI behavior:

- dashboard fetches `/api/v2/graph/summary`
- if graph has data, show real counts and latest node samples
- if graph is empty, show an empty graph state
- `Explore Graph` opens a read-only desktop drawer or panel
- graph explorer supports search and selecting a node neighborhood
- merge/split, OCR links, task candidates and graph editing remain disabled

The UI remains desktop/laptop only while `ADR-0031` is active.

## Validation Gates

V2 graph core is complete when:

- migration creates graph tables and constraints
- graph store upserts nodes idempotently
- graph store upserts edges idempotently
- graph store requires evidence for system-created edges
- projection from contacts, messages and documents is idempotent
- exact email identity rules do not create fuzzy person merges
- `GET /api/v2/graph/summary` passes auth and response tests
- `GET /api/v2/graph/neighborhood` passes auth, 404, unsupported depth and happy-path tests
- `GET /api/v2/graph/search` passes auth, empty query and happy-path tests
- `make backend-graph-smoke-dev` passes against live PostgreSQL
- `make validate` includes the graph smoke target
- frontend `pnpm check` and `pnpm build` pass after graph UI wiring

## Risks And Controls

### False Identity Merge

Risk: the graph incorrectly merges people who share ambiguous names or weak identifiers.

Control: first slice uses exact contact email matches only and creates explicit email address nodes when no contact exists.

### Graph Schema Churn

Risk: relationship types change as V2 grows.

Control: use generic graph tables with constrained initial type values and JSONB display metadata. Add new relationship types through migrations when needed.

### Missing Provenance

Risk: graph edges become unverifiable.

Control: system-created edges require `graph_evidence` records.

### UI Complexity

Risk: graph UI becomes too broad too early.

Control: first UI is read-only summary/search/neighborhood. Editing and merge workflows are later V2 tasks.

### Projection Replay Complexity

Risk: graph rebuilds diverge from source state.

Control: graph is a rebuildable projection from V1 tables. The first implementation can use focused idempotent projection functions before adding a full replay runner.

## Implementation Sequence

1. Create `ADR-0045 Graph Core Projection` before schema work.
2. Add PostgreSQL graph migration.
3. Implement Rust graph store.
4. Implement graph projection from contacts, messages and documents.
5. Add graph read API.
6. Add Makefile smoke target and validation wiring.
7. Wire dashboard Knowledge Graph to real V2 API data.
8. Update roadmap/checklist docs for V2 graph core status.

## Open Questions

There are no blocking open questions for the first V2 graph slice. Later V2 work must separately decide:

- identity merge/split UX
- project detection rules
- document entity extraction engine
- task candidate extraction rules
- graph editing policy
