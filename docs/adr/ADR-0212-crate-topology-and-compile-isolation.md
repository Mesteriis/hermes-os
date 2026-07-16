# ADR-0212: Топология Cargo packages и изоляция пересборки модулей

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Dependency guard и self-tests реализованы; production
packages ещё не созданы

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0211: Backend workspace и физическая структура исходного кода](ADR-0211-backend-workspace-and-source-layout.md);
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md).

Уточнено:

- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

Этот ADR фиксирует границы Cargo compilation graph. Он не разрешает создание
заблокированных ADR-0208 доменов и проекций и не заменяет ещё отсутствующие
owner contracts.

## Контекст

Cargo пересобирает изменившийся package и его reverse dependencies. Само
наличие workspace не заставляет Cargo пересобирать все packages, но один общий
composition package, общий `common`, общий provider API или зависимость module
от Kernel снова соединяют независимые владельцы в широкий compile graph.

Требование Hermes: изменение implementation любого integration не должно
пересобирать Communications, Kernel, другие integrations или business domains.
Telegram является лишь примером. Аналогично изменение любого domain
implementation не должно пересобирать соседние domains: изменение Contacts не
пересобирает Tasks или Calendar.
Широкая пересборка допустима только после осознанного изменения действительно
общего wire protocol или узкого ingress contract.

Предыдущий backend в `references/backend-legacy/` использован только как
evidence. В нём aggregate backend package зависел почти от всего workspace,
provider API знал Kernel, persistence adapters пересекали ownership, а
значительная часть Telegram runtime, SQL и Communications mapping оставалась в
монолите. Эти наблюдения объясняют запреты ниже, но legacy package graph не
является шаблоном новой системы.

## Решение

### Две независимые границы

Hermes использует обе границы одновременно:

1. Cargo package определяет ownership исходного кода и область пересборки.
2. OS process определяет failure isolation и independently restartable runtime.

Нельзя объединять несколько independently restartable modules в один runtime
package ради удобства composition. Нельзя также дробить один cohesive owner на
случайные packages только ради уменьшения файлов: package создаётся по причине
изменения, dependency direction и типу adapter.

### Foundation packages

Общие packages являются protocol-first и не содержат owner-specific business
types:

```text
backend/src/platform/runtime_protocol/    hermes-runtime-protocol
backend/src/platform/events/protocol/     hermes-events-protocol
backend/src/platform/telemetry/protocol/  hermes-telemetry-protocol
backend/src/platform/storage/protocol/    hermes-storage-protocol
backend/src/platform/storage/control/     hermes-storage-control
backend/src/platform/storage/runtime/     hermes-storage-runtime
backend/src/platform/storage/postgres/    hermes-storage-postgres
backend/src/platform/storage/pgbouncer/   hermes-storage-pgbouncer
backend/src/platform/storage/migrations/  hermes-storage-migrations
backend/src/platform/vault/protocol/      hermes-vault-protocol
backend/src/platform/vault/key_provider/  hermes-vault-key-provider
backend/src/platform/vault/runtime/       hermes-vault-runtime
backend/src/platform/vault/store_sqlcipher/ hermes-vault-store-sqlcipher
backend/src/platform/vault/keychain_macos/ hermes-vault-keychain-macos
backend/src/platform/blob/protocol/       hermes-blob-protocol
backend/src/api/gateway/contracts/        hermes-gateway-protocol
backend/src/kernel/                       hermes-kernel
backend/src/kernel/control_store/contract/ hermes-kernel-control-store
backend/src/kernel/control_store/sqlite/   hermes-kernel-control-store-sqlite
backend/src/services/telemetry/collector/ hermes-telemetry-collector
```

`hermes-runtime-protocol` владеет `ModuleDescriptorV1`, capability declarations,
lifecycle/control messages, health wire types и settings schema/snapshot wire
contracts ADR-0221/0222. Module runtime и Kernel зависят от этого protocol;
module никогда не зависит от `hermes-kernel`.

Этот high-fanout package не содержит owner payloads, Kernel implementation,
NATS/SQL/SQLite clients, Vault implementation, JSON negotiation или filesystem
operations. `DistributionManifestV1` остаётся release artifact ADR-0219, а не
вторым runtime protocol package.

`hermes-events-protocol` владеет только `DurableEnvelopeV1`, kind-specific
delivery metadata, dead-letter technical record и envelope versioning
ADR-0220. Его exact metadata — `platform:events:contract`; второму package
нельзя объявить ownership canonical envelope.

