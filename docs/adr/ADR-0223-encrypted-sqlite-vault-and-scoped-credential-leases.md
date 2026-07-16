# ADR-0223: Encrypted SQLite Vault и scoped credential leases

Статус: Принято
Дата: 2026-07-16
Состояние реализации: Не реализовано; `hermes-vault-*` packages,
`VaultTransportSessionV1`, SQLCipher schema/migrations, macOS platform-key
adapter, backup/recovery tooling и conformance tests ещё не созданы

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213: Конституция кода, ownership и автономность модулей](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0219: Целостность managed modules, distribution manifest и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

Этот ADR уточняет перечисленные решения, но не заменяет их. Он определяет
отдельный Hermes Vault для credential material и не меняет ownership Kernel
Control Store, PostgreSQL, provider operational state, blobs или client/device
identity.

Vault packages/process не входят в `kernel_recovery_only_v1` и открываются
только `vault_v1` после `managed_launch_trust_v1` ADR-0225.

## Контекст

Hermes должен безопасно хранить:

- passwords и app passwords;
- API/client secrets;
- OAuth refresh credentials;
- provider auth keys и небольшие session credential blobs;
- ключи для больших integration-owned encrypted session stores.

Эти данные нужны independently restartable integration runtimes, но не являются
settings, business truth, event payload или общей PostgreSQL state. Kernel
обязан запускаться и сохранять recovery surface без Vault, а падение Vault не
должно останавливать modules, которым credentials не нужны.

Legacy `HostVault` в `references/backend-legacy/` является только evidence. В
нём были полезны отдельный SQLite store, root key вне database, explicit
`locked/unlocked` lifecycle, XChaCha20-Poly1305, случайный nonce, AAD и
zeroization. Переносить реализацию как шаблон нельзя:

- шифровались значения, но `secret_ref`, account, purpose, timestamps и
  manifest metadata оставались plaintext;
- root key напрямую хранился в Keychain;
- общий process мог читать secret по произвольному строковому reference;
- account/purpose/module identity, grant epoch и runtime generation не
  авторизовывались;
- recovery phrase фактически являлась экспортом root key;
- secret и manifest изменялись не одной transaction;
- большие provider session databases смешивались с небольшими credentials.

Новая система сохраняет идею локального Vault, но создаёт новый contract и
storage format без compatibility с legacy database.

## Решение

### Owner и process boundary

Vault имеет exclusive owner `platform/vault`. Он не является частью Kernel,
domain или integration:

~~~text
Kernel supervisor
    ↓ lifecycle, GrantSet context, fencing, opaque routing
hermes-vault-runtime                 отдельный managed OS process
    ├─ hermes-vault-store-sqlcipher
    └─ hermes-vault-keychain-macos
            ↓
authorized module runtime
    получает только process-bound CredentialLeaseV1
~~~

Kernel:

- запускает, quiesce/drain, останавливает и bounded-перезапускает Vault;
- проверяет managed executable по ADR-0219;
- вычисляет effective `GrantSet` и маршрутизирует versioned ciphertext frames;
- видит только sanitized state, generation и blocker code;
- не линкует Vault runtime, SQLCipher, crypto или Keychain implementation;
- не получает `VaultRootKey`, record keys или credential plaintext.

Vault runtime:

- является единственным владельцем encrypted SQLite path и connections;
- unwrap-ит ключи, выполняет crypto/storage операции и выдаёт leases;
- не зависит от PostgreSQL, PgBouncer, NATS, provider SDK или module packages;
- не является generic KV/database service;
- не интерпретирует Mail, Telegram, Zulip, WhatsApp или domain semantics.

Module runtime:

- зависит только от public `hermes-vault-protocol`;
- не получает SQLite path, SQL, key slots, root key или enumeration API;
- получает material только в рамках approved purpose и process-bound lease;
- хранит plaintext в памяти минимально необходимое время и zeroize-ит его при
  stop/revoke/expiry настолько, насколько это поддерживает provider SDK.

Первая реализация Vault является только bundled `managed` process. External
Vault registration и альтернативная implementation/topology запрещены.

### Cargo packages

Первая package topology фиксирована:

~~~text
backend/src/platform/vault/protocol/
    hermes-vault-protocol
    platform:vault:contract

backend/src/platform/vault/key_provider/
    hermes-vault-key-provider
    platform:vault:contract

backend/src/platform/vault/runtime/
    hermes-vault-runtime
    platform:vault:runtime
    component: vault_service

backend/src/platform/vault/store_sqlcipher/
    hermes-vault-store-sqlcipher
    platform:vault:persistence

backend/src/platform/vault/keychain_macos/
    hermes-vault-keychain-macos
    platform:vault:implementation
~~~

`hermes-vault-key-provider` является внутренним adapter port владельца Vault.
Kernel, modules и Gateway не зависят от него. Новый Vault package или platform
adapter требует изменения этого ADR и executable policy.

### Threat boundary

Первая версия защищает от:

- offline theft Vault database, journal, temporary migration files или backup;
- чтения account/provider metadata из plaintext SQLite;
- соседнего module без grant либо с другим account/purpose;
- replay stale lease после restart, revoke или epoch change;
- случайного раскрытия plaintext внутри Kernel routing/logging;
- потери platform wrapping key при наличии recovery package и Recovery Key.

Первая версия не обещает защиту от:

- полного compromise Vault process, пока он unlocked;
- host root/administrator или полного compromise owner OS account;
- malicious runtime, который уже получил plaintext и скопировал его;
- provider-side compromise после успешной authentication.

Lease ограничивает выдачу и lifetime авторизации, но не может отозвать bytes,
уже скопированные чужим process. Revoke поэтому включает fencing и
quiesce/stop затронутого runtime, а provider credential при необходимости
отзывается или rotates у внешнего provider.

### Encrypted SQLite profile

Vault использует SQLCipher full-database encryption и отдельный
XChaCha20-Poly1305 envelope для credential payload.

SQLCipher скрывает schema, indexes и metadata at rest, а record envelope:

- связывает ciphertext с exact owner/configuration/purpose/class/revision;
- не позволяет переставить ciphertext между logical records;
- даёт независимый record/key epoch и controlled crypto-suite migration.

Initial SQLite profile:

~~~text
journal_mode = DELETE
synchronous = FULL
foreign_keys = ON
trusted_schema = OFF
temp_store = MEMORY
extension loading = disabled
ATTACH = forbidden
single writer actor
~~~

Правила:

- local parent directory имеет mode `0700`, files `0600`;
- SQLCipher key передаётся через raw-key API, а не interpolated SQL;
- SQLite connection принадлежит одной dedicated blocking thread/actor;
- requests typed, bounded и имеют deadline;
- mutation выполняется одной короткой transaction;
- raw SQL, row types и file paths не пересекают persistence boundary;
- unknown cipher/schema/record major version fails closed;
- schema и embedded migrations принадлежат
  `hermes-vault-store-sqlcipher`;
- migration возможна только после успешного unlock;
- destructive downgrade, plaintext export и automatic fallback запрещены.

WAL не включается в первой версии: low-write Vault не получает достаточного
выигрыша, чтобы оправдать отдельный checkpoint/sidecar lifecycle. Изменение
journal mode требует crash, plaintext-leak и backup conformance tests.

[SQLCipher](https://www.zetetic.net/sqlcipher/design/) используется как
page-encryption implementation boundary, а не как generic database dependency
Kernel или modules.

### Key hierarchy

~~~text
PlatformWrappingKey                 RecoveryKeyV1
macOS Data Protection Keychain      хранится владельцем вне Hermes
          │                              │
          └──── authenticated KeySlotV1 ─┘
                            ↓
                     VaultRootKey
                 ┌──────────┴──────────┐
          SQLCipher key          record-domain keys
~~~

- `VaultRootKey` — 32 random bytes из OS CSPRNG.
- `PlatformWrappingKey` — отдельные 32 random bytes в non-synchronizable,
  device-only macOS Data Protection Keychain item с dedicated access group.
- Owner/device ES256 signing key ADR-0218 не используется для Vault encryption
  или wrapping.
- `RecoveryKeyV1` — независимые 32 random bytes; Hermes показывает их один раз
  и не сохраняет plaintext.
- Human representation `RecoveryKeyV1` — 24-word BIP-39 entropy encoding с
  checksum и English word list. Используется только entropy-to-mnemonic
  encoding; BIP-39 seed/PBKDF2/passphrase semantics не используются. Формат
  следует только части entropy/mnemonic стандарта
  [BIP-39](https://bips.dev/39/).
- User-created password/passphrase unlock в V1 отсутствует. Его появление
  потребует отдельного Argon2id contract.
- Wrapping suite V1 — XChaCha20-Poly1305 с random 24-byte nonce.
- SQLCipher key и record-domain keys выводятся через HKDF-SHA-256 с
  `VaultInstanceId`, distinct fixed info labels и key epoch.
- Credential record использует unique random nonce; AAD включает
  `VaultInstanceId`, record ID, logical owner, opaque configuration instance,
  purpose, class, revision, suite и key epoch.
- Root/key plaintext существует только в unlocked Vault memory, не
  сериализуется и zeroize-ится при lock/stop/failure.

`vault.anchor` содержит только version, `VaultInstanceId` и authenticated
`KeySlotV1` records. Он принадлежит Vault, а не Kernel Control Store. Key slot
содержит kind, suite, key epoch, nonce и wrapped root key, но не credential
metadata.

Platform wrapping slot и recovery slot позволяют:

- менять platform key без database rewrite;
- восстанавливать Vault на новом устройстве;
- менять recovery key без изменения credential records;
- не делать recovery phrase равной root key.

### State machine и unlock policy

Vault использует явные состояния:

~~~text
uninitialized
    → sealed → unlocking → unlocked
                    └────→ recovery_required

unlocked ↔ rotating
unlocked → quiescing → stopped
sealed / recovery_required → stopped
~~~

Unlock modes:

- `platform_auto` — desktop default после входа пользователя в разблокированную
  OS session;
- `owner_presence` — hardened mode, требующий Touch ID/system credential при
  каждом unlock;
- `manual_local` — headless/local interactive unlock через поддержанный
  platform adapter;
- `recovery_offline` — stopped Kernel/Vault, exclusive lock и
  `RecoveryKeyV1`.

Initialize, recovery export/import, root rotation и изменение recovery slot
всегда требуют fresh operation-bound owner proof и platform user presence.
Apple Keychain `userPresence` допускает биометрию или device passcode:
[Apple Keychain access control](https://developer.apple.com/documentation/security/secaccesscontrolcreateflags/userpresence).

`platform_auto` является non-secret Vault-owned operator setting. Она
применяется только при trustworthy Control Store. Unavailable/untrusted
Control Store запрещает online unlock и lease issuance независимо от
сохранённого desired mode.

Startup:

1. Kernel достигает `recovery_only` без Vault.
2. При trustworthy Control Store supervisor проверяет exact Vault executable.
3. Vault process запускается в `sealed`.
4. Разрешённый unlock unwrap-ит `VaultRootKey`.
5. Vault открывает SQLCipher и проверяет cipher/schema/integrity.
6. Создаётся новая `vault_runtime_generation`.
7. Только затем capability становится ready и выдаёт leases.

Vault failure блокирует только methods/capabilities с Vault dependency. Kernel,
Control Store, recovery surface и modules без credential dependency продолжают
работать.

### Secret record и metadata privacy

Logical record:

~~~text
SecretRecordV1
  secret_record_id
  logical_owner_id
  opaque_configuration_instance_id
  vault_purpose_id
  secret_class
  secret_revision
  key_epoch
  state
  not_before?
  expires_at?
  bounded encrypted payload
~~~

Closed `secret_class` V1:

- `provider_credential`;
- `oauth_refresh_credential`;
- `session_credential_blob`;
- `platform_credential`;
- `session_store_key`.

Record states:

- `active`;
- `retiring`;
- `revoked`;
- `tombstoned`.

Vault не хранит email, phone, username, provider account ID, display label,
provider URL или arbitrary JSON metadata. Integration owner хранит связь
своего account с opaque `configuration_instance_id` и purpose. Replacement
registration не наследует credential binding или grants автоматически.

Generic `ListSecrets`, `GetSecret(secret_ref)` и client read-back отсутствуют.
Owner provisioning является write-only: после записи client видит только
`configured`, revision, expiry/rotation status и sanitized blocker.

Health, logs, errors, SSE и NATS не содержат record ID, purpose, account
metadata, counts по providers, ciphertext/plaintext length или database path.

### Разделение credentials и provider session state

Vault хранит:

- passwords, app passwords, API/client secrets;
- OAuth refresh credentials;
- private auth keys/cookies, если они являются небольшим credential material;
- небольшой opaque session blob, достаточный для impersonation;
- wrapping keys для integration-owned encrypted session stores.

Vault не хранит:

- messages, contacts, attachments, documents, media или prompts;
- settings, cursors, checkpoints, pts/qts/seq, mailbox position;
- retry/reconciliation state, outbox/inbox или jobs;
- provider operational projections;
- большие/high-churn TDLib/provider session databases;
- экспортированные cookies/local storage hidden WhatsApp WebView.

Limits V1:

- обычный credential payload — не более 64 KiB;
- `session_credential_blob` — не более 4 MiB;
- больший или часто изменяемый state остаётся в private integration store;
- Vault выдаёт ему только `SessionStoreKeyLease`.

WhatsApp hidden WebView использует отдельный OS-managed per-account profile и
не экспортирует cookies/storage в generic Vault. Mail passwords/OAuth refresh
credentials, Telegram auth material и Zulip API key следуют тому же общему
Vault contract без provider-specific API внутри Vault.

PostgreSQL/PgBouncer bootstrap, admin и runtime credentials являются
`platform_credential` ADR-0224. Только Storage Control получает narrow
`create`/`replace_cas`/`retire` actions и exact-purpose `resolve` для bounded
bootstrap/role provisioning; plaintext остаётся в его memory только до
передачи PostgreSQL tool/connection и zeroize. Module runtime получает другой
scoped `resolve` для exact `StorageBindingV1`. Vault не знает SQL, schemas,
table names или PgBouncer admin model и не является database session revocation
service.

### Descriptor request и authorization

`ModuleDescriptorV1` может запросить:

~~~text
VaultPurposeRequestV1
  purpose_id
  allowed_secret_class[]
  actions[]
  target_scope
  requested_lease_ttl
~~~

Actions V1:

- `resolve`;
- `create`;
- `replace_cas`;
- `retire`;
- `delete`;
- `issue_session_store_key`.

Purpose ID является stable bounded identifier владельца contract. Wildcard
owner/resource, arbitrary provider account label, email/phone и secret
reference в descriptor запрещены.

Vault operation разрешена только как пересечение:

~~~text
VaultPurposeRequestV1
∩ owner-approved GrantSet
∩ hard Kernel/Vault policy
∩ current runtime session and generation
~~~

Vault повторно проверяет signed authorization context перед decrypt/unwrap.
`pending`, `suspended`, `revoked`, wrong audience/purpose и stale epoch
отклоняются до чтения credential payload.

### CredentialLeaseV1

~~~text
CredentialLeaseV1
  lease_id
  vault_instance_id
  vault_runtime_generation
  secret_revision
  logical_owner_id
  configuration_instance_id
  purpose_id
  actions
  audience_module_registration_id
  audience_runtime_instance_id
  grant_epoch
  issued_at
  expires_at
  single_resolve
  sealed_material
~~~

Для storage `configuration_instance_id`, purpose и `secret_revision` связаны с
exact current `StorageBindingV1`; смена storage/runtime/grant/role generation
создаёт новую binding/credential revision. Vault сверяет exact opaque binding,
audience, purpose, revision и grant epoch до delivery, а Storage Control
отдельно проверяет storage/role generations и завершает session fencing
ADR-0224. Vault не интерпретирует database semantics.

Initial policy:

- default TTL — 10 минут;
- hard maximum — 1 час;
- lease и resolved material не сохраняются в SQLite;
- `Resolve` выполняется не более одного раза;
- renewal создаёт новый lease и заново проверяет current `GrantSet`;
- Vault restart/lock/restore или generation change инвалидирует все leases;
- module restart, suspend/revoke или grant epoch change инвалидирует его
  leases;
- revoke закрывает transport session и блокирует renewal;
- истечение/revoke Vault lease запрещает дальнейший resolve/renewal, но не
  считается отзывом уже открытой SQL session; полный role/PgBouncer/PostgreSQL
  session fencing подтверждает Storage Control ADR-0224;
- secret material никогда не проходит через NATS, durable events, SSE,
  settings, Control Store, argv, environment, logs или filesystem spool.

### Secret-bearing transport

`VaultTransportSessionV1` использует standard HPKE
[RFC 9180](https://www.rfc-editor.org/rfc/rfc9180.html) suite:

~~~text
DHKEM(X25519, HKDF-SHA256)
HKDF-SHA256
ChaCha20Poly1305
~~~

HPKE context `info` и AAD связывают frame с:

- `vault_runtime_generation`;
- authenticated owner device session или `ModuleRegistrationId`;
- `runtime_instance_id`;
- `grant_epoch`;
- request ID;
- operation digest;
- direction и protocol major.

Kernel/Gateway авторизует и маршрутизирует HPKE ciphertext, но не имеет
recipient private key и не видит plaintext. Module-to-module socket не
появляется: transport остаётся частью versioned capability routing.

Vault transport keypair ephemeral и привязан к одной runtime generation.
Replay, wrong direction/context, unknown suite/major и malformed ciphertext
fail closed до credential mutation или delivery.

Owner provisioning использует тот же sealed payload boundary. Tauri/Android
host adapter выполняет HPKE operation; browser business API не получает Vault
root/platform keys и не имеет generic secret read method.

### Mutations и rotation

Public Vault control/data operations:

~~~text
Status
Initialize
Unlock
Lock
PutCredential
ReplaceCredential(expected_revision)
RetireCredential
DeleteCredential
IssueLease
RenewLease
RevokeLease
CreateBackup
RotatePlatformSlot
RotateRecoverySlot
RotateRootKey
Quiesce
Drain
Stop
~~~

- `Initialize` разрешён только pristine Vault instance с fresh owner proof.
- Module write требует отдельного scoped action; resolve grant не даёт write.
- `ReplaceCredential` использует compare-and-swap по revision.
- Provider credential rotation выполняет integration/provider workflow, а
  Vault только атомарно хранит versions.
- Old/new credential overlap явный, bounded и не включается автоматически.
- `DeleteCredential` создаёт tombstone; physical secure erase SQLite page,
  filesystem snapshot или старого backup не обещается.

Три независимых rotation:

1. credential revision через CAS и explicit retire;
2. platform/recovery wrapping slot без database rewrite;
3. Vault root/SQLCipher/record keys через explicit
   `quiesce → encrypted copy/rekey → verify → atomic swap`.

Automatic rollback/fallback после rotation запрещён. Failed rotation сохраняет
однозначно старый либо проверенный новый generation.

### Backup и recovery

Backup:

- выполняется только unlocked Vault;
- требует fresh owner proof и platform user presence;
- bounded-quiesce-ит writes;
- использует SQLCipher-compatible SQLite backup/export, а не `cp` открытого
  файла;
- включает encrypted database snapshot, `vault.anchor`, schema/cipher/key
  epochs и authenticated manifest;
- не включает `RecoveryKeyV1`;
- проверяется пробным unwrap/open/integrity check до публикации.

SQLite Online Backup API создаёт consistent snapshot работающей database:
[SQLite Backup API](https://www.sqlite.org/backup.html).

Restore:

- выполняется только offline при остановленных Kernel и Vault;
- требует explicit `--data-dir`, exclusive instance lock, local interactive
  confirmation и `RecoveryKeyV1`;
- проверяет package/anchor/DB integrity до mutation;
- создаёт новый platform wrapping slot;
- повышает Vault generation и инвалидирует все leases;
- не восстанавливает Kernel grants, OwnerAuthority или device identity;
- не rebind-ит replacement registration к старым secrets автоматически.

Recovery Key даёт только decrypt authority конкретного Vault backup. Он не
является OwnerAuthority, client session или module grant.

Wrong recovery key, missing platform key, corruption либо incompatible version
никогда не создают empty Vault и не перезаписывают working Keychain slot.
Состояние становится `sealed` или `recovery_required` до explicit action.

### Failure, privacy и observability

- Process exit допускает bounded restart по ADR-0203.
- Restart создаёт новый runtime generation и не сохраняет leases.
- `sealed` не является crash и не запускает restart loop.
- Integrity/cipher/schema failure становится `recovery_required` без
  automatic init/reset/restore.
- Core dumps Vault отключены; memory locking используется where supported.
- Secret buffers используют non-clone secret types и best-effort zeroization.
- Public errors typed и redacted; raw crypto/SQL/Keychain errors остаются
  внутри bounded mapping без values/paths/account metadata.
- Telemetry содержит state transition, generation, duration и reason code, но
  не secret IDs, purpose, payload length или account/provider identity.
- Automated tests используют только synthetic marker bytes и platform test
  adapters; live provider credentials запрещены.

## Отклонённые варианты

### Хранить credentials в Kernel Control Store

Отклонено: создаёт boot cycle, расширяет compromise Kernel и смешивает
technical control state с secret material.

### Хранить credentials в PostgreSQL

Отклонено: Vault должен работать независимо от PostgreSQL, а module roles,
backup и query surfaces расширяют область доступа.

### Встроить Vault implementation в Kernel process

Отклонено: root key/plaintext попадают в общий failure/compromise domain и
исчезает independently restartable boundary.

### Plain SQLite только с ciphertext value columns

Отклонено: schema, manifest, account/purpose metadata, journal и access pattern
артефакты остаются шире необходимого. Full-database encryption является
основным at-rest boundary.

### Только SQLCipher без record envelope

Отклонено: page encryption не задаёт typed AAD, record revision/key epoch и
защиту от semantic ciphertext swapping внутри storage implementation.

### Общий `read_secret(secret_ref)`

Отклонено: строковый reference не доказывает module, account, purpose, epoch
или process audience и создаёт enumeration confused-deputy API.

### Recovery phrase равна Vault root key

Отклонено: root нельзя независимо rotate/re-wrap, а leak recovery value сразу
становится прямым DB key material.

### Один Vault blob для любой provider session

Отклонено: high-churn databases превращают Vault в generic blob/session store,
увеличивают write contention и связывают provider lifecycle с secret DB.

### User-created password как default unlock

Отклонено в V1: random platform/recovery keys сильнее и не требуют выбора KDF
параметров. Password-based portable profile требует отдельного решения.

### File-backed production key или silent platform fallback

Отклонено: release build без поддержанного platform key adapter обязан fail
closed, а не сохранять wrapping/root key рядом с database.

## Проверка решения

До изменения `Состояние реализации` обязательны:

- Kernel boot/recovery без Vault;
- Vault является отдельным verified managed process;
- Vault crash не завершает Kernel или независимые modules;
- restart создаёт новую generation и инвалидирует leases;
- `pending`/`suspended`/`revoked`/stale epoch denied до decrypt;
- wrong registration/runtime/account-purpose audience отклоняется;
- duplicate/replayed `Resolve` отклоняется;
- TTL/renew/revoke и hard maximum проверяются fake clock;
- plaintext markers отсутствуют в DB, journal, temp migration и backup bytes;
- tampered anchor, key slot, SQLCipher page, record nonce/AAD/ciphertext fail
  closed без overwrite;
- wrong Recovery Key не меняет Keychain, anchor или database;
- platform key loss даёт `sealed`, не auto-initialize;
- recovery restore проходит на новом host adapter и повышает generation;
- fault injection для put/replace/delete/migration/backup/rekey/atomic swap;
- concurrent requests проходят single-writer bounded queue без partial state;
- CAS conflict не теряет active credential revision;
- key rotation поддерживает mixed record epochs только в declared migration
  window;
- revoke quiesce/stop-ит affected runtime и не обещает remote side-effect undo;
- Control Store не содержит keys, slots, secret IDs/bindings или leases;
- settings не содержат secret values/references/bindings;
- events/NATS/SSE/logs/errors/health/crash reports не содержат secret/private
  metadata;
- Kernel и modules не зависят от Vault runtime/store/key-provider adapter;
- Vault не зависит от PostgreSQL/NATS/provider/module packages;
- release build не поддерживает file-backed wrapping key;
- automated suite не использует live accounts.

Static architecture policy доказывает package ownership, dependency direction,
forbidden carriers и declared lifecycle/lease invariants. SQLCipher, crypto,
Keychain, IPC, memory, crash, backup и recovery guarantees доказываются только
production conformance/integration tests.

## Последствия

Положительные:

- Kernel остаётся bootable и diagnosable без Vault;
- compromise или crash integration не открывает credentials соседних modules;
- metadata и secret values защищены at rest;
- restart/revoke имеют typed generation/epoch fencing;
- realtime integrations получают automatic desktop unlock без постоянных UI
  prompts;
- recovery не экспортирует root key и отделено от OwnerAuthority;
- Telegram/Mail/Zulip и будущие integrations используют один capability
  contract без общего provider abstraction.

Стоимость:

- появляется отдельный managed process, encrypted SQLite format и key hierarchy;
- нужны native Keychain adapter и platform-specific tests;
- dual SQLCipher + record AEAD требует crypto conformance и rotation tooling;
- HPKE transport и sealed provisioning усложняют IPC;
- большие provider session stores требуют собственного encryption lifecycle;
- physical secure erase одной записи или старого backup не обещается;
- Linux/Windows/headless platform-key adapters не считаются поддержанными до
  отдельной реализации и conformance.
