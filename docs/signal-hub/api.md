# Signal Hub API

Status: target ConnectRPC API.

Signal Hub APIs are command/query APIs for the local owner and UI. They do not
expose raw provider protocols. They must be implemented as contract-first
Protobuf + ConnectRPC services, with Axum hosting the HTTP transport.

REST endpoints may exist only as temporary compatibility shims during migration.
They must not become the canonical API contract.

## Service

```proto
service SignalHubService {
  rpc ListSources(ListSourcesRequest) returns (ListSourcesResponse);
  rpc GetSource(GetSourceRequest) returns (GetSourceResponse);
  rpc ListConnections(ListConnectionsRequest) returns (ListConnectionsResponse);
  rpc CreateConnection(CreateConnectionRequest) returns (CreateConnectionResponse);
  rpc UpdateConnection(UpdateConnectionRequest) returns (UpdateConnectionResponse);
  rpc RemoveConnection(RemoveConnectionRequest) returns (RemoveConnectionResponse);

  rpc EnableSource(EnableSourceRequest) returns (EnableSourceResponse);
  rpc DisableSource(DisableSourceRequest) returns (DisableSourceResponse);
  rpc MuteSignals(MuteSignalsRequest) returns (MuteSignalsResponse);
  rpc UnmuteSignals(UnmuteSignalsRequest) returns (UnmuteSignalsResponse);
  rpc PauseSignals(PauseSignalsRequest) returns (PauseSignalsResponse);
  rpc ResumeSignals(ResumeSignalsRequest) returns (ResumeSignalsResponse);

  rpc GetHealth(GetHealthRequest) returns (GetHealthResponse);
  rpc RunHealthCheck(RunHealthCheckRequest) returns (RunHealthCheckResponse);

  rpc ApplyProfile(ApplyProfileRequest) returns (ApplyProfileResponse);
  rpc ListProfiles(ListProfilesRequest) returns (ListProfilesResponse);

  rpc RequestReplay(RequestReplayRequest) returns (RequestReplayResponse);
  rpc ListReplayRequests(ListReplayRequestsRequest) returns (ListReplayRequestsResponse);

  rpc ListFixtureSources(ListFixtureSourcesRequest) returns (ListFixtureSourcesResponse);
  rpc EmitFixtureSignal(EmitFixtureSignalRequest) returns (EmitFixtureSignalResponse);
  rpc RestoreSystemFixture(RestoreSystemFixtureRequest) returns (RestoreSystemFixtureResponse);
}
```

## SSE Realtime

Realtime is not ConnectRPC streaming in the first browser surface. Browser
updates use Axum SSE:

```text
GET /api/v1/events/stream
```

SSE event families:

```text
signal.source.updated
signal.connection.updated
signal.health.updated
signal.policy.updated
signal.replay.updated
projection.signal_hub.updated
```

The SSE stream patches generated-client query caches on the frontend. It does
not replace durable event processing.

## Command Semantics

### EnableSource

Enables source runtime and publication policy.

Must emit:

```text
signal.source.enabled
```

### DisableSource

Stops source runtime and prevents capture/publication.

Must emit:

```text
signal.source.disabled
```

### MuteSignals

Suppresses publication according to scope and pattern. Runtime may stay active.

Scopes:

```text
global
source
connection
event_pattern
profile
```

Must emit:

```text
signal.source.muted
signal.policy.changed
```

### PauseSignals

Captures/buffers eligible signals but does not publish them to downstream
consumers until resumed.

Must emit:

```text
signal.source.paused
signal.policy.changed
```

### RequestReplay

Creates a replay request. Replay consumes from event store / NATS JetStream / fixture
catalog depending on request scope.

Must emit:

```text
signal.replay.requested
signal.replay.completed
signal.replay.failed
```

### RestoreSystemFixture

Restores missing system Signal Hub source definitions from the schema-agnostic
fixture. It must never overwrite user-owned connection secrets or provider
runtime sessions.

Must emit:

```text
signal.fixture.restore_requested
signal.fixture.restored
signal.fixture.restore_failed
```

## Query Semantics

Queries read projections where possible:

```text
signal_hub_dashboard_projection
signal_hub_source_projection
signal_hub_health_projection
```

They must not perform expensive cross-domain joins. If a query needs
Communications, Radar or provider runtime details, it should use read-model
composition at app/BFF level, not mutate ownership boundaries.

## Authorization

Initial local API auth remains the repository's local protected API pattern.
Signal Hub commands are owner-local admin commands and should be guarded as
sensitive local operations.

Command classes:

| Command | Action class |
|---|---|
| EnableSource | admin |
| DisableSource | admin |
| MuteSignals | admin |
| PauseSignals | admin |
| ResumeSignals | admin |
| RequestReplay | admin/export-sensitive when payload may be replayed |
| EmitFixtureSignal | test/admin |
| RestoreSystemFixture | recovery/admin |
| CreateConnection | secret-bearing when it starts auth |
| RemoveConnection | destructive |

## Error Model

Errors should be typed and redacted:

```text
SOURCE_NOT_FOUND
CONNECTION_NOT_FOUND
CAPABILITY_UNAVAILABLE
POLICY_CONFLICT
SOURCE_DISABLED
SOURCE_MUTED
SOURCE_PAUSED
REPLAY_RANGE_INVALID
REPLAY_ALREADY_RUNNING
FIXTURE_INVALID
FIXTURE_RESTORE_FAILED
SECRET_REF_MISSING
RUNTIME_UNAVAILABLE
```

Errors must not expose secret values, raw message bodies or provider session
material.
