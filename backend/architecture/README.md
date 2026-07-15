# Executable architecture policy

Файл [`policy.json`](policy.json) является executable companion для active ADR
clean-room системы. ADR объясняют решение, а policy и linters запрещают
нарушающие его packages, paths, SQL ownership и dependency edges.

[ADR-0211](../../docs/adr/ADR-0211-backend-workspace-and-source-layout.md)
принимает `backend/` как единственную физическую границу. Policy, linters,
architecture tests и virtual Cargo workspace находятся внутри этой границы;
root-level compatibility paths отсутствуют и запрещены layout guard.

## Проверки

```sh
make -C backend architecture-policy-check
make -C backend cargo-boundaries-check
make -C backend test-architecture
make -C backend architecture-check
```

- `architecture-policy-check` проверяет согласованность allowlist/blocklist,
  обязательные Kernel components, production paths и SQL ownership.
- `cargo-boundaries-check` использует `cargo metadata --no-deps`, поэтому не
  собирает workspace и не требует загрузки dependencies. Он также проверяет,
  что все production `Cargo.toml` зарегистрированы в workspace, а SQL-файлы
  принадлежат persistence package своего владельца.
- `test-architecture` запускает positive и negative self-tests из
  `tests/architecture/`.
- `architecture-check` запускает оба production linter.

Virtual clean-room Cargo workspace существует, даже когда в нём ещё нет
production packages. Каждый будущий workspace package обязан иметь metadata,
жить в production или test-only root и проходить полный dependency check.

## Cargo metadata contract

Каждый workspace package объявляет ровно одну роль, одного владельца и один
тип поверхности:

```toml
[package.metadata.hermes]
role = "domain"
owner = "contacts"
surface = "contract"
```

Допустимые роли и surfaces определены только в `policy.json`. Неизвестная или
отсутствующая роль немедленно ломает guard.

Kernel дополнительно объявляет закрытый список компонентов:

```toml
[package.metadata.hermes]
role = "core"
owner = "kernel"
surface = "runtime"
components = [
  "supervisor",
  "module_registry",
  "capability_router",
  "core_gateway",
  "event_hub",
  "telemetry_control",
]
```

Telemetry Collector является отдельным platform runtime:

```toml
[package.metadata.hermes]
role = "platform"
owner = "telemetry"
surface = "runtime"
components = ["telemetry_collector"]
```

## Dependency rules

- domain не зависит от другого domain или integration;
- integration зависит из business domains только от нейтрального
  Communications contract; остальные domain contracts запрещены;
- workflow и API используют чужие packages только через `contract`;
- contract не зависит от runtime, implementation или persistence своего
  владельца;
- implementation не зависит от persistence; runtime своего владельца является
  единственной surface, которая собирает implementation и persistence вместе;
- core использует только platform/API contracts и не линкует module или
  Telemetry Collector implementations;
- cross-owner dependency на implementation запрещена для normal, build и dev
  edges;
- test support разрешён production package только как dev dependency;
- PostgreSQL client crates и SQL-файлы разрешены только `persistence` surface;
- SQL identifiers используют owner prefix; cross-owner reads, writes и foreign
  keys запрещены;
- Telemetry Collector не зависит от NATS или PostgreSQL clients;
- Event Hub и telemetry control могут быть компонентами только Kernel;
- blocked domains и projections запрещены в package metadata, package names,
  production paths и SQL ownership declarations.

## Область source scan

Source scan применяется только к `backend/src` через relative root `src` из
`policy.json`. `references/`, documentation, tests и generated/build
directories не входят в production scan; symlink внутри source запрещён, а не
игнорируется. Текущие legacy frontend и protobuf contracts не объявляются как
clean-room module roots; для их замены потребуется отдельный synchronized
client/contracts cutover и собственный guard.

Baseline-файлы и per-file exceptions не поддерживаются. Изменение allowlist,
blocked projections или Kernel ownership требует принятого ADR и одновременного
изменения policy и negative tests.

Guard анализирует standalone `.sql` migrations и ownership declarations.
Строковые SQL queries внутри Rust дополнительно ограничены тем, что SQL client
dependency разрешена только persistence package; окончательное фактическое
разделение таблиц обеспечивают owner roles/grants из ADR-0202 и их integration
tests. Формат module/distribution manifest пока не линтится: ADR-0206 оставляет
его отдельным нерешённым контрактом.