NATS, PostgreSQL outbox/inbox, Event Hub implementation и routing являются
adapters или Kernel responsibilities, а не частью protocol package.
`hermes-events-protocol` не зависит от NATS/SQL/SQLite clients, JSON transport,
Kernel/Gateway или owner-specific contracts. Owner payload остаётся в owner
contract package и попадает в envelope как catalog-bound opaque Protobuf bytes.

Дополнительный platform protocol создаётся только при доказанной общей
capability и минимум двух независимых consumers. Один omnibus package
`hermes-common` запрещён.

Storage topology является закрытой группой ADR-0224:

- `hermes-storage-protocol` — единственный public contract для Kernel и
  authorized modules;
- `hermes-storage-control` — SQL-free orchestration и internal adapter ports;
- `hermes-storage-runtime` — отдельный managed process composition с component
  `storage_control`;
- `hermes-storage-postgres` — единственный platform adapter с PostgreSQL
  client, cluster/role/grant/ledger implementation и direct admin path;
- `hermes-storage-pgbouncer` — pooler config/readiness/pause/drain/kill adapter;
- `hermes-storage-migrations` — `StorageBundleV1` parser, PostgreSQL AST
  admission и migration planning без database credential.

Kernel, Gateway и modules не зависят от пяти private storage packages. Owner
persistence packages сохраняют собственный SQL и производят immutable
`StorageBundleV1` artifact, но Storage Control не получает Cargo dependency на
эти owners. `hermes-storage-protocol` не зависит от SQL/SQLite/NATS clients,
`serde_json`, Vault implementation или owner modules. Изменение PostgreSQL,
PgBouncer либо migration adapter пересобирает только `hermes-storage-runtime`
и storage tests; изменение owner persistence не пересобирает Storage Control.

Vault topology является закрытой owner-local группой ADR-0223:

- `hermes-vault-protocol` — единственный public contract для Kernel, Gateway и
  modules;
- `hermes-vault-key-provider` — внутренний adapter port владельца Vault;
- `hermes-vault-runtime` — отдельный managed process composition;
- `hermes-vault-store-sqlcipher` — encrypted SQLite schema/migrations adapter;
- `hermes-vault-keychain-macos` — macOS platform-key adapter.

Kernel, Gateway и modules не зависят от четырёх implementation packages. Vault
runtime не зависит от PostgreSQL/NATS clients, provider SDK, integration/domain
packages или Kernel implementation. Изменение store/keychain adapter может
пересобрать только Vault runtime и его tests; изменение public Vault protocol
осознанно инвалидирует его authorized consumers.

Kernel Control Store следует обычному правилу ответственности: узкий
core-owned port отделён от SQLite persistence adapter, чтобы `rusqlite`, schema
и migrations не попадали в `hermes-kernel` logic и тем более в module compile
graphs. Оба packages принадлежат owner `kernel`; только SQLite package имеет
surface `persistence`. Architecture exception не создаётся.

### Обычный durable domain

Независимый durable domain имеет до четырёх owner-local packages:

```text
backend/src/domains/<owner>/contracts/      hermes-<owner>-contracts
backend/src/domains/<owner>/implementation/ hermes-<owner>-domain
backend/src/domains/<owner>/persistence/    hermes-<owner>-persistence
backend/src/domains/<owner>/runtime/        hermes-<owner>-runtime
```

- `contracts` — commands, queries, events, typed errors и wire types владельца;
- `domain` — business rules и application behavior без SQL и runtime bootstrap;
- `persistence` — schema migrations, row mapping и adapter реализации только
  для собственного владельца;
- `runtime` — process entrypoint и owner-local composition.

Surface создаётся только когда ответственность существует. Малый stateless
owner может не иметь persistence; pure library не получает фиктивный runtime.
Runtime зависит от packages своего владельца и platform protocols, но не от
runtime другого владельца.

### Communications

Communications имеет два разных публичных контракта и поэтому получает отдельный
ingress package:

```text
hermes-communications-ingress
hermes-communications-api
hermes-communications-domain
hermes-communications-persistence
hermes-communications-runtime
```

`hermes-communications-ingress` содержит только provider-neutral observation,
source provenance, attachment/blob references и acknowledgement semantics,
необходимые integrations для публикации внешнего сигнала.

`hermes-communications-api` содержит client-facing и owner-facing commands,
queries и events. Integration не зависит от него. Тем самым provider может
публиковать evidence, но не получает доступ к Communications behavior или
storage.

Только точное package name `hermes-communications-ingress` находится в
executable allowlist integration-to-domain contracts. Разрешение по owner
`communications` было бы слишком широким.

