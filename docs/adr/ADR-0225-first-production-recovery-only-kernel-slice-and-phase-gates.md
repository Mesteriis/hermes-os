# ADR-0225: Первый production slice — recovery-only Kernel и фазовые ворота

Статус: Принято
Дата: 2026-07-16
Состояние реализации: exact six-package inventory, recovery-only runtime,
private IPC, Control Store anchor/export/restore/reset fencing и architecture
self-tests реализованы; device enrollment и все post-recovery gates закрыты.

Уточняет:

- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0202: PostgreSQL ownership, PgBouncer и extensions](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0209: Kernel Event Hub и subscription control plane](ADR-0209-kernel-event-hub-and-subscription-control-plane.md);
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md);
- [ADR-0211: Backend workspace и физическая структура исходного кода](ADR-0211-backend-workspace-and-source-layout.md);
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213: Конституция кода, ownership и автономность модулей](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0214: Durable Job Platform и Scheduler](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0217: Нулевой внешний bootstrap Kernel](ADR-0217-zero-external-dependency-kernel-bootstrap.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0219: Целостность managed modules и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0224: Storage Control Plane и owner-scoped PostgreSQL](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md).
- [ADR-0227: Deployment profiles и server bootstrap pairing](ADR-0227-deployment-profiles-and-server-bootstrap-pairing.md).

## Контекст

Clean-room ADR уже определили финальную конституцию Kernel, но до этого решения
первый production package оставался заблокирован неопределёнными capability и
domain ownership inventory. Если сразу реализовать весь финальный graph, первый
срез одновременно потребует PostgreSQL, PgBouncer, Storage Control, NATS,
Vault, Telemetry Collector, modules, settings и client API. Такой срез нельзя
проверить изолированно и невозможно безопасно откатить по ответственности.

Обратная крайность — создать пустые crates, fake handlers или состояние
`ready` без capabilities — также запрещена. Первый срез обязан доказывать
завершённое полезное поведение: Kernel запускается без внешних сервисов,
владеет private Control Store, удерживает single-instance boundary и всегда
оставляет безопасный локальный recovery surface.

Это решение закрывает inventory только для первого среза. Оно не объявляет
готовыми business domains, managed infrastructure или data plane.

## Решение

### Текущий implementation slice

Первый разрешённый production slice имеет stable ID:

```text
kernel_recovery_only_v1
```

Его максимально достижимое состояние — `recovery_only`. Пустой business или
required-capability inventory не превращает Kernel в `ready`. Если позже
появится HTTP readiness endpoint, он возвращает not-ready с sanitized reason
`production_phase_recovery_only`.

Переход к следующему slice выполняется только явным изменением ADR, executable
policy и tests. Feature flag, settings value, environment variable, manifest
entry или обнаруженный executable не открывает фазовый gate.

### Точный production package inventory

В `kernel_recovery_only_v1` разрешены ровно шесть production packages:

| Package | Hermes metadata | Ответственность |
|---|---|---|
| `hermes-events-protocol` | `platform:events:contract` | Реальный binary `DurableEnvelopeV1` contract ADR-0220 без NATS/outbox runtime |
| `hermes-runtime-protocol` | `platform:runtime_protocol:contract` | Реальные lifecycle, health, descriptor и settings wire types ADR-0221 без module activation |
| `hermes-gateway-protocol` | `api:gateway:contract` | Typed local recovery requests, results и safe errors |
| `hermes-kernel-control-store` | `core:kernel:contract` | Узкий Control Store port, models и typed errors без `rusqlite` |
| `hermes-kernel-control-store-sqlite` | `core:kernel:persistence` | Private SQLite schema, migrations, integrity validation и Control Store export/restore adapter |
| `hermes-kernel` | `core:kernel:runtime` | Один process/binary: bootstrap, lock, state machine, recovery routing и bounded shutdown |

Пустой scaffold не считается package. Protocol packages содержат принятые
Protobuf V1 messages и conformance tests, даже если соответствующий data plane
ещё выключен.

Допустимый production dependency graph:

```text
hermes-kernel-control-store-sqlite
  -> hermes-kernel-control-store

hermes-kernel
  -> hermes-kernel-control-store
  -> hermes-kernel-control-store-sqlite
  -> hermes-gateway-protocol
  -> hermes-runtime-protocol

hermes-events-protocol
  -> no production implementation
```

`hermes-gateway-protocol` может зависеть только от необходимых platform
contracts. Он не переиспользует internal durable envelope как client frame.
Protocol packages не зависят от Kernel, SQLite, SQLx, NATS, HTTP clients,
filesystem или owner packages.

