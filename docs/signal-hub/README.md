# Hermes Signal Hub

Status: `TARGET SPECIFICATION`, 2026-06-22.

Signal Hub is the system control plane for external and synthetic signal
sources in Hermes. It is not a messenger UI, not an email client and not a
provider-specific integration folder. Signal Hub owns the durable registry of
sources, connections, capabilities, runtime state, health, profiles, mute/pause
policies and recovery fixtures.

Hermes receives signals from the world and turns them into Communications,
Radar, Review, domain objects, Memory and Knowledge. Signal Hub controls the
first boundary of that chain.

```text
External World
  -> Signal Hub
  -> Event Backbone
  -> Communications / Calendar / Documents / Tasks / Knowledge
  -> Radar
  -> Review
  -> Domain Objects
  -> Memory / Knowledge / Projections
```

## Position

Signal Hub is a system domain. It manages all signal sources, not only
communication providers.

Examples:

- Email / Mail;
- Telegram;
- WhatsApp;
- GitHub;
- Browser capture;
- RSS;
- Calendar providers;
- Filesystem watchers;
- Home Assistant;
- voice input;
- deterministic fixture sources.

Signal Hub does not own provider protocol code. Provider protocol/runtime code
continues to live under `backend/src/integrations/*`. Signal Hub owns the
source registry and control state used to decide whether a source can publish,
be muted, be paused, replayed, restored or used in tests.

## Core Invariants

- A provider is not a domain.
- A source is not automatically a Communication.
- A signal is evidence from the external or synthetic world.
- Signal Hub controls sources and signal flow.
- Event Backbone transports versioned events.
- Communications owns messages, conversations, participants and attachments.
- Radar owns attention and candidate incubation.
- Review owns promotion decisions.
- Domain objects are created by their owning domains.
- AI can suggest candidates, but cannot become source of truth.

## Current Repository Context

The repository already contains:

- append-only `event_log` migration;
- canonical `EventEnvelope` model;
- an in-process broadcast `EventBus`;
- durable event consumers, retry state and DLQ tables;
- Communications as the provider-neutral message domain;
- Telegram and Mail documentation that states channels are integrations, not
  domains.

Signal Hub documentation defines the next platform-level refactor target: a
source-control domain and event platform that are designed from the start for
NATS JetStream, ConnectRPC, SSE, fixture-first testing and schema-agnostic
recovery fixtures.

## Navigation

- [Architecture](architecture.md)
- [Data Model](data-model.md)
- [Modules](modules.md)
- [API Reference](api.md)
- [Operations](operations.md)
- [Testing](testing.md)
- [Fixtures And Recovery](fixtures.md)
- [Status](status.md)
- [Gap Analysis](gap-analysis.md)
- [Blockers](blockers.md)