### Integration packages

Каждая integration владеет собственным operational API, provider protocol,
adapter, session/cursor persistence и process runtime. Общая форма одинакова
для всех providers:

```text
hermes-<provider>-api
hermes-<provider>-core
hermes-<provider>-<protocol-adapter>
hermes-<provider>-persistence
hermes-<provider>-runtime
```

Это pattern ownership, а не общий provider abstraction. Packages разных
providers не зависят друг от друга и не реализуют один универсальный
operational trait. Точные начальные package graphs:

```text
Mail:     api + core + imap + smtp + persistence + runtime
Telegram: api + core + tdlib + persistence + runtime
Zulip:    api + core + http + persistence + runtime
```

На примере Telegram физическая форма выглядит так:

```text
backend/src/integrations/telegram/api/         hermes-telegram-api
backend/src/integrations/telegram/core/        hermes-telegram-core
backend/src/integrations/telegram/tdlib/       hermes-telegram-tdlib
backend/src/integrations/telegram/persistence/ hermes-telegram-persistence
backend/src/integrations/telegram/runtime/     hermes-telegram-runtime
```

- `api` — provider operational contract для Core Gateway и clients;
- `core` — Telegram-specific state machine, mapping и neutral evidence output;
- `tdlib` — TDLib/TDJSON adapter без SQL и domain imports;
- `persistence` — только Telegram accounts, session metadata, cursors и
  operational state;
- `runtime` — единственная owner-local composition этих packages.

`hermes-<provider>-core` может зависеть от
`hermes-communications-ingress`. Обратной зависимости нет. Поэтому изменение
Mail, Telegram или Zulip implementation не инвалидирует Communications.

Protocol-specific split (`imap`/`smtp`, `tdlib`, `http`) создаётся только при
реальной независимой причине изменения. Заранее scaffold-ить пустые packages
запрещено. Любая будущая integration применяет то же правило автоматически и
не требует расширения общего provider facade.

WhatsApp остаётся hidden Tauri WebView по ADR-0204. Backend package для него не
создаётся до отдельного решения о versioned host-to-backend bridge; WebView
implementation не переносится в backend workspace.

### Kernel и Gateway

Kernel обнаруживает module runtime через open local registration и
`hermes-runtime-protocol` по ADR-0215/0221. Pending `ModuleDescriptorV1` не
выдаёт rights; после approval Kernel маршрутизирует только effective opaque
owner capabilities, но не линкует domain, workflow или integration contracts и
implementations.

Gateway transport также не становится статическим aggregate всех module
contracts. `ModuleDescriptorV1` содержит exact client contract references, а
generated client schemas остаются first-party bundle artifacts. Kernel/Gateway
generic code зависит только от platform и gateway protocols.

Это означает, что добавление любой integration или изменение её API не требует
пересборки Kernel. Packaging может обновить signed `DistributionManifestV1` и
exact descriptor/settings schema artifacts без Rust dependency Kernel →
конкретный provider.

### Запрещённые aggregation packages

Следующие package names запрещены, потому что их прежний смысл неизбежно
собирает разные owners в один reverse-dependency fan-out:

```text
hermes-hub-backend
hermes-api
hermes-worker-runtime
hermes-desktop-runtime
hermes-schema
hermes-common
hermes-provider-api
```

Это запрет ответственности, а не только имени. Нельзя вернуть ту же агрегацию
под новым названием. В частности:

- schema и migrations принадлежат owner persistence packages;
- process composition принадлежит owner runtime или Kernel;
- provider contracts принадлежат конкретной integration;
- test composition находится только в test-support packages.

### Разрешённый dependency graph

```text
platform protocol ← Kernel
platform protocol ← owner contracts / implementation / adapters / runtime

owner contracts ← owner implementation
owner implementation ← owner persistence adapter
owner contracts + implementation + persistence ← owner runtime

communications ingress ← integration core
integration api + core + provider adapter + persistence ← integration runtime
```

Стрелка означает «правый package зависит от левого».

Запрещено:

- module → Kernel implementation;
- Kernel/Gateway → owner-specific module package;
- integration → любой domain contract, кроме точного Communications ingress;
- cross-owner persistence → persistence;
- runtime → runtime даже внутри одного owner;
- cross-owner dependency на implementation для normal, build и dev edges;
- production → test-support иначе чем через dev dependency;
- compatibility facade или re-export aggregate.

### Ожидаемая область пересборки