Test-only package не входит в production inventory. Если нужен shared process
harness, он живёт только в `backend/tests/support/kernel-recovery/` как
`hermes-kernel-recovery-testkit` с metadata `test:test:test_support`, запускает
Kernel как внешний child и не импортирует production composition root.

### Пустой business ownership inventory

Для текущего slice inventory фиксируется явно:

```text
domains      = []
integrations = []
workflows    = []
engines      = []
business capabilities = []
```

`domains.registered` и `domains.developmentAllowlist` остаются product
governance ADR-0207/0208. Allowlist означает «разрешено проектировать и
реализовывать после открытия owner gate», а не «уже входит в текущую
distribution». До `first_owner_v1` никакой domain, integration, workflow,
engine или AI production package не допускается.

### Активные Kernel components

ADR-0206 сохраняет полный закрытый перечень обязанностей Kernel. В текущем
slice реально активны только:

- `supervisor` — process state machine, data-directory lock, bounded shutdown
  и recovery lifecycle без managed children; SIGTERM/SIGINT опрашиваются с
  bounded local interval и удаляют private recovery socket перед exit;
- `core_gateway` — owner-private local IPC recovery surface.

Остальные Kernel-owned components зарезервированы конституцией, но не
объявляются реализованными и не получают fake handlers:

- Module Registry;
- capability router;
- Event Hub;
- Telemetry control;
- Settings Registry.

Emergency bootstrap/crash log является bounded внутренней boot
ответственностью и не выдаётся за Telemetry Collector.

### Runtime states и операции

В текущем slice разрешены состояния:

```text
cold_start
bootstrap
recovery_only
quiescing
draining
stopped
fatal
```

Запрещены `infrastructure_starting`, `modules_starting`, `ready` и `degraded`:
они требуют capabilities, которых в inventory нет.

Online local IPC предоставляет ровно:

- `GetRecoveryStatusV1`;
- `ValidateControlStoreV1`;
- `ExportControlStoreV1`;
- `ShutdownKernelV1`.

`shutdown` является lifecycle operation и не превращает unavailable Control
Store в writable recovery surface. Online restore/reset, settings mutation,
module approval, infrastructure start/stop и business request запрещены.

На pristine instance разрешён один bootstrap mutation
`InitialOwnerEnrollmentV1` только через inherited private FD и platform signer
по ADR-0218. Он не доступен через registration listener, HTTP, argv,
environment или обычный local IPC. Повторная enrollment без отдельной
owner-authorized recovery operation отклоняется.

Offline packaged CLI сохраняет только уже принятые Control Store operations:

```text
control-store restore --data-dir <explicit-path>
control-store reset --data-dir <explicit-path>
```

Они требуют остановленный Kernel, explicit data directory, exclusive lock,
interactive local confirmation и generation/epoch fencing. Это не
whole-instance backup/restore и не затрагивает PostgreSQL, Vault, blobs,
provider sessions или business data.

### Нулевые внешние сервисы и exact crate profile

До следующей фазы Kernel не запускает и не подключает:

- PostgreSQL, PgBouncer или Storage Control;
- NATS/JetStream;
- Vault;
- Blob service;
- Telemetry Collector;
- Scheduler;
- module runtimes;
- provider integrations.

В production graph текущего slice запрещены NATS clients, PostgreSQL clients,
Vault implementation, provider SDK и owner packages. При этом recovery
поведение не реализуется самописной криптографией, CLI parser или SQLite file
copy. На 2026-07-16 разрешён только следующий direct crates.io profile:

| Package | Exact direct dependency profile |
|---|---|
| три `hermes-*-protocol` | `prost =0.14.4`, `prost-types =0.14.4`, build-only `prost-build =0.14.4`, `protoc-bin-vendored =3.2.0`; default features |
| `hermes-kernel-control-store-sqlite` | `rusqlite =0.40.1`, defaults off, `backup,bundled`; `sha2 =0.11.0`, defaults off |
| `hermes-kernel` | `clap =4.6.2`, defaults off, `derive,error-context,help,std,usage`; `directories =6.0.0`; `p256 =0.14.0`, defaults off, `ecdsa`; `getrandom =0.4.3`, defaults off; `sha2 =0.11.0`, defaults off; `signal-hook =0.3.18` |

Version requirement, dependency kind, crates.io source, default-feature mode и
feature set проверяются executable policy. Rename, optional edge, git/path
substitution и неразрешённый direct dependency fail closed. Internal edges
проверяются по resolved Cargo package ID, а не только по совпавшему имени.

Разделение ответственности:

- `clap` владеет typed offline/recovery commands, feature `env` не включён;
- `directories::ProjectDirs::data_local_dir()` выбирает OS-standard local
  data path;
