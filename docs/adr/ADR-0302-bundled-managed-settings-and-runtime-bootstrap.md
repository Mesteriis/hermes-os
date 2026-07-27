# ADR-0302: Bundled managed Settings и первый runtime bootstrap

Статус: Принято
Дата: 2026-07-27
Состояние реализации: частично реализовано. Exact schema admission, signed
defaults, blocked initial snapshot, initial managed integration apply и
development ensemble реализованы. Provider-owned Add Account surfaces для
Mail, Telegram, WhatsApp и Zulip, write-only credential provisioning
capabilities и generated lifecycle clients реализованы и прошли static/browser
validation. Live provider readiness конкретной учётной записи требует реальных
owner credentials и не выводится из наличия ADR или формы.

Уточняет:

- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219: Целостность managed modules и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0222: Kernel Settings Registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0236: Integration owners и configuration instances](ADR-0236-integration-owners-protocol-adapters-and-configuration-instances.md);
- [ADR-0292: Managed integration Settings apply](ADR-0292-managed-integration-settings-apply-and-credential-binding.md);
- [ADR-0301: Bundled module discovery и development admission](ADR-0301-bundled-module-discovery-and-development-admission.md).

## Контекст

Generic bundled proposal уже создаёт pending registration из exact descriptor,
а owner отдельно approve-ит capabilities и bind-ит signed executable. Однако
текущий binding сохраняет только executable/descriptor digests:

- связанный `SettingsSchemaV1` из того же signed artifact не попадает в
  Settings Registry;
- browser catalog поэтому честно возвращает `module_not_registered`;
- первый managed integration launch требует уже current settings snapshot;
- обычный replacement path ADR-0292 требует уже работающий predecessor;
- integration runtime нельзя запустить впервые, а без runtime provider account
  lifecycle и operational routes остаются недоступны.

Раскрытие route через frontend flag создало бы fake availability. Передача
settings или credentials через development assembly превратила бы assembly в
integration/platform implementation.

## Решение

### Exact bundled Settings admission

Owner-authorized bind одного `module_runtime` artifact включает проверку
связанного immutable settings contract:

```text
verified signed artifact
  -> exact descriptor registration binding
  -> exact SettingsSchema binding
  -> initial typed snapshot materialization
  -> managed executable binding
```

Kernel берёт descriptor/schema bytes только из повторно проверенного installed
distribution. Клиент и development assembly не передают эти bytes, schema
digest, provider settings или defaults.

Schema admission является exact-idempotent:

- повтор той же registration, schema major/revision, digest и exact bytes
  возвращает существующее состояние;
- другая schema под той же registration fail closed;
- частичный сбой между schema и executable binding безопасно продолжается
  повтором exact owner operation.

### Initial snapshot не является replacement

Для новой registration с состоянием:

```text
desired_revision = 0
effective_revision = 0
apply_state = current
```

Settings Registry атомарно materializes revision `1`. Единственный источник
начальных значений — typed `default_value` внутри exact signed
`SettingDefinitionV1`. Kernel не знает provider semantics и не придумывает
значения из environment, assembly, UI или module identity.

Initial materialization:

- валидирует каждый signed default против exact типа definition;
- создаёт snapshot в canonical schema order;
- если schema пуста или все definitions имеют default, записывает snapshot и
  `desired = effective = 1`, `apply_state = current`;
- если хотя бы один definition не имеет default, записывает только available
  defaults и `desired = 1`, `effective = 0`,
  `apply_state = blocked_config`,
  `sanitized_reason_code = required_settings_missing`;
- записывает snapshot и revision state одной Control Store transaction;
- допускается только до первого desired mutation;
- exact repeat возвращает revision `1`;
- конфликтующий snapshot или любое non-initial state отклоняется.

Это configuration baseline без provider credential и без утверждения provider
readiness. `desired = 1/effective = 0` является честным admitted, но ещё не
configured состоянием. Последующие owner mutations проходят обычную validation;
первая successful apply не требует работающего predecessor, а дальнейшие
изменения используют replacement path ADR-0292.

### Первый managed runtime start

После exact release/schema/snapshot admission assembly отдельно:

1. резервирует runtime generation;
2. получает exact owner-scoped Storage binding;
3. вызывает type-specific start:
   `domain | engine | integration`;
4. для integration передаёт только stable opaque
   `configuration_instance_id`;
5. запрашивает host bridge только для descriptor capability, которому он
   действительно нужен;
6. не запускает integration с `blocked_config/effective = 0`, а возвращает
   exact launch state `unconfigured`;
7. проверяет observed runtime readiness отдельно от accepted launch receipt.

Domain, engine и integration не объединяются в один runtime или package.
Development assembly хранит только opaque registration/capability/configuration
identities и порядок композиции.

### Provider account bootstrap остаётся integration-owned

First-party client предоставляет account setup даже когда provider operational
surface ещё недоступна. Setup принадлежит integration frontend и использует
существующие typed owner surfaces:

