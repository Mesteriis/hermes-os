# Signal Hub Blockers

Status: active planning blockers.

## Technical Blockers

| Blocker | Required decision |
|---|---|
| NATS dev/test lifecycle | define local/testcontainer NATS JetStream setup for in-process and connector fixtures |
| ConnectRPC Rust wiring | choose exact crate versions and codegen layout for Axum-hosted ConnectRPC |
| Protobuf package layout | decide whether contracts live at repository root or under backend first |
| Signal Hub migration order | create tables before source fixtures; loader must run after schema exists |
| Event transport split | keep PostgreSQL event log as source of truth while NATS handles live delivery |
| Fixture loader timing | decide whether bootstrap runs at backend startup, migration hook or explicit recovery command |
| Projection ownership | decide initial Signal Hub dashboard/read models |

## Architectural Blockers

| Blocker | Risk |
|---|---|
| Provider-specific code still leaking into Communications | Signal Hub controls could inherit provider naming and break neutrality |
| Event family naming | `integration.*` compatibility vs `signal.*` canonical family must be explicit |
| Direct imports | integrations/domains/workflows must not bypass event contracts |
| Connector rollout | a connector requires a crate boundary, durable acknowledgement, lease fencing and fixture evidence |
| Redis temptation | second event substrate would fragment replay/audit semantics |

## Product Blockers

| Blocker | Required UX decision |
|---|---|
| UI naming | final surface label: `Signal Hub`, likely under Settings or Hub workspace |
| Dangerous controls | disable/mute/pause/replay need confirmation and clear visual state |
| Profile switching | profiles must be obvious because testing profile can suppress real signals |
| Fixture mode | UI must show fixture mode loudly to avoid confusing test data with real data |

## Not Blockers

These are intentionally not blockers for the first implementation:

- extracting a provider runtime only after its connector acceptance gates pass;
- Redis;
- Kafka;
- WebSocket hub;
- full WhatsApp implementation;
- multi-user or multi-tenant permission model;
- external SaaS deployment.
