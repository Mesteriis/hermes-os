# V2 Memory Workflow Completion Design

## Purpose

Hermes Hub has the V1 local memory core, V2 graph core, project memory spine and project link review workflow. The remaining Version 2 roadmap functions are still represented mostly as architecture direction or static frontend surfaces:

- task candidates from messages and documents;
- contact merge/split through identity review;
- asynchronous document processing and OCR/extraction state.

This design packages those functions into three independent implementation slices. The package goal is to complete the review-first V2 memory workflows without introducing the Version 3 agent runtime as a hidden dependency.

## Current Context

Relevant current implementation:

- canonical events, event log APIs and projection runner exist;
- raw email records, canonical messages, contacts, documents and local mail blobs exist;
- graph nodes, graph edges and graph evidence are rebuildable PostgreSQL projections;
- project records, keyword-derived project links and event-backed project link reviews exist;
- the Svelte desktop shell has visible Tasks and AI Agents tabs, but task and agent data is still static presentation data;
- the Projects tab already has API-backed project data and a review workflow pattern.

Relevant ADRs:

- `ADR-0001 Event Sourcing as System Spine`;
- `ADR-0008 Knowledge Graph First`;
- `ADR-0015 Command Query Separation`;
- `ADR-0017 Document Processing Pipeline`;
- `ADR-0019 Contact Identity Resolution`;
- `ADR-0020 Task Candidate Lifecycle`;
- `ADR-0023 Rebuildable Projections`;
- `ADR-0031 Temporary Desktop Only UI Scope`;
- `ADR-0038 Local Event API Capability Token`;
- `ADR-0040 Local API Actor Identity`;
- `ADR-0045 Graph Core Projection`;
- `ADR-0047 Project Memory Spine`;
- `ADR-0048 Project Link Review Workflow`.

## Package Shape

Use one umbrella design and three separate implementation plans:

1. `task-candidate-review`
2. `contact-identity-review`
3. `document-processing-pipeline`

Each plan must be independently executable, testable and committable. Shared patterns should be reused, but implementation should not require all three slices to land together.

## Recommended Order

### 1. Task Candidate Review

Implement first because it reuses the freshest local pattern: candidate generation, source-backed evidence, protected review commands and UI review queue. It can use existing messages, documents and projects without requiring OCR or AI inference.

### 2. Contact Identity Review

Implement second because it needs the same candidate-review discipline, but carries higher data-integrity risk. The first implementation must avoid automatic ambiguous person merges and must preserve source provenance for email addresses and contacts.

### 3. Document Processing Pipeline

Implement third because it introduces asynchronous processing state, artifact boundaries and failure/retry behavior. The first implementation should establish the pipeline contract and local extraction state before adding heavyweight OCR providers.

## Shared Architectural Rules

All three slices follow the same constraints:

- local-first PostgreSQL remains the primary store;
- meaningful user decisions are represented as canonical events;
- read models and graph/search state are rebuildable projections;
- protected local APIs require bearer token plus `X-Hermes-Actor-Id`;
- AI output is not source of truth;
- private data is not sent to remote services;
- UI scope remains desktop-only under `ADR-0031`;
- implementation stays scoped and does not introduce a general agent runtime.

## Slice 1: Task Candidate Review

### Goal

Create a source-backed task candidate lifecycle for messages and documents with user confirmation/rejection before anything becomes an active task.

### Data Model

Add a task candidate read model with:

- stable `task_candidate_id`;
- `source_kind`: `message` or `document`;
- `source_id`;
- optional `project_id`;
- candidate title;
- optional due date text or parsed due timestamp;
- optional assignee label;
- confidence;
- review state: `suggested`, `user_confirmed`, `user_rejected`;
- evidence excerpt limited to safe snippets;
- event provenance and timestamps.

Confirmed task candidates become active task records only through explicit user review or a later policy-backed command. The first slice should not create calendar events, provider tasks or outbound writes.

### Candidate Generation

The first implementation uses deterministic rules only:

- message subject/body and document title/text can create candidates when they contain explicit action markers such as `Action`, `Follow up`, `Please`, or similar configured local rules;
- project association can reuse existing project active-link rules when a source is connected to a project;
- generated evidence must reference the source and must not expose entire message bodies through list APIs.

Local AI/Ollama extraction is intentionally deferred. The task candidate model should leave room for future extractor provenance without depending on it.

### API

Add protected read/write APIs:

- `GET /api/v2/task-candidates`;
- `PUT /api/v2/task-candidates/{task_candidate_id}/review`;
- `GET /api/v2/tasks`.

The review command accepts `command_id` and target review state. The command appends a canonical event and updates the task candidate/task read models transactionally.

### Frontend

Replace the static Tasks tab with local API data:

- candidate review queue;
- active confirmed tasks;
- source/project labels;
- confirm/reject/reset controls;
- empty/error/loading states.

No task provider integration, due-date automation or mobile UI is in scope.

## Slice 2: Contact Identity Review

### Goal

Create explicit identity candidates for possible contact merge/split decisions without allowing ambiguous automatic person merges.

