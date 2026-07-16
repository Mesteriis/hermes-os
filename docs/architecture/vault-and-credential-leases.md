# Vault и scoped credential leases

Статус решения: Принято
Состояние реализации: Не реализовано
Дата: 2026-07-16

Каноническое решение находится в
[ADR-0223](../adr/ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md).
Этот summary описывает обязательную operational boundary, но не является
доказательством существующего runtime, storage format или security guarantees.
Использование Vault для PostgreSQL credentials уточняет
[ADR-0224](../adr/ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md).

## Назначение

Hermes Vault хранит только небольшой credential material, который нужен
авторизованным module runtimes:

- passwords и app passwords;
- API/client secrets;
- OAuth refresh credentials;
- private provider auth keys и небольшие session credential blobs;
- wrapping keys для integration-owned encrypted session stores.

Vault не является generic key-value, blob, settings или provider session store.
Он не владеет business truth и не зависит от PostgreSQL, PgBouncer, NATS,
provider SDK или owner-specific module packages.

## Process и ownership boundary

Vault имеет exclusive owner `platform/vault` и запускается только как отдельный
verified `managed` OS process:

```text
Kernel supervisor
    ↓ verify / start / quiesce / drain / stop / bounded restart
hermes-vault-runtime
    ├─ hermes-vault-store-sqlcipher
    └─ hermes-vault-keychain-macos
            ↓ sealed CredentialLeaseV1 through capability routing
authorized module runtime
```

Kernel:

- достигает `recovery_only` без Vault;
- вычисляет effective `GrantSet`, supervises lifecycle и fences stale access;
- маршрутизирует только versioned HPKE ciphertext frames;
- видит sanitized state, generation и blocker code;
- не получает root/record keys или credential plaintext;
- не линкует Vault runtime, SQLCipher, crypto или Keychain implementation.

Vault runtime:

- единолично владеет encrypted SQLite path, connections и key hierarchy;
- повторно проверяет signed authorization context до decrypt/unwrap;
- выдаёт process-bound leases только для exact owner, configuration, purpose,
  runtime audience и current grant epoch;
- не интерпретирует provider или domain semantics.

Module runtime зависит только от `hermes-vault-protocol`. Он не получает
database path, SQL, root key, key slots, enumeration API или generic
`GetSecret(secret_ref)`.

Первая версия поддерживает только bundled managed Vault. External Vault,
альтернативная topology или silent implementation fallback запрещены.

## Authorization и transport

Доступ разрешён только как пересечение:

```text
VaultPurposeRequestV1
∩ owner-approved GrantSet
∩ hard Kernel/Vault policy
∩ current runtime session and generation
```

Module descriptor объявляет bounded purpose, secret classes, actions, target
scope и requested TTL без secret value, account label или wildcard scope.
`pending`, `suspended`, `revoked`, wrong audience/purpose и stale epoch
отклоняются до чтения credential payload.

Secret-bearing control/data frames используют `VaultTransportSessionV1` с HPKE.
Kernel/Gateway авторизует и маршрутизирует ciphertext, но не имеет recipient
private key. Secret material никогда не проходит через:

- PostgreSQL или Kernel Control Store;
- NATS, durable events или client SSE;
- settings snapshots;
- argv, environment, logs или filesystem spool.

Это capability routing, а не direct module-to-module socket.

## PostgreSQL credentials

Bootstrap/admin и module runtime database credentials принадлежат Vault, а не
Kernel, Storage Control, PostgreSQL ledger или PgBouncer configuration. Storage
Control и module runtime получают разные exact-purpose, process-bound leases,
fenced registration, runtime, grant, role и storage generations. Storage
Control может resolve plaintext только для bounded bootstrap/role provisioning;
он держит его в памяти минимальное время и zeroize-ит после передачи PostgreSQL
tool/connection.

`StorageBindingV1` содержит opaque PgBouncer endpoint/principal и credential
lease purpose/revision, но не password или Vault record identifier. После
выдачи binding module выполняет business SQL напрямую через PgBouncer; Kernel и
Storage Control не проксируют запросы. Kernel никогда не получает plaintext;
Storage Control не получает module lease и не сохраняет bootstrap/provisioning
plaintext.

Initial `initdb` bootstrap допускает только one-shot password file из Vault
lease: exclusive create в process-private `0700` directory, mode `0600`,
немедленное удаление после open/exit и zeroization buffer. Постоянное хранение
bootstrap secret вне Vault запрещено.

Lease expiry прекращает новую выдачу, но сам по себе не завершает уже открытую
PostgreSQL session. Storage revoke поэтому координирует runtime quiesce,
`NOLOGIN`, PgBouncer alias drain/kill, termination PostgreSQL backends, rotation
credential и повышение epochs до нового binding. Полный lifecycle и ограничения
topology, где PgBouncer является единственным runtime front door, описаны в
[Storage Control Plane](storage-control-plane.md).

## CredentialLeaseV1

Lease привязан как минимум к:

