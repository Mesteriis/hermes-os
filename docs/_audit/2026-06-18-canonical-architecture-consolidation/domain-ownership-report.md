# Domain Ownership Report

Date: 2026-06-18

Scope: bounded context audit only. No code or schema changes were made.

## Summary

The current canonical architecture supports these durable domains:

- Personas;
- Organizations;
- Communications;
- Documents;
- Projects;
- Tasks;
- Calendar/Events;
- Relationships;
- Decisions;
- Obligations;
- Knowledge Graph;
- Agents.

These concepts are not accepted as durable domains today:

- Email;
- Telegram;
- WhatsApp;
- Notes;
- Radar;
- Timeline;
- generic Observations;
- generic Knowledge silo.

## Ownership Matrix

| Candidate | Audit classification | Owns | Does not own | Data owner | Allowed links |
|---|---|---|---|---|---|
| Personas | Domain | Persona identity, Owner Persona, identity traces, Persona memory anchors, dossiers. | Provider messages, Organization lifecycle, Task lifecycle. | Personas domain, currently compatibility `persons` implementation. | Communications participants, Organization membership, Relationships, Projects, Decisions, Obligations, Events. |
| Organizations | Domain | Organization identity, domains, aliases, portals, procedures, playbooks, organization memory. | Persona identity, Project lifecycle, provider accounts. | Organizations domain. | Personas, Projects, Communications, Documents, Decisions, Obligations, Relationships. |
| Communications | Domain | Conversations, messages, participants as observed, channel accounts, communication attachments, delivery/draft state. | Persona truth, Task lifecycle, Decision truth, Obligation truth, global Memory. | Communications domain, currently mail-heavy compatibility storage plus channel integrations. | Personas, Organizations, Projects, Documents, Tasks, Decisions, Obligations, Events, Relationships. |
| Documents | Domain | Document artifacts, versions, extraction, metadata, document evidence. | Generic Knowledge truth, Task status, provider message lifecycle. | Documents domain. | Communications attachments, Projects, Decisions, Obligations, Personas, Organizations, Knowledge/Memory. |
| Notes | Not domain today | Document-like capture artifact if retained. | Independent lifecycle or source-of-truth memory. | Documents domain until an ADR says otherwise. | Documents, Projects, Memory context. |
| Projects | Domain | Bounded work contexts, project state, links and project context. | Organization identity, Task lifecycle, Decision truth. | Projects domain. | Tasks, Documents, Communications, Personas, Organizations, Decisions, Obligations, Relationships. |
| Tasks | Domain | Actionable work items, status lifecycle, local overlay, evidence, provider overlays. | Obligations as commitments, every follow-up, provider delivery. | Tasks domain. | Obligations, Communications, Projects, Documents, Calendar Events, Personas, Organizations. |
| Calendar | Domain as Calendar/Events | Scheduled events, meetings, attendees, calendar source identity, event evidence. | Global Timeline Engine, Decision/Obligation truth. | Calendar/Events domain. | Personas, Organizations, Projects, Tasks, Documents, Decisions, Obligations, Communications. |
| Knowledge | Emergent layer, not a silo domain today | Reviewed understanding with provenance after ownership policy is defined. | Documents, Relationships, Memory, Decisions or Obligations duplicated into a generic bucket. | Open. Requires future ADR for generic Knowledge Item storage. | Graph, Memory, Relationships, source evidence, domain records. |
| Radar | Workflow/read model, not domain today | Candidate aggregation and review ergonomics if accepted later. | Task lifecycle, accepted Memory, source truth, promotion target truth. | Producing candidate/observation owners. | Tasks, Personas, Organizations, Documents, Relationships, Decisions, Obligations, Memory. |
| Agents | Domain | Agent identity, run records, capability integration, proposed actions, audit. | Domain truth, credentials, Memory truth. | Agents domain plus Persona representation for AI agents. | Owner Persona, domain APIs, engines, audit, review queues. |
| Graph | Domain/projection substrate | Graph nodes, edges, evidence for traversal and relationship-aware retrieval. | Relationship semantics when first-class Relationship records exist, provider sync, binary storage. | Graph domain/projection; Relationships domain owns first-class relationship semantics. | All durable entity types with source evidence. |

## Boundary Conclusions

### Personas

Personas replace Contact/CRM framing. Compatibility names such as `persons` and
`person_id` remain implementation facts. They should not drive new product
language.

### Communications

Email, Telegram and WhatsApp must be channel subdomains under Communications.
Separate UI surfaces are acceptable. Separate durable Memory, Task, Decision or
Persona ownership is not.

### Documents And Notes

Notes are document-like capture artifacts unless a future ADR defines a durable
Notes lifecycle. Attachments belong to Communications until explicitly promoted
or linked as Documents.

### Tasks And Obligations

Tasks are executable actions. Obligations are commitments or duties with
evidence. One may lead to the other, but neither owns the other.

### Decisions

Projects, meetings and communications can produce decision evidence. Decisions
own durable choice and rationale.

### Knowledge

Knowledge is currently a cross-domain memory layer. Creating a generic
Knowledge domain before defining storage ownership would duplicate Documents,
Graph, Relationships, Decisions, Obligations and Memory.

### Radar

Radar is a candidate intake/review concept. It becomes a domain only if a
durable Signal lifecycle is proven and cannot live in existing candidates,
observations or engine outputs.

## Ownership Risks

| Risk | Why it matters | Required next step |
|---|---|---|
| Knowledge as a generic silo | Could duplicate every other domain. | Write Knowledge Item RFC before implementation. |
| Radar as hidden task tracker | Could steal ownership from Tasks, Obligations and review workflows. | Keep Radar as workflow/read model until Signal taxonomy exists. |
| Telegram/WhatsApp as product domains | Would duplicate Communications lifecycle and engine integration. | Keep channel specs under Communications architecture. |
| Observations as a god table | Would blur engine/domain responsibility. | Define concrete observation ownership first. |
| Attachments split across channels | Could duplicate bytes or lose scanner state. | Use shared Communications attachment boundary and explicit Document promotion. |
| Frontend `domains/` naming | UI module names may be mistaken for backend bounded contexts. | Document UI modules as surfaces, not truth owners. |