- `getrandom` выдаёт challenge bytes из OS CSPRNG, ошибка блокирует operation;
- `p256` только проверяет ES256 proof: private device key остаётся platform
  signer;
- `sha2` в Kernel считает operation/anchor digests, а в SQLite adapter —
  checksum export artifact;
- `signal-hook` устанавливает только SIGTERM/SIGINT flags для bounded polling
  private recovery socket; handler не выполняет I/O или state mutation;
- `rusqlite::backup` создаёт consistent export; открытый database file не
  копируется напрямую.

`p256 0.14.0` прямо предупреждает, что его curve arithmetic не проходила
независимый полный аудит. Это не скрывается и crate не объявляется
сертифицированным. Для текущего verifier-only boundary решение принимается с
обязательными cross-platform vectors, malformed/non-canonical/length checks,
low-S enforcement и dependency/advisory review до merge первого manifest.
Замена crypto backend меняет policy и security evidence отдельным решением.
Самописная криптография запрещена.

Дополнительный lock/IPC crate в текущем Unix/macOS slice не нужен:
`std::fs::File::try_lock` удерживает advisory single-instance lock, а private
Unix socket использует `std::os::unix::net`. Поддержка Windows local IPC не
объявляется этим slice и требует отдельного platform profile.

### Минимальная модель времени

Полностью исключить clock нельзя: bootstrap challenge, session deadline,
expiry и bounded shutdown уже требуют корректного времени. Поэтому
`hermes-kernel` использует внутренний внедряемый `KernelClock` port:

- wall clock используется только для UTC timestamps и persisted absolute
  instants;
- monotonic clock используется для elapsed duration, timeout и deadline внутри
  process lifetime;
- тесты используют deterministic fake clock;
- wall-clock jump не продлевает monotonic in-process deadline;
- Kernel не публикует module-facing Clock capability в текущем slice.

Отдельный `clock_v1` gate по-прежнему обязателен до Scheduler, periodic jobs и
module timer requests. Rust `SystemTime` и `Instant` имеют разные semantics и
не подменяют друг друга.

## Фазовые ворота

`not_authorized` означает запрет package, process, route, dependency, feature
flag и runtime activation. Gate открывается только одним согласованным
изменением ADR + policy + executable evidence.

### `module_control_plane_v1`

Открывает Module Registry, external `pending` registration, owner approval,
GrantSet, capability router и Settings Registry.

До открытия нужны:

- owner/device session conformance ADR-0218;
- exact ModuleDescriptorV1 parser/limits;
- grant/revoke epoch persistence;
- local IPC replay/abuse tests;
- owner-authorized mutation surface.

### `managed_launch_trust_v1`

Открывает любой managed child launch. До gate должны быть зафиксированы и
проверены:

- exact binary encoding и schema digest `DistributionManifestV1`;
- detached signature suite;
- источник pinned verification keys и rotation policy;
- release-signing evidence;
- path/symlink/partial-install validation;
- TOCTOU-safe platform spawn adapter;
- exact-byte executable/descriptor/settings verification перед каждым launch.

Tauri updater/platform signature не заменяет внутренний managed-launch
verification. Kernel download/install, automatic rollback и silent fallback
остаются запрещены.

### `vault_v1`

Открывает Vault packages/process только после `managed_launch_trust_v1` и
требует exact package inventory, HPKE session conformance, SQLCipher/platform
key adapter, lease expiry/revoke/epoch fencing, secret non-disclosure tests и
backup/restore classification. Verified launch сам по себе не доказывает Vault
semantics.

### `telemetry_v1`

Открывает Telemetry Collector только после `managed_launch_trust_v1` и требует
exact package inventory, private local transport, schema/redaction/quotas,
bounded retention, collector failure isolation и negative tests на secrets и
private content.

### `storage_control_v1`

Открывает PostgreSQL, PgBouncer и Storage Control после
`managed_launch_trust_v1` и `vault_v1`. До gate нужны exact packages/artifacts,
role/grant/pool/budget conformance, AST migration admission, Vault credential
fencing, evidence ограничения PgBouncer bypass и readiness/recovery tests.

### `nats_data_plane_v1`

Открывает managed NATS, Event Hub reconciliation и durable delivery. Gate
зависит от `managed_launch_trust_v1`, `vault_v1` и `storage_control_v1`:
Vault выдаёт per-runtime credentials, а PostgreSQL outbox/inbox остаётся
durable truth. До gate требуются:

- exact NATS artifact/version/listener profile;
- Event Hub/NATS adapter package inventory;
- subject catalog version;
- concrete stream retention/bytes/storage/replica budgets;
- concrete consumer ack/delivery/backoff/deadline/DLQ budgets;
- per-runtime NATS identity;
- credential authority, protected delivery, expiry, rotation, revoke, forced
  disconnect и runtime/grant-generation fencing;
- outage, replay, duplicate и stale-generation tests.

Shared broker token и временный wildcard `hermes.>` запрещены.

### `whole_instance_backup_v1`

Зависит от `vault_v1`, `telemetry_v1`, `storage_control_v1` и
`nats_data_plane_v1` и обязателен до production rollout первого durable
owner/user data. Gate требует:

- component inclusion matrix;
- quiesce/consistency order;
- retention и encryption/key authority;
- signed media/manifest format;
- PostgreSQL roles/grants/storage-ledger handling;
- Control Store, Vault, provider session, Blob и JetStream inclusion policy;
- restore authorization, order и generation/epoch fencing;
- полный restore test в disposable environment.

Control Store export или component-local SQLite/PostgreSQL backup не закрывает
этот gate.

### `blob_v1`

Зависит от `managed_launch_trust_v1` и `vault_v1`. Обязателен до первой выдачи
`BlobRef` и до attachments, documents или media. Gate требует exact
protocol/package topology, opaque ref bindings, encryption,
owner/account/runtime/grant/expiry scopes, quotas, retention/GC, range policy,
path-traversal defense, revoke semantics, backup classification и replay tests.
Если Blob уже включён, `whole_instance_backup_v1` дополнительно зависит от
`blob_v1`.

Если Scheduler уже включён, `whole_instance_backup_v1` также условно зависит
от `scheduler_v1` и включает его schedules, runs, leases и fencing state.

### `clock_v1`

Зависит от `module_control_plane_v1` и `managed_launch_trust_v1`. Обязателен до
Scheduler, periodic jobs и module timer capability. Gate требует public
protocol/package topology, wall/monotonic semantics, UTC/timezone/DST, clock
jump/drift/suspend behavior, deadline contract и deterministic fake-clock
suite. Внутренний `KernelClock` текущего slice не открывает этот gate.

### `scheduler_v1`

Открывает Scheduler и Job Plane только после `module_control_plane_v1`,
`managed_launch_trust_v1`, `vault_v1`, `telemetry_v1`,
`storage_control_v1`, `nats_data_plane_v1` и `clock_v1`. Gate требует:

- exact Scheduler protocol/runtime/package inventory;
- exact `JobSpec`/`JobKind`/owner contract binding;
- PostgreSQL schedule/run/lease/fencing storage;
- NATS acceptance/result/ack contract;
- hot reconciliation без dynamic Rust loading;
- retry/idempotency/misfire/recovery semantics;
- deterministic clock, lease-race и crash tests.

Код job остаётся в owner module; Scheduler владеет временем, scheduling state,
run identity и fencing, но не business execution.

### `client_gateway_v1`

Открывает public client transport только после `module_control_plane_v1`,
`telemetry_v1` и `nats_data_plane_v1`. Gate требует:

- exact gateway package и listener inventory;
- owner-device session/authentication/authorization conformance;
- ConnectRPC deadlines, typed errors и durable command receipt mapping;
- один SSE stream с replay, gap/reset и explicit disconnect semantics;
- отделение ConnectRPC от health/OAuth/blob/SSE HTTP surface;
- remote HTTP/2 + TLS baseline;
- HTTP/3 fallback, raw-QUIC prohibition и no-0-RTT conformance;
- abuse, replay, privacy, redaction и Android suspension/reconnect tests.

Текущий local recovery `core_gateway` не открывает public listener и не
считается реализацией этого gate.

### `first_owner_v1`

Открывает первый owner module любого типа: domain, integration, workflow или
engine — только после отдельного owner ADR с exact packages, public contracts,
StorageBundle, capabilities, dependencies и tests. Gate требует
`module_control_plane_v1`, `managed_launch_trust_v1`, `vault_v1`,
`telemetry_v1`, `storage_control_v1`, `nats_data_plane_v1` и
`client_gateway_v1` и `whole_instance_backup_v1`. Для owner с blobs
дополнительно требуется `blob_v1`; для owner с jobs, schedules или timers —
`scheduler_v1` (который уже требует `clock_v1`).

Если Scheduler state уже включён к моменту whole-instance backup,
`whole_instance_backup_v1` условно зависит от `scheduler_v1` и обязан включить
его PostgreSQL state, leases/fencing и restore order. Наличие schedules у
первого owner никогда не открывает Scheduler автоматически.

