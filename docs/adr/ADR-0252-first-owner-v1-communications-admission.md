# ADR-0252: `first_owner_v1` admission for Communications

Статус: Принято
Дата: 2026-07-23
Состояние реализации: policy admission открыт, live owner admission остаётся в
работе. Exact owner inventory, module descriptor, settings schema, capability
GrantSet, Storage bundle, NATS routes и owner query route закреплены executable
policy. Managed Docker conformance доказывает owner-local PostgreSQL
backup/restore round trip для canonical Communications evidence через Storage
offline recovery contract. До полного admission остаётся полная проверка
acceptance matrix ниже. Kernel остаётся в честном `module_control_plane`:
отдельное production state `ready` ещё не реализовано.

Зависит от:

- [ADR-0225: phase gates](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md);
- [ADR-0233: backup and restore](ADR-0233-whole-instance-backup-and-fenced-restore.md);
- [ADR-0240: Communications clean-room migration](ADR-0240-canonical-communications-owner-clean-room-migration.md);
- [ADR-0249: Communications Storage profile](ADR-0249-communications-storage-control-v1-admission-profile.md);
- [ADR-0250: Communications NATS profile](ADR-0250-communications-nats-data-plane-v1-admission-profile.md);
- [ADR-0251: client_gateway_v1](ADR-0251-client-gateway-v1-opening-for-owner-contracts.md).

## Gate prerequisites

До policy transition должны быть открыты:

```text
module_control_plane_v1
managed_launch_trust_v1
vault_v1
telemetry_v1
storage_control_v1
nats_data_plane_v1
client_gateway_v1
whole_instance_backup_v1
blob_v1
```

`blob_v1` обязателен, потому что Communications имеет body/media anchors.
Scheduler не является prerequisite, пока Communications не владеет jobs,
schedules или timers.

## First owner

Первый business owner — только `communications`. Exact package inventory:

```text
hermes-communications-ingress
hermes-communications-api
hermes-communications-domain
hermes-communications-persistence
hermes-communications-runtime
```

Integration owners `mail`, `telegram`, `whatsapp` и `zulip` не становятся
частью Communications и не входят в этот owner inventory. Их packages,
admission, storage, credentials и operational contracts остаются отдельными.
Они могут публиковать только typed Communications ingress observations через
свои outbox.

## Owner capabilities

Initial exact capabilities:

- `communications.observe.v1` — event subscription, не client RPC;
- `communications.query.v1` — metadata-only owner query;
- `communications.events.v1` — canonical owner event publication;
- `communications.storage.v1` — owner-local fenced storage;
- `communications.blob.v1` — bounded body/media admission by opaque reference.

Generic provider execution, cross-owner query, workflow promotion, AI context,
arbitrary SQL и generic content API не входят в admission.

## Runtime boundary

`hermes-communications-runtime` является единственным composition root. Он:

- получает managed domain configuration от Kernel;
- открывает owner-local persistence через Storage/Vault lease;
- consumes one approved ingress route;
- выполняет domain decision;
- commits inbox + canonical mutation + outbox atomically;
- публикует exact canonical event bytes;
- обслуживает owner query через managed module client delivery.

Kernel, Gateway и integrations не импортируют Communications implementation.
Communications не импортирует другой domain или integration.

## Последовательность открытия

1. Revalidate ADR-0249 Storage profile.
2. Revalidate ADR-0250 NATS profile.
3. Открыть ADR-0251 `client_gateway_v1`.
4. Открыть `whole_instance_backup_v1` с Communications restore evidence.
5. Запустить signed managed Communications runtime end-to-end.
6. Атомарно обновить policy: owner inventory, exact packages, capabilities,
   dependencies и gate state.
7. Удалить временное ADR-0239 read-only exception.

## Acceptance evidence

- signed managed launch to ready with exact descriptor/settings/schema digests;
- live PostgreSQL/PgBouncer/Vault/NATS/Blob contour;
- integration event → inbox → mutation → outbox → ACK;
- duplicate/replay/outage/revoke/generation fencing;
- owner query through Core Gateway and generated client;
- backup, destructive reset of disposable instance and complete restore;
- compile isolation and no cross-owner source/storage dependencies;
- no legacy, facade, fallback or dual-write production path.

## Rollback

Gate transition is atomic and reversible before production data rollout:
remove owner inventory/capabilities/routes, revoke credentials and bindings,
stop runtime, retain owner data and backup artifacts. После production data
rollout rollback означает disable/revoke и восстановление совместимой версии,
но не автоматическое удаление или downgrade schema.