| Изменение | Допустимые reverse dependencies |
|---|---|
| Любой provider adapter (`imap`, `tdlib`, `http`) | runtime того же provider и его tests |
| `hermes-<provider>-core` | adapters/persistence/runtime того же provider и его tests |
| `hermes-<provider>-persistence` | runtime того же provider и его tests |
| `hermes-communications-domain` | Communications persistence/runtime и Communications tests |
| `hermes-contacts-domain` | Contacts persistence/runtime и Contacts tests |
| `hermes-communications-ingress` | integrations, которые публикуют neutral evidence; это осознанный fan-out |
| global runtime/event protocol | его consumers; изменение требует contract review |
| Storage PostgreSQL/PgBouncer/migration adapter | `hermes-storage-runtime` и Storage tests |
| `hermes-storage-control` | `hermes-storage-runtime` и Storage tests |
| `hermes-storage-protocol` | authorized modules, Kernel и Storage runtime; contract review обязателен |
| Vault store/keychain adapter | `hermes-vault-runtime` и Vault tests |
| `hermes-vault-protocol` | Vault runtime и authorized protocol consumers; contract review обязателен |
| `hermes-kernel` implementation | только Kernel tests и packaging |

Package-local source change не должен инвалидировать client code, чужие
domains или integrations. Contract change может иметь больший fan-out; это
сигнал реального публичного изменения, а не архитектурная ошибка Cargo.

## Executable policy

`backend/architecture/policy.json` и Cargo guard проверяют:

- точный allowlist integration ingress packages;
- единственный `hermes-events-protocol` с metadata
  `platform:events:contract` и без transport/storage dependencies;
- запрещённые aggregate package names;
- module → Kernel и Kernel/Gateway → module edges;
- runtime aggregation;
- cross-owner persistence dependencies;
- незаявленные `hermes-storage-*`, SQL client вне owner persistence или
  `hermes-storage-postgres`, migration/admin dependency module runtime и
  Kernel dependency на private storage implementation;
- normal, build и dev dependency edges;
- положительные Mail, Telegram и Zulip graphs без Communications implementation
  dependency;
- положительные Communications и все currently enabled domain package graphs;
- запрет backend implementation/persistence/runtime для host-only WhatsApp.

Policy не содержит baseline или per-package exceptions. Новое исключение
требует изменения ADR и общего правила, а не добавления файла-должника.

## Последствия

### Положительные

- изменение provider implementation остаётся внутри provider compile graph;
- runtime и compile isolation выражены разными, проверяемыми механизмами;
- Kernel остаётся стабильным technical control plane;
- SQL, migrations и provider SDK не протекают через public contracts;
- Cargo diagnostics показывают конкретное нарушение ownership.

### Цена

- wire contracts требуют отдельного versioning и descriptor packaging;
- один owner может иметь несколько packages и явный composition runtime;
- изменение действительно общего protocol осознанно пересобирает consumers;
- process-level isolation всё равно требует lifecycle и integration tests,
  одного Cargo guard недостаточно.

## Отклонённые варианты

### Один backend package

Отклонено: связывает все причины изменения, тесты, SQL и provider SDK в один
compile graph и не выражает ownership.

### Package на каждый файл или use case

Отклонено: увеличивает coordination cost без самостоятельной причины изменения
или dependency boundary.

### Общий `provider-api`

Отклонено: Mail, Telegram, WhatsApp и Zulip имеют разные operational contracts.
Общим является только runtime/evidence protocol, а не provider behavior.

### Kernel как compile-time composition root всех modules

Отклонено: любое module изменение пересобирает Kernel и делает независимое
обновление/перезапуск фиктивным.

## Критерии приёмки первого implementation slice

ADR-0225 уже закрыл capability/ownership inventory первого slice: разрешён
ровно exact six-package `kernel_recovery_only_v1` с пустым owner inventory.
Эти packages ещё не созданы; любой следующий package остаётся запрещён до
открытия соответствующего phase gate.

Первый slice считается соответствующим этому ADR, когда:

- каждый package имеет ровно одну Hermes role, owner и surface;
- `cargo metadata` проходит executable guard;
- `cargo tree` каждого integration не содержит Kernel или Communications
  implementation;
- reverse dependencies implementation каждого integration не содержат
  Communications, Kernel или другие integrations;
- Kernel/Gateway tree не содержит owner-specific packages;
- targeted package check не собирает чужой owner;
- architecture self-tests и `make -C backend validate` проходят.

Текущий факт реализации ограничен policy/guard и self-tests. Ни один
production package этим ADR не объявляется существующим.