```text
non-secret integration Settings
  -> write-only Vault provisioning where credential нужен до startup
  -> provider-neutral first supervised apply
  -> write-only Vault provisioning where runtime должен сначала создать binding
  -> integration-owned account bind / OAuth / authorization
  -> supervised credential activation/restart where required
  -> integration-owned readiness query
```

Kernel/Gateway не создают Mail, Telegram, WhatsApp или Zulip account и не
интерпретируют provider settings. Communications domain не участвует в account
lifecycle. Credential bytes остаются только в sealed Vault ceremony; provider
session state остаётся у integration owner.

Provider runtime descriptor обязан отдельно объявлять:

- required runtime `Resolve` capability для каждого credential purpose;
- optional owner-facing provisioning capability с exact purpose, secret class
  и только `Create/ReplaceCas`, если account setup должен создавать credential;
- provider lifecycle capability для binding/retire, если binding живёт в
  integration storage.

Provisioning capability не даёт runtime право читать новый секрет, а runtime
Resolve capability не даёт frontend право создавать или заменять его. Это
разные approval units.

Write-only owner provisioning contract допускает только явно перечисленные
bounded secret classes. Для Telegram это включает отдельный
`SessionStoreKey`; он не смешивается с provider credential, OAuth token или
bulk session blob и сохраняет собственный Vault class code.

Browser и desktop используют один generated Gateway/provider contract.
Credential plaintext не отправляется handwritten REST или generic Settings:
его sealing выполняет first-party secure provisioning host. Если конкретный
client profile не имеет secure host, non-secret setup остаётся видимым, но
credential step fail closed с sanitized reason вместо отправки plaintext через
loopback API.

## Kernel/Core agreement

Kernel/Core согласуют:

- registration, capability, release и runtime generation identities;
- exact schema digest и desired/effective revision;
- opaque configuration instance;
- owner authorization, Storage/Vault/Event fences;
- accepted launch и observed readiness как разные состояния.

Kernel/Core не согласуют:

- email, phone, realm, username или provider account ID;
- password, token, OAuth verifier/code или session bytes;
- integration tables, provider commands или Communications business payload;
- UI form composition.

## Units of assembly

```text
Kernel bundled contract admission
  exact signed descriptor/schema/executable binding

Kernel Settings Registry
  initial snapshot transaction and later revision lifecycle

Kernel runtime lifecycle
  type-specific reservation, launch and readiness

development assembly
  owner-authorized composition of opaque identities only

integration runtime
  provider-owned account lifecycle and operational state

frontend integration surface
  generated provider contract composition
```

Assembly не является domain, integration или runtime. Integration не импортирует
Communications implementation; Communications получает только typed events.

## Failure semantics

- missing/mismatched schema artifact: release bind denied;
- exact schema replay: no duplicate revision or binding;
- initial snapshot conflict: denied without overwrite;
- accepted launch without readiness: module unavailable, provider routes не
  считаются ready;
- unconfigured integration: account setup доступен, provider execution
  disabled;
- missing secure provisioning host: non-secret form остаётся доступной, secret
  commit отклоняется до появления approved host;
- отсутствующий credential: sanitized `unconfigured`, без fake account;
- restart/revoke/stale generation: current runtime fences invalidate access.

## Phase gates

### `bundled_managed_settings_bootstrap_v1`

1. signed descriptor/schema bytes are the only contract source;
2. exact-idempotent schema admission and collision negative;
3. atomic initial snapshot materialization;
4. no credential/session/business data in Settings;
5. crash-safe retry between schema, snapshot and executable binding;
6. browser catalog exposes only admitted sanitized settings;
7. required field without signed default stays blocked at effective revision
   zero.

### `development_managed_runtime_ensemble_v1`

1. all exact Communications, Attachment Security, Mail, Telegram, WhatsApp and
   Zulip registrations are approved and release-bound;
2. every Storage owner has its own binding and runtime generation;
3. Domain, Engine and Integration use distinct start paths;
4. WhatsApp alone receives its approved host bridge;
5. unconfigured provider runtimes не запускаются до first Settings apply;
6. `make dev` keeps Kernel, platform, module runtimes, Gateway and frontend
   under one bounded cleanup lifecycle;
7. browser evidence distinguishes admission, launch and account readiness.

### `provider_account_setup_v1`

1. Mail, Telegram, WhatsApp и Zulip имеют отдельные provider-owned setup
   surfaces на Settings page;
2. setup доступен при approved registration даже с `effective_revision = 0`;
3. non-secret values идут только через Owner Module Settings;
4. secrets идут только через approved write-only Vault provisioning capability;
5. provider binding/OAuth/authorization использует generated integration
   contract;
6. successful setup приводит к повторному bootstrap и раскрывает только
   capabilities реально ready runtime;
7. один provider setup не импортирует другой provider или Communications.

## Последствия

- Settings sections больше не зависят от случайного external runtime
  self-registration.
- Первый provider runtime не стартует с выдуманной учётной записью или пустым
  required Settings snapshot.
- Account forms доступны до provider runtime readiness и используют реальные
  typed Settings, Vault и integration contracts.
- `make dev` остаётся assembly command, а не новым business owner.
