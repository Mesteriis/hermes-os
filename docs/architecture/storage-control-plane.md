# Storage Control Plane

Статус: foundation package/Protobuf/AST-admission реализованы; runtime gate закрыт
Дата: 2026-07-16

Полный нормативный contract находится в
[ADR-0224](../adr/ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md).
Это summary не расширяет ADR.

## Topology

```text
Kernel Supervisor
├─ PostgreSQL
├─ PgBouncer
└─ Storage Control
       ├─ cluster/bootstrap
       ├─ roles/grants/budgets
       ├─ migration admission/coordinator
       └─ readiness/reconciliation

module runtime ── StorageBindingV1 + CredentialLease ──> PgBouncer ──> PostgreSQL
```

Kernel владеет process lifecycle, но не SQL implementation. Storage Control
является отдельным managed control-plane process и никогда не проксирует
business queries. Module runtimes ходят только через PgBouncer. Kernel может
работать в `recovery_only` без всего storage stack.

## Packages

```text
backend/src/platform/storage/protocol/     hermes-storage-protocol
backend/src/platform/storage/control/      hermes-storage-control
backend/src/platform/storage/runtime/      hermes-storage-runtime
backend/src/platform/storage/postgres/     hermes-storage-postgres
backend/src/platform/storage/pgbouncer/    hermes-storage-pgbouncer
backend/src/platform/storage/migrations/   hermes-storage-migrations
```

Только protocol является public dependency Kernel/modules. SQL-free control
package владеет use cases/internal ports, runtime только собирает adapters.
Только PostgreSQL adapter и owner persistence packages получают SQL client.
Owner migration bundle передаётся как admitted artifact, поэтому Storage
Control не зависит от всех owner persistence packages.

## Data boundary

Используются один managed PostgreSQL cluster, одна Hermes database и три fixed
schemas:

- `hermes_data` — owner-prefixed business/provider objects;
- `hermes_platform` — technical state, ledgers и shared outbox/inbox;
- `hermes_extensions` — extension-owned objects.

`PUBLIC CREATE` отозван. Runtime `search_path` содержит только `pg_catalog`, SQL
всегда schema-qualified. Каждый durable owner имеет `NOLOGIN` DDL role и
generation-scoped `LOGIN NOINHERIT` runtime principal. Cross-owner SQL/FK
запрещены. Shared technical tables доступны modules только через exact
versioned functions с caller/generation checks; V1 exact allowlist содержит
только append-outbox и accept-inbox functions. Прямой DML запрещён. RLS может
быть defense in depth, но не заменяет grants/function contract.

## Runtime data path

```text
owner mutation + outbox append
              ↓ one local transaction
       shared technical outbox
              ↓ exact bytes
          Event relay → NATS
```

PgBouncer работает в `transaction` mode. Connection budgets действуют в module
client pool, PgBouncer, PostgreSQL role и global PostgreSQL reserve. Pooler не
является единственной security/budget boundary.

`StorageBindingV1` связывает endpoint/principal/budget с exact storage,
runtime, grant и role generations, а также applied migration bundle digest.
Credential приходит только process-bound Vault lease. Revoke повышает epoch,
останавливает новую выдачу, drain/kill-ит PgBouncer pool, завершает PostgreSQL
backends и ротирует role credential до выдачи нового binding.

Пока same-UID module process не ограничен OS socket/network sandbox, policy не
обещает физическую невозможность открыть скрытый direct endpoint. Modules не
получают его address/credential; PostgreSQL grants и role limits остаются
authority, а endpoint isolation требует отдельного conformance evidence.

## Migration data path

```text
owner persistence package
        ↓ package time
canonical StorageBundleV1
        ↓ signed distribution entry / owner-pinned digest
Storage Control admission
        ↓ digest + PostgreSQL AST + owner/grant checks
owner DDL role + direct admin transaction
        ↓
immutable storage ledger + privilege audit
        ↓ exact digest match
owner runtime start
```

Migration V1 является transactional, additive, owner-local и forward-only.
Module runtime не передаёт SQL и не выполняет DDL. Regex architecture guard —
только ранний heuristic; authoritative enforcement требует AST parser,
roles/grants и Testcontainers PostgreSQL/PgBouncer tests.

## Failure ownership

- Storage Control crash перезапускает только Storage Control.
- PgBouncer crash перезапускает только PgBouncer.
- Pool exhaustion не перезапускает PostgreSQL.
- Planned PostgreSQL restart quiesce-ит modules, drain-ит transactions,
  проверяет ledger/roles/grants, повышает storage generation и выдаёт новые
  bindings.
- Automatic reset, restore, down migration и fallback запрещены.

## Implementation status

Сейчас реализованы six-package foundation, `StorageBundleV1` Protobuf,
structural bundle validation и fail-closed AST admission для owner-local
`CREATE TABLE` / `ALTER TABLE … ADD COLUMN`. Managed adapters, Vault lease
delivery, ledger, role/grant reconciliation, distribution trust execution и
PostgreSQL/PgBouncer integration suite ещё должны быть реализованы. Поэтому
`storage_control_v1` остаётся закрытым.
