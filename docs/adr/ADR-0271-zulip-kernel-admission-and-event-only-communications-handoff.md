# ADR-0271: Zulip Kernel admission and event-only Communications handoff

Статус: Принято
Дата: 2026-07-24
Состояние реализации: Phase gate; `zulip_integration_v1` не открыт. Пять
Zulip-owned packages, provider anti-corruption mapper, HTTPS adapter,
owner-local PostgreSQL state и exact-byte Communications outbox уже существуют.
Runtime использует один `ManagedControlChannelV2<UnixStream>` для descriptor,
Storage/Vault, Blob, Event Hub и client delivery; cloned readers, V1 platform
helpers и `MSG_PEEK` удалены, а вложенная client delivery получает
correlation-bound `RUNTIME_BUSY`. Generated command/query contracts, exact
descriptor-set binding, canonical settings schema и `ModuleDescriptorV1` с
раздельными client/platform capabilities реализованы. Immutable owner-local
Storage bundle, отдельная unsigned release assembly unit и generic signed
distribution binding реализованы. Live managed admission ещё не реализован.

Уточняет:

- [ADR-0201: Core module communication and NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204: integration plugins](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205: Core Gateway](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0215: module admission](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219: managed distribution integrity](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0221: module descriptor and capabilities](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0248: Zulip clean-room provider contract](ADR-0248-zulip-clean-room-provider-contract.md);
- [ADR-0256: owner-declared ClientRpc routing](ADR-0256-owner-declared-client-rpc-route-admission.md);
- [ADR-0258: correlated duplex managed control](ADR-0258-correlated-duplex-managed-control-transport.md).

## Контекст

Zulip является integration owner. Его обращение к Kernel/Core означает только
platform control plane: registration, managed launch, settings, Storage, Vault,
Blob, Event Hub и opaque client routing. Это не business-вызов Communications.

Provider observation пересекает owner boundary только как typed durable
observation:

```text
External Zulip
  -> Zulip HTTPS adapter
  -> Zulip-owned PostgreSQL transaction/outbox
  -> exact DurableEnvelopeV1 bytes
  -> NATS JetStream
  -> Communications inbox/domain/outbox
```

Kernel выдаёт и отзывает права, но не декодирует Zulip или Communications
payload. Communications не вызывает Zulip runtime, не читает Zulip queue
cursor и не получает API key, realm-private locator или provider content.

Текущий `ZulipOperationalService/Execute` объединяет provider command и
operation-status query в одном umbrella request. Эти операции имеют разные
authority, failure modes и причины изменения. Runtime control transport также
не удовлетворяет protocol-major `2`: несколько cloned readers одного Unix FD
могут потребить чужой correlated response.

## Решение

### Owner и единицы сборки

Production runtime identity:

```text
owner_id  = zulip
module_id = hermes-zulip-runtime
```

Zulip source units:

```text
hermes-zulip-api          generated provider operational contracts
hermes-zulip-core         provider anti-corruption and neutral evidence mapper
hermes-zulip-http         HTTPS protocol adapter
hermes-zulip-persistence  owner-local PostgreSQL state and outbox
hermes-zulip-runtime      managed application/runtime composition
```

Release composition is a separate integration-owned assembly unit from
ADR-0272. It is not a runtime, domain, platform component or signing authority.

`hermes-communications-ingress` remains the only Zulip dependency owned by
Communications and exposes typed provider-neutral observation construction,
not domain implementation. Zulip must not import Communications API, domain,
persistence or runtime. Communications, Kernel and Gateway must not import a
Zulip implementation package.

### Kernel control plane

Kernel:

- registers exact descriptor bytes as `pending`;
- intersects requested capabilities with owner-approved grants and hard policy;
- verifies signed executable, descriptor, settings schema and Storage bundle
  bindings before managed launch;
- issues runtime generation/grant epoch-bound Storage, Vault, Blob and Event
  credentials;
- routes opaque client payload only through exact approved descriptor routes;
- fences every route and lease on revoke, suspend, generation or binding
  replacement.

Kernel does not choose Zulip account, command, recipient, stream/topic, queue
cursor or Communications mutation. It never stores API key or provider
content.

Zulip runtime uses exactly one `ManagedControlChannelV2<UnixStream>` and one
correlated frame pump for descriptor/ready, Storage, Vault/provider credential,
Blob, Event Hub and client delivery. `UnixStream::try_clone`, independent V1
request readers and `MSG_PEEK` dispatch are forbidden.

### Route-specific operational contracts

Generated Protobuf exposes two independent public client contracts:

