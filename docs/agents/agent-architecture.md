# Agent Architecture

## Agent System Goal

Agents are specialized application actors that help classify, connect, summarize, search and act on the user's memory. They are not the source of truth. They operate through typed tools, explicit permissions and source-backed context.

## Initial Agents

| Agent | Role |
| --- | --- |
| HESTIA | coordinator, intent routing, policy mediation |
| HERMES | communications triage, threads, drafting, channel context |
| MNEMOSYNE | memory, graph linking, recall, provenance |
| ATHENA | analytics, trend detection, decision support |
| HEPHAESTUS | development, maintenance and tool automation |

## Agent Runtime

```mermaid
flowchart LR
    UserIntent["User intent or system event"] --> Hestia["HESTIA"]
    Hestia --> Planner["Plan"]
    Planner --> Policy["Policy and capability check"]
    Policy --> Tools["Typed tools"]
    Tools --> Memory["Search, graph, events, documents"]
    Tools --> External["External providers if permitted"]
    Tools --> Audit["Agent audit events"]
    Memory --> Response["Source-backed response"]
    Audit --> Response
```

## Tool Contract

Agent tools must define:

- input schema
- output schema
- permissions
- data access scope
- side-effect class
- timeout
- audit behavior
- error model

## Memory Use

Agents retrieve context from:

- graph queries
- full text search
- vector search
- event timeline
- document extracts
- task/project projections

Agents must distinguish source facts, inferred links and generated summaries.

## Side Effects

Safe read-only actions may execute automatically inside an approved workflow. External writes, message sending, deletion, provider changes and sensitive exports require explicit confirmation and audit events.
