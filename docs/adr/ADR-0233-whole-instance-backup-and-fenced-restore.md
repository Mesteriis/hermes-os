# ADR-0233: Whole-instance backup и fenced restore

Статус: Принято  
Дата: 2026-07-19  
Состояние реализации: Реализовано на уровне evidence/contour; `whole_instance_backup_v1`
остается fail-closed в `phaseGates.notAuthorized` до снятия gate в следующем
срезе. Kernel уже содержит fail-closed capture/restore coordinators, P-256 signed
media inventory с fd-based no-symlink verification и production process-port, который
вызывает только `StagedNativeArtifact` component executables для Vault, Storage,
Blob и Scheduler. Он также сохраняет и восстанавливает Event Hub topology через
verified Control Store authority. Top-level owner-authorized recovery CLI покрывает как
capture, так и restore, включая empty-target fencing, component-ported восстановление
и offline JetStream topology replay.

Зависит от:

- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0220: Канонический durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

## Контекст

Отдельные export-операции не являются backup экземпляра: между ними могут
измениться Control Store, Vault, PostgreSQL, Blob и JetStream. Называть такой
набор «whole instance» означало бы скрыть потерю causation, duplicate delivery
или старые credential/runtime leases после restore.

Backup должен быть owner-private recovery material, а не Gateway API и не
generic file copier. Он не передаёт private content в NATS, логи, CLI argv или
manifest plaintext. Restore никогда не применяется к существующему target и
не запускается автоматически при boot.

## Решение

### Exact inventory и ownership

Координатор является Kernel-owned offline operation. Он вызывает только
component-owned backup/restore ports; не открывает чужие databases, не читает
owner SQL и не создаёт cross-owner schema.

| Component | Backup unit | Restore authority | Inclusion |
| --- | --- | --- | --- |
| Kernel Control Store | fenced SQLite export | stopped Kernel, exclusive data-dir lock | required |
| Vault | authenticated encrypted snapshot and anchor | recovery-key holder into empty Vault paths | required |
| Storage Control / PostgreSQL | verified custom dump plus schema/migration ledger | isolated empty cluster through Storage Control | required |
| Blob | owner-independent encrypted object inventory and bytes | empty Blob root, verified refs only | required when Blob is admitted |
| JetStream | stream/consumer/config catalog, then replay from durable outbox/inbox | recreated after stores, never arbitrary retained payload copy | required |
| Scheduler | schedules, runs, leases and fences | after PostgreSQL and NATS recovery | only when Scheduler is enabled |
| provider OS profiles, browser credentials, logs, caches, release outputs | none | none | explicitly excluded |

Provider session material belongs to the integration owner and is admitted only
with that owner. The first Mail slice therefore does not silently inherit a
backup promise outside this matrix.

### Quiesce and capture order

The operator requests backup through an owner-authenticated local recovery
surface. Coordinator obtains the exclusive recovery lock, rejects a running
Kernel/managed child, records a monotonic backup generation and captures:

1. Control Store desired topology and generation fence;
2. Vault snapshot while no Vault runtime owns its store;
3. PostgreSQL owner data and migration ledger through Storage Control;
4. required encrypted Blob objects, if enabled;
5. JetStream topology only — canonical messages recover through exact-byte
   outbox/inbox replay, not a second mutable payload authority;
6. Scheduler state only after its conditional gate is admitted.

Any failure removes only the new staging directory; it never overwrites an
existing completed backup. A successful capture fsyncs every regular file and
directory, then publishes exactly one new backup directory atomically.

### Signed media manifest

The published directory contains a versioned binary manifest, component
inventory, byte length and SHA-256 for every file, source commit, Cargo lock,
toolchain and policy digest. The manifest is signed by a P-256 recovery/media
authority distinct from a provider credential or client device key. Its
signature covers canonical order, inclusion classes, backup generation and
all digests. Paths are relative, normalized, non-empty and contain neither
symlinks nor special files. Secrets and message bodies never occur in manifest
fields.

Media retention is an explicit owner policy, not a best-effort cleanup task.
The coordinator only creates private `0700` directories and regular `0600`
files. It uses fd-based no-symlink access for every operator-supplied key,
input and output. Encryption is component-native: Vault/Blob retain their
encryption; the media envelope receives a separate recovery encryption key.

### Restore order and fencing

Restore requires a stopped Kernel, explicit source and **empty** target,
exclusive target lock, local interactive confirmation and P-256 owner-device
proof. The coordinator verifies the complete signed inventory before creating
any target state, including no missing, extra or symlinked files.

It restores in this order:

1. Control Store into a staged empty target and raises the global generation;
2. Vault into empty paths and validates its authenticated snapshot;
3. isolated PostgreSQL and migration ledger through Storage Control;
4. Blob bytes/metadata after their referenced owner storage exists;
5. JetStream catalog and consumers; outbox/inbox replay reconstructs delivery;
6. Scheduler state and leases, if admitted.

Before any runtime starts, all module sessions, capability grants, storage
bindings, JetStream consumers and Scheduler leases are invalidated or fenced
above the captured generation. A stale process can neither use a restored
credential lease nor publish into a new generation. Any verification failure
leaves the target unpublished and boot remains fail-closed.

### Required evidence before gate admission

`whole_instance_backup_v1` opens only with executable proof of the matrix,
quiesce order, encrypted/signature media validation, wrong/replayed authority
rejection, corrupt/missing/extra/symlink inventory rejection, empty-target
enforcement, generation/lease invalidation and a disposable full restore of
Control Store, Vault, PostgreSQL, required Blob and NATS outbox/inbox replay.
When Blob or Scheduler are enabled, their corresponding conditional evidence is
mandatory. A component-local snapshot, a raw pg_dump or a documentation claim
does not satisfy this ADR.

## Consequences

The existing independent recovery commands remain available but must be named
precisely: they are component recovery, not whole-instance backup. UI and
Gateway do not expose a persistent backup owner endpoint. Backup/restore is
admitted only after the exact coordinator package/ports, test contour and
policy evidence are in place; it is a prerequisite for the first durable
owner, not an exception to that gate.