### Data Model

Add identity candidate and review state tables:

- stable `identity_candidate_id`;
- candidate kind: `merge_contacts`, `attach_email_address`, `split_contact`;
- subject contact IDs and email address IDs;
- evidence summary;
- confidence;
- review state: `suggested`, `user_confirmed`, `user_rejected`;
- event provenance and timestamps.

Confirmed identity decisions should be durable and replayable. Rejected decisions must suppress the same candidate from reappearing unless source evidence changes meaningfully.

### Candidate Generation

The first implementation uses conservative deterministic signals:

- exact normalized display-name match across contacts;
- exact email local-part similarity only as low-confidence evidence;
- same provider account plus repeated co-occurrence as supporting evidence;
- no automatic merge for ambiguous candidates.

Graph projection may use confirmed decisions to connect contact/email nodes more accurately, but graph edges remain rebuildable projection state.

### API

Add protected APIs:

- `GET /api/v2/identity-candidates`;
- `PUT /api/v2/identity-candidates/{identity_candidate_id}/review`;
- `GET /api/v2/contacts/{contact_id}/identity`.

Review commands append canonical events and update identity review state. Public responses must avoid leaking private message bodies.

### Frontend

Add a compact identity review surface inside Contacts or Knowledge Graph context:

- candidate comparison rows;
- evidence summary;
- confirm/reject/reset actions;
- clear warning that suggested candidates are not applied automatically.

No broad contact management UI rewrite is in scope.

## Slice 3: Document Processing Pipeline

### Goal

Add an asynchronous document processing pipeline that tracks extraction/OCR states and stores derived artifacts without blocking document import.

### Data Model

Add processing jobs/artifacts:

- `document_processing_jobs` with document ID, step, status, attempts, error summary and timestamps;
- `document_artifacts` with artifact kind, source document ID, storage reference, content hash, text metadata and timestamps;
- processing status values: `queued`, `running`, `succeeded`, `failed`, `skipped`;
- scanner/extractor provenance for safety and rebuildability.

Document records remain source metadata. Extracted text, OCR output and derived artifacts are projection/artifact state, not the original source of truth.

### Pipeline

The first implementation should provide:

- enqueue on document import or explicit development command;
- worker-runner function that processes queued jobs in bounded batches;
- Markdown/plain-text extraction as the initial concrete processor;
- PDF metadata preservation with OCR marked `skipped` unless a real OCR backend is configured;
- failure recording with safe error messages and retry eligibility.

This slice should not add an external OCR dependency unless the implementation plan proves it is already available and practical in the repository environment.

### API

Add protected read APIs:

- `GET /api/v2/documents/{document_id}/processing`;
- `GET /api/v2/document-processing/jobs`;

Write/retry commands should be added only if they use the existing command boundary and audit pattern. Otherwise, development validation can use a CLI/Make target first.

### Frontend

Replace static document-processing UI copy with API-backed status:

- document processing badges;
- failed/skipped/succeeded states;
- artifact summaries;
- retry control only if a backend command exists.

No full document reader, OCR viewer or mobile UI is in scope.

## Cross-Slice Data Flow

1. Source records enter through existing import/provider/document paths.
2. Deterministic generators create candidates or processing jobs.
3. Users review candidates through protected commands.
4. Canonical events capture review decisions.
5. Read models and graph/search projections consume source records plus explicit decisions.
6. Frontend renders only API-backed state for these workflows.

## Error Handling

All protected APIs should:

- reject missing/invalid bearer token;
- reject missing actor ID for commands;
- validate `command_id` for idempotent writes;
- return stable error codes;
- avoid leaking SQL details, message bodies, document contents or secrets.

Processing jobs should persist failure summaries that are useful for local debugging without storing private document contents in errors.

## Validation Strategy

Each implementation plan must include:

- live PostgreSQL tests for schema/read-model behavior;
- command idempotency tests for review APIs;
- projection/rebuild tests where derived graph/search/read state changes;
- API auth tests for protected endpoints;
- frontend `pnpm --dir frontend check`;
- frontend `pnpm --dir frontend build` when UI changes;
- `make backend-validate` for backend-only work;
- `make validate` before reporting broad package completion.

## Non-Goals

- Version 3 agent runtime;
- remote AI model calls;
- automatic ambiguous contact merges;
- outbound task provider writes;
- calendar event creation;
- full OCR provider integration without a separate explicit decision;
- mobile UI;
- broad frontend component rewrite.

## Risks

- Task extraction can create noisy false positives. Mitigation: deterministic rules, review-first lifecycle and evidence snippets.
- Identity candidates can damage trust if applied too aggressively. Mitigation: suggestions are inactive until confirmed; ambiguous automatic merge is disallowed.
- Document processing can introduce heavy dependencies and slow validation. Mitigation: start with pipeline state and lightweight local processors, then add OCR as a separate decision.
- The Svelte page is already large. Mitigation: keep UI changes scoped; introduce component splits only when a plan can do so without changing behavior.
