# Signal Hub Operations

Status: target operations contract.

Signal Hub operations are owner-facing controls for the external and synthetic
signal world.

## Control States

| State | Meaning |
|---|---|
| Enabled | source can capture and publish signals |
| Disabled | source should not run or publish |
| Muted | runtime may run, but publication is suppressed by policy |
| Paused | runtime may capture, but publication is buffered/deferred |
| Fixture-only | real source disabled; fixture source enabled |
| Replay-only | no live runtime; replay commands are allowed |

## Global Controls

Global controls affect all sources.

Use cases:

- integration tests;
- deterministic development mode;
- projection rebuild;
- emergency stop when a provider floods the system;
- privacy mode.

Global controls must be explicit and auditable. A global mute without an expiry
should be visually obvious in the UI. Humans will absolutely forget they muted
the universe and then blame the computer. Naturally.

## Selective Mute

Selective mute can target:

```text
source code
connection id
event type pattern
capability family
profile
```

Examples:

```text
mute telegram
mute telegram.message.*
mute mail.attachment.*
mute github.issue.*
mute all during test profile
```

## Pause And Buffer

Pause means Signal Hub can accept/capture events but must not publish to
downstream domain consumers until resumed.

Pause must record:

- policy id;
- source/connection scope;
- started_at;
- reason;
- owner/actor;
- buffered event count;
- maximum buffer policy.

If buffering is not supported for a source, pause degrades to mute and must make
that visible through capability state.

## Replay

Replay can target:

- source;
- connection;
- event type pattern;
- event position range;
- time range;
- target consumer;
- projection rebuild.

Replay must be idempotent and consumer-safe.

Consumers must not assume replay means a new fact. Replay means a previously
stored fact is delivered again for reconstruction, recovery or diagnostics.

## Health Checks

Health checks should produce durable, reviewable state:

```text
signal.source.health_changed
```

Health dimensions:

- runtime process/module state;
- provider authorization state;
- secret reference availability;
- last successful signal;
- last provider failure;
- backoff/retry state;
- capability degradation;
- fixture availability.

## Profiles

Profiles are named policy bundles.

Target system profiles:

| Profile | Intent |
|---|---|
| production | all configured real sources enabled according to owner settings |
| development | selected real sources enabled; noisy providers muted by default |
| testing | real sources muted/disabled; fixture sources enabled |
| maintenance | capture may pause; replay and projections allowed |

Profiles are not security boundaries. They are operational presets.

## UI Requirements

Signal Hub UI should show:

- all source cards;
- status and health;
- active mute/pause/global policies;
- active profile;
- last signal time;
- last error summary;
- replay jobs;
- fixture mode indicators;
- dangerous controls with explicit confirmation.

Recommended workspace:

```text
Settings / Signal Hub
```

or product-level:

```text
Hub / Signals
```

## Auditing

All control operations produce audit records and events:

```text
signal.policy.changed
signal.source.enabled
signal.source.disabled
signal.source.muted
signal.source.paused
signal.replay.requested
signal.fixture.restored
```

Audit payloads must be redacted.