- Vault instance и runtime generation;
- logical owner и opaque configuration instance;
- purpose, actions и exact secret revision;
- module registration и runtime instance audience;
- current `grant_epoch`;
- issued/expiry time.

Initial policy:

- default TTL — 10 минут;
- hard maximum — 1 час;
- resolved material и lease не сохраняются в SQLite;
- `Resolve` является single-use;
- renewal создаёт новый lease и заново проверяет current grants;
- Vault lock/restart/restore или generation change инвалидирует все leases;
- module restart, suspend/revoke или grant epoch change инвалидирует leases этого
  runtime;
- revoke закрывает transport session и блокирует renewal.

Revoke не может стереть bytes, уже скопированные скомпрометированным runtime.
Поэтому он также fences и quiesce/stop-ит затронутый runtime; remote credential
при необходимости отдельно rotates/revokes provider workflow.

## Storage и metadata privacy

Vault использует:

- SQLCipher full-database encryption для schema, indexes и metadata at rest;
- XChaCha20-Poly1305 record envelope для bounded credential payload;
- typed AAD с Vault instance, record, owner, opaque configuration, purpose,
  class, revision, suite и key epoch;
- одну dedicated blocking thread/single-writer actor и bounded typed queue;
- короткие atomic SQLite transactions;
- `DELETE` journal, `synchronous=FULL`, `trusted_schema=OFF`, in-memory temp
  storage и disabled extension loading в V1.

Vault не хранит email, phone, username, provider account ID, display label,
provider URL или arbitrary JSON metadata. Provisioning write-only: client видит
только sanitized configured/revision/expiry/rotation state, но не secret,
record ID или storage path.

Limits V1:

- обычный credential payload — до 64 KiB;
- `session_credential_blob` — до 4 MiB;
- больший или часто изменяемый state остаётся в private integration store;
- Vault выдаёт такому store только scoped `SessionStoreKeyLease`.

## Что не хранится в Vault

- messages, contacts, attachments, documents, media и prompts;
- settings, cursors, checkpoints и provider sequence state;
- retry/reconciliation state, outbox/inbox и Scheduler jobs;
- provider operational projections;
- большие/high-churn TDLib или provider session databases;
- cookies/local storage hidden WhatsApp WebView;
- Owner/device ES256 private keys.

WhatsApp использует OS-managed per-account WebView profile. Большие Telegram и
другие provider session stores принадлежат соответствующей integration и имеют
собственный encryption lifecycle; Vault хранит только wrapping key.

## Key hierarchy и recovery

`VaultRootKey` является случайным 32-byte key. Он wrapped независимыми slots:

- platform slot использует device-only, non-synchronizable macOS Data Protection
  Keychain key;
- recovery slot использует независимый `RecoveryKeyV1`, который владелец хранит
  вне Hermes;
- Owner/device signing key не используется для Vault encryption или wrapping.

Kernel Control Store не содержит Vault keys, slots, secret IDs/bindings или
leases и не шифруется Vault-derived key.

Vault запускается `sealed`; только после разрешённого unlock, SQLCipher
integrity/version checks и создания новой runtime generation capability
становится ready. `sealed` не является crash и не вызывает restart loop.

Backup создаётся только unlocked Vault после fresh owner proof, bounded quiesce
и проверки полученного encrypted package. Restore выполняется только offline
при остановленных Kernel и Vault, explicit data directory, exclusive lock,
interactive confirmation и Recovery Key. Wrong key, corruption или platform key
loss никогда не создают empty Vault и не перезаписывают working key slot.

## Failure и privacy behavior

- Vault process exit допускает только bounded supervised restart.
- Restart создаёт новую generation, сохраняет encrypted records и инвалидирует
  все active leases.
- Integrity/cipher/schema failure даёт `recovery_required` без automatic
  init/reset/restore/fallback.
- Vault failure блокирует только capabilities с credential dependency.
- Kernel, Control Store, recovery surface и modules без Vault dependency
  продолжают работу.
- Logs, errors, health, telemetry, SSE и NATS не содержат secret IDs, purpose,
  provider/account identity, payload length, ciphertext или database path.

## Package boundary

Зафиксированы packages:

```text
hermes-vault-protocol
hermes-vault-key-provider
hermes-vault-runtime
hermes-vault-store-sqlcipher
hermes-vault-keychain-macos
```

Kernel и modules могут зависеть только от public protocol там, где это разрешено
architecture policy. `hermes-vault-key-provider` является private adapter port
Vault owner. Runtime/store/keychain packages не попадают в Kernel или module
compile graphs.

## Состояние реализации

На 2026-07-16 не существуют production `hermes-vault-*` packages,
`VaultTransportSessionV1`, SQLCipher schema/migrations, macOS key adapter,
backup/recovery tooling или conformance tests. Legacy `HostVault` используется
только как evidence и не является implementation template или compatible data
format.

Статус может измениться только после executable dependency guards и targeted
process, crypto, storage, crash, lease, recovery и secret-leak tests из
ADR-0223.
