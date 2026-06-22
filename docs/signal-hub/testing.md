# Signal Hub Testing

Status: target testing strategy.

Signal Hub must be designed for tests from the first implementation. Tests
should not require live Telegram, WhatsApp, Mail, GitHub, browser or Home
Assistant providers unless the test explicitly validates a provider adapter.

## Testing Principle

Every real source must have a deterministic replacement.

```text
TelegramSource  -> FixtureTelegramSource
MailSource      -> FixtureMailSource
WhatsAppSource  -> FixtureWhatsAppSource
GitHubSource    -> FixtureGitHubSource
```

Domain tests should operate on signals, not live providers.

## Test Layers

| Layer | Event transport | Database | Purpose |
|---|---|---|---|
| Unit | InMemoryEventBus | none/fake store | policy, validation, fixture parsing |
| Domain integration | InMemoryEventBus or Postgres event store | test PostgreSQL | Signal Hub -> Communications/Radar flows |
| Event integration | NATS JetStream test container + PostgreSQL | test PostgreSQL | publish/consume/replay/DLQ behavior |
| Provider adapter | in-process runtime fixture or mocked provider | test PostgreSQL | protocol-specific parser/runtime boundary |
| E2E local | NATS JetStream + PostgreSQL + SSE | test PostgreSQL | UI-visible source controls and projections |

## Required Test Features

Signal Hub must support:

- global mute;
- selective source mute;
- event-pattern mute;
- pause/resume;
- replay;
- fixture-only profile;
- deterministic fixture emission;
- health check fakes;
- no-provider integration tests.

## FixtureSource Contract

Fixture sources emit normal Signal Hub events. Downstream domains must not know
whether an event came from Telegram, Mail, WhatsApp or a fixture.

```text
FixtureSource
  -> signal.telegram.message.observed
  -> communication.message.recorded
  -> radar.signal.detected
```

## No Live Provider In Core Tests

The following must not be required for core domain tests:

- TDLib login;
- WhatsApp QR login;
- IMAP server credentials;
- GitHub token;
- browser extension runtime;
- Home Assistant instance.

Those belong to provider-specific adapter tests.

## Mute Tests

Test cases:

```text
global mute suppresses all publication
source mute suppresses only selected source
pattern mute suppresses selected event family
muted source still updates health if configured
mute event is audited
mute expiry restores publication
```

## Pause Tests

Test cases:

```text
pause buffers events
resume publishes buffered events in deterministic order
pause degrades to mute when buffering unsupported
pause/resume preserves correlation_id and causation_id
pause does not drop health events unless policy says so
```

## Replay Tests

Test cases:

```text
replay by position range
replay by event type pattern
replay to one consumer
replay projection update flow
replay skips already-processed event when consumer idempotency says so
replay dead-letter after owner review
```

## Snapshot Testing

Use snapshot tests for:

- fixture parsing;
- event envelope shape;
- redacted error payloads;
- capability snapshots;
- health projection rows;
- ConnectRPC DTO examples.

The repository already uses `insta`; Signal Hub should continue using snapshot
assertions for contract drift.

## Mocking

Use mocks for ports, not for internal random functions.

Good:

```text
mock SignalSource
mock EventPublisher
mock SecretResolver
mock Clock
```

Bad:

```text
mock half of SignalHubService internals
```

## Deterministic IDs And Time

Tests should inject:

- clock;
- event id generator;
- provider event id generator for fixture events.

Do not depend on wall-clock time or random IDs in snapshot tests.

## CI Profile

The recommended CI profile:

```text
SignalProfile = testing
Real sources = disabled or muted
Fixture sources = enabled
Event transport = InMemoryEventBus for unit tests
PostgreSQL = testcontainer for integration tests
NATS JetStream = testcontainer for event transport tests
```

If testcontainers are not yet introduced in this repository slice, document the
gap rather than weakening the architecture.
