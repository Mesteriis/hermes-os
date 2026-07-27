# ADR-0301: Bundled module discovery и owner-authorized development admission

Статус: Принято
Дата: 2026-07-27
Состояние реализации: частично реализовано. Generic bundled proposal,
owner-authorized development assembly, exact signed admission, platform и
Communications/Attachment Security readiness, Gateway client realtime и
provider setup surfaces реализованы. `bundled_artifact_proposal_v1` и
`loopback_full_stack_runtime_admission_v1` закрыты для loopback development
profile с учётом ADR-0302: provider runtime с обязательными Settings
показывается как `unconfigured` и запускается только после account setup.
`provider_account_experience_v1` остаётся открытым до live readiness и полного
replace/retire/delete UI каждой integration.

Уточняет:

- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219: Целостность managed modules и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0221: ModuleDescriptorV1 и capability lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0224: Storage Control Plane и owner-scoped PostgreSQL](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0300: Loopback full-stack development assembly](ADR-0300-loopback-full-stack-development-assembly.md).

## Контекст

Текущий Kernel умеет:

- проверять подписанный installed `DistributionManifestV1`;
- bind/reserve/start уже существующую approved registration;
- запускать signed platform foundation;
- принимать external pending registration через отдельный runtime protocol.

Но bundled managed artifact нельзя корректно впервые провести от installed
manifest до pending registration через generic owner-control contract.
Production owner-control требует уже известный `registration_id`, а
development integration tests создают registrations прямым вызовом private
Control Store. Test fixture не является assembly и не может быть runtime
dependency `make dev`.

Из-за этого Kernel/Gateway/Vite могут быть healthy, пока Vault, Storage,
Communications и integrations остаются `runtime_status_not_admitted`.
Capability-driven frontend обязан в таком состоянии скрывать provider routes и
account provisioning. Раскрыть их UI-флагом означало бы создать fake
availability.

## Решение

### Generic bundled artifact proposal

Core получает owner-private operation:

```text
ProposeBundledManagedArtifactV1
```

Request содержит только:

- current owner session;
- exact `artifact_id` из selected installed distribution;
- expected distribution ID и generation;
- idempotency key.

Path, executable bytes, descriptor bytes, digest, module/provider identity,
capability list и registration ID клиентом не передаются. Kernel:

1. повторно проверяет signed manifest и exact artifact bytes;
2. требует `module_runtime` artifact с descriptor и optional settings schema;
3. декодирует descriptor через общий registration policy;
4. создаёт opaque pending registration и сохраняет exact descriptor requests;
5. возвращает bounded registration summary и requested capability IDs;
6. при повторе exact idempotency key возвращает тот же receipt;
7. при отличающемся descriptor/distribution fail closed, не наследуя approval.

Operation не одобряет capability, не создаёт Storage/Vault/Event grants, не
bind-ит release и не запускает process.

### Owner authorization остаётся отдельной

После proposal нужны отдельные owner-authorized операции:

```text
approve exact requested capabilities
bind exact bundled artifact
admit exact StorageBundleV1 when declared
issue current-generation Storage binding
reserve managed runtime
start reserved domain | integration | engine runtime
```

Каждый шаг сохраняет собственный durable receipt и fence. Registration,
approval, distribution binding, storage binding, settings revision и observed
runtime readiness остаются разными authority.

Kernel не импортирует Communications, Mail, Telegram, WhatsApp, Zulip или
Attachment Security packages. Он видит только generic manifest artifacts,
validated descriptors и typed platform requests. Integration не импортирует
domain; event and public-contract routing остаются единственным cross-owner
data path.

### Development assembly plan

ADR-0300 получает отдельный versioned assembly plan:

```text
loopback_full_stack_dev_plan_v1
```

Plan принадлежит development assembly, не Kernel и не любому domain или
integration. Он перечисляет exact artifact IDs и dependency order:

```text
signed local distribution
  -> Vault / Blob / Telemetry / Storage / Events / Scheduler
  -> Communications
  -> Attachment Security
  -> Mail
  -> Telegram
  -> WhatsApp
  -> Zulip
  -> Gateway client realtime
  -> Vite/browser
```

Каждый runtime остаётся отдельным process и unit сборки. Assembly только
координирует public Kernel contracts и readiness.

Явный запуск `make dev` разрешает development operator открыть session
существующим file-backed development owner signer и выполнить exact plan. Это
не implicit Kernel boot approval: без assembly invocation Kernel не предлагает,
не одобряет и не запускает owner modules.

Development plan:

- работает только с literal loopback listeners и explicit absolute dev data
  directory;
- использует отдельный locally signed distribution, materialized существующим
  release compiler;
- не принимает arbitrary artifact IDs, paths или capability overrides из
  browser/environment;
- idempotently повторяет exact receipts и fail closed при drift;
- не создаёт provider accounts, tokens, passwords, cookies или sessions;
- не записывает secrets в Control Store, plan, argv, logs или frontend bundle;
- не объявляет surface available до current settings и runtime readiness.

### Provider account boundary

После runtime admission provider UI становится видимым, но account появляется
только через owner-authorized provider contract:

```text
provider settings
  -> fresh owner proof
  -> write-only Vault provisioning
  -> integration-owned credential binding
  -> provider-specific authorization/session
  -> sanitized account readiness
```

Mail password/Gmail OAuth, Telegram authorization, WhatsApp hidden WebView
session и Zulip credential flows остаются четырьмя integration-owned
experiences. Development assembly не создаёт тестовую учётку и не знает
credential plaintext.

## Phase gates

### `bundled_artifact_proposal_v1`

Gate закрывается только при наличии:

1. typed owner-control request/response и canonical validation;
2. exact installed manifest verification перед proposal;
3. idempotent pending registration receipt;
4. tests на wrong distribution/generation, non-module artifact, digest drift,
   duplicate proposal и absence of implicit approval;
5. architecture test, что Kernel не содержит owner-specific artifact IDs.

### `loopback_full_stack_runtime_admission_v1`

Gate закрывается только при наличии:

1. locally signed distribution из production release compiler;
2. generic development operator, использующего только owner/control public
   contracts;
3. live platform, Communications и integration process readiness;
4. active Storage/Event/Vault bindings и client realtime;
5. Gateway bootstrap, показывающего Communications/Mail/Telegram/WhatsApp/Zulip
   available только после runtime readiness;
6. interrupt/restart evidence без orphan processes, leaked proof/signing key
   или повторной регистрации.

### `provider_account_experience_v1`

Каждая integration закрывает gate независимо:

1. non-secret settings form;
2. write-only fresh-proof credential/session provisioning;
3. sanitized account list/readiness;
4. replace/retire/delete semantics;
5. live provider-specific evidence либо честный unavailable state.

## Последствия

- `make dev` сможет означать реальный ensemble, а не только healthy transport.
- Kernel получает недостающий generic admission seam без знания доменов и
  providers.
- Development assembly остаётся отдельной единицей сборки и не становится
  production release или business owner.
- Provider screens не раскрываются до настоящего capability/runtime admission.
- Добавление account не смешивается с module installation и никогда не
  превращается в seed/fake data.