| Capability | Contract | Connect path |
|---|---|---|
| `zulip.command.v1` | `zulip.command.v1` | `/hermes.zulip.v1.ZulipCommandService/ExecuteCommand` |
| `zulip.query.v1` | `zulip.query.v1` | `/hermes.zulip.v1.ZulipQueryService/GetOperationStatus` |

Both use exact descriptor-set SHA-256, `major = 1`, `revision = 1`. A command
grant never authorizes query and a query grant never authorizes provider
mutation. Runtime dispatches from Kernel-supplied exact contract reference and
decodes only the corresponding generated request. Umbrella decode probing and
fallback to `ZulipOperationalService/Execute` are forbidden after cutover.

Platform capabilities are separate units:

```text
zulip.blob.v1
zulip.credentials.v1
zulip.events.v1
zulip.storage.v1
```

`zulip.credentials.v1` requests only exact API-key purpose for the current
configuration instance. Blob grants remain operation/size/reference bound;
they do not expose generic Blob access.

Provider command returns an accepted receipt. `accepted` is not terminal
provider success. Terminal result is persisted in Zulip storage and read only
through `zulip.query.v1` or a separately admitted realtime contract.

### Event-only Communications handoff

Zulip maps provider frames into typed provider-neutral observations through
`hermes-communications-ingress`, persists exact bytes in its own outbox and
relays them without re-encoding. Communications ACKs only after inbox
deduplication and owner-local mutation/outbox commit.

Forbidden:

- direct Zulip → Communications RPC/socket/handler;
- cross-owner SQL, shared table, database role or transaction;
- Communications import of Zulip DTO/HTTP/runtime/persistence;
- Zulip import of Communications domain/runtime/persistence;
- Kernel/Gateway payload conversion;
- REST proxy, compatibility facade, dual-write or fallback;
- provider bodies, API keys, queue IDs or realm-private URLs in subjects,
  route metadata, logs, errors or health.

### Phase gate `zulip_integration_v1`

Gate opens atomically only with:

1. exact five-package source inventory and compile-isolation guards;
2. route-specific generated command/query contracts and exact descriptor;
3. canonical settings schema and immutable owner-local Storage bundle;
4. separate Zulip release assembly and signed distribution binding;
5. pending registration plus explicit owner-approved capability subset;
6. managed launch on one correlated V2 control channel;
7. exact Storage/Vault/Blob/Event grants with stale/revoke/replacement fences;
8. live HTTPS provider command with accepted and terminal query evidence;
9. live provider event → Zulip outbox → NATS → Communications inbox flow;
10. duplicate delivery and NATS outage replay without runtime failure;
11. privacy evidence for subjects, routes, errors, logs and health.

Frontend cutover is secondary and cannot prove backend admission. Removing the
legacy frontend REST path is a separate final client slice; it does not permit
a backend facade or fallback during this gate.

Opening the gate does not expand `first_owner_v1`, add Zulip to Communications
inventory, make integration a domain, or authorize WhatsApp/another provider.

## Порядок реализации

1. ~~Replace the shared control FD readers with one correlated V2 frame pump.~~
   Реализовано в `8bc3acc73`.
2. ~~Split generated command/query routes and build exact descriptor/settings.~~
   Реализовано в `6f9229ce3`.
3. ~~Add immutable Storage bundle and separate release assembly unit.~~
   Реализовано в `81449906e`; signed distribution binding доказан в
   `ff2c53983`.
4. Add signed managed launch, grant and revoke/generation conformance.
5. Prove live provider command and event-only Communications delivery.
6. Remove the frontend legacy REST surface in its own client slice.

Каждый крупный slice является отдельным commit и проходит focused owner tests,
Clippy, architecture/SRP/Cargo boundary gates и relevant live conformance.

## Отклонённые варианты

### Zulip вызывает Communications API

Отклонено: это direct cross-owner business dependency, обходящая outbox/inbox,
deduplication и failure isolation.

### Kernel вызывает Communications от имени Zulip

Отклонено: Kernel стал бы owner-specific facade и интерпретировал business
payload.

### Один `zulip.client`

Отклонено: command и query требуют независимых grants. Общий route выдаёт
лишние права и возвращает decode probing.

### Сборка внутри runtime или Communications

Отклонено: release composition, provider runtime и domain имеют разные причины
изменения и authority.

## Последствия

Zulip остаётся отдельной integration unit. Kernel/Core является только
platform admission/routing boundary, Communications остаётся provider-neutral
domain consumer, а связь между ними существует только через typed durable
events.