Таким образом, flat toggle открыть позднюю фазу не может. `phaseGates.requires`
и `conditionalRequires` executable policy фиксируют тот же порядок и меняются
атомарно с ADR и evidence.

## Запрещено в текущем slice

- domain/integration/workflow/engine/AI production packages;
- business API, SSE, public HTTP/TCP listener или Android connection;
- module registration, grants и settings mutations;
- managed children и executable update/rollback;
- NATS/Event Hub data plane;
- PostgreSQL/PgBouncer/Storage Control, Vault, Blob, Scheduler или public
  client gateway;
- provider-account/agent identity;
- whole-instance backup claim;
- legacy import, migration или compatibility facade;
- автоматический reset, fallback или переход в `ready`.

## Проверка решения

До изменения `Состояние реализации` обязательны:

- policy принимает только exact six-package production inventory;
- empty и test-only workspace остаются допустимыми;
- лишний domain/integration/platform implementation package ломает guard;
- любой file под `backend/src`, не принадлежащий registered root одного из
  exact six packages, ломает guard, включая hidden `.rs`, `.proto`, `.sql` и
  `build.rs`;
- Kernel metadata объявляет только реально active components текущего slice;
- package dependency graph не содержит NATS/PostgreSQL/HTTP clients,
  Vault/Storage/provider packages или undeclared internal edges; internal edge
  разрешается только к exact Cargo package ID, а не registry namesake;
- direct third-party dependencies совпадают по crate, exact version, kind,
  crates.io source, default-feature mode и feature set;
- Cargo features не могут скрыто открывать phase, а `hermes-kernel` имеет один
  binary target без собственного build script;
- runtime state set не содержит `ready`, `degraded` или startup states будущих
  managed capabilities;
- boot без PostgreSQL/NATS/Vault/Blob достигает `recovery_only`;
- пустой capability inventory не достигает `ready`;
- invalid/missing Control Store оставляет только status/validate/export/shutdown;
- online mutation, network listener и managed spawn отсутствуют;
- pristine enrollment принимает только inherited-FD proof;
- second enrollment, replay и OS-identity-only authorization отклоняются;
- single-instance lock и bounded shutdown доказаны process tests;
- wall/monotonic/fake-clock semantics доказаны deterministic tests;
- diagnostics не содержат secrets или private content;
- `make -C backend validate` проходит.

## Последствия

Положительные:

- запрет первого production package заменён точным проверяемым scope;
- Kernel можно реализовать без одновременного запуска всей инфраструктуры;
- будущие возможности не маскируются fake components;
- опасные managed/NATS/backup/blob paths fail closed до своих решений;
- product allowlist отделён от фактически реализованного inventory.

Отрицательные:

- первый slice намеренно не является usable product backend;
- два protocol packages реализуются раньше их transport adapters;
- переход к каждому следующему gate требует отдельного ADR/policy change;
- whole-instance backup становится обязательной ранней работой до user data.

## Ссылки

- [prost 0.14.4](https://docs.rs/prost/0.14.4/prost/)
- [clap 4.6.2](https://docs.rs/clap/4.6.2/clap/)
- [directories 6.0.0 `ProjectDirs`](https://docs.rs/directories/6.0.0/directories/struct.ProjectDirs.html)
- [p256 0.14.0 и security warning](https://docs.rs/p256/0.14.0/p256/)
- [getrandom 0.4.3](https://docs.rs/getrandom/0.4.3/getrandom/)
- [sha2 0.11.0](https://docs.rs/sha2/0.11.0/sha2/)
- [rusqlite 0.40.1](https://docs.rs/rusqlite/0.40.1/rusqlite/)
- [Rust `File::try_lock`](https://doc.rust-lang.org/stable/std/fs/struct.File.html#method.try_lock)
- [Rust `UnixListener`](https://doc.rust-lang.org/stable/std/os/unix/net/struct.UnixListener.html)
- [Rust `SystemTime`](https://doc.rust-lang.org/std/time/struct.SystemTime.html)
- [Rust `Instant`](https://doc.rust-lang.org/std/time/struct.Instant.html)
- [NATS authentication](https://docs.nats.io/running-a-nats-service/configuration/securing_nats/auth_intro)
- [NATS authorization](https://docs.nats.io/running-a-nats-service/configuration/securing_nats/authorization)
- [NATS credential revocation](https://docs.nats.io/using-nats/nats-tools/nsc/revocation)
- [Tauri updater security model](https://v2.tauri.app/plugin/updater/)
- [SQLite Online Backup API](https://www.sqlite.org/backup.html)
- [PostgreSQL backup approaches](https://www.postgresql.org/docs/current/backup.html)
