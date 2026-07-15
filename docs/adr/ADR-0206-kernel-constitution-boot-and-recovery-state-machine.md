# ADR-0206: Конституция Kernel и автомат запуска и восстановления

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Не реализовано

Связанные решения:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0209: Kernel Event Hub и контроль подписок](ADR-0209-kernel-event-hub-and-subscription-control-plane.md);
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md).

## Контекст

Kernel должен переживать отказ PostgreSQL, PgBouncer, NATS, vault и отдельных
module runtimes настолько, чтобы сохранить безопасный control/recovery surface.
Одновременно он не должен постепенно превратиться в god service, который
содержит бизнес-логику, междоменную координацию, provider mapping или общий
доступ к данным всех модулей.

Для clean-room реализации нужна явная конституция Kernel: исчерпывающий список
его обязанностей, запрещённые причины изменения, состояния процесса и правила
доступности операций при частичном отказе. Без этого понятия `core`, `runtime`
и `recovery mode` останутся соглашениями, которые невозможно проверить.

## Решение

### Kernel является техническим control plane

Kernel владеет только следующими обязанностями:

1. boot/recovery state machine всего локального экземпляра Hermes;
2. supervisor managed infrastructure и independently restartable module
   runtimes;
3. module registry, проверка manifest, protocol и distribution inventory;
4. построение и проверка startup dependency graph;
5. корневая runtime identity/capability authority и выдача scoped runtime
   capabilities через отдельные platform boundaries;
6. Core Gateway, client session bootstrap и transport-level authorization;
7. маршрутизация public contracts без интерпретации business payload;
8. sanitized health, diagnostics, lifecycle audit и recovery surface;
9. техническая конфигурация процесса, listeners, resource budgets и managed
   service ownership;
10. Event Hub: catalog event contracts, проверка publishers/subscribers,
    reconciliation NATS streams/consumers/permissions и delivery health без
    чтения business payload;
11. Telemetry Hub control surface: telemetry identity, schema, redaction,
    quotas, health и authorized diagnostics при отдельном supervised Collector
    process.

Это закрытый список. Новая ответственность Kernel требует изменения этого ADR
или нового superseding ADR.

### Kernel не владеет предметной логикой

В Kernel запрещены:

- domain entities, domain policies и domain state;
- provider protocol, provider mapping и neutral evidence mapping;
- cross-domain orchestration, workflows и business read composition;
- business schedulers и фоновые product jobs;
- AI, search, ranking, memory и context algorithms;
- module-owned SQL, tables, repositories и generic SQL proxy;
- NATS broker/data plane, Telemetry Collector, vault, blob или storage
  implementation как библиотека внутри Kernel;
- provider accounts, prompts, user automation rules и другие module settings;
- private message/document/media content в health, diagnostics или lifecycle
  audit.

Vault, blobs, event data plane, Telemetry Collector и storage доступны только
через отдельные platform capability boundaries. Их process topology задаётся
distribution manifest и общим правилом ADR-0200 для independently restartable
runtime. Kernel управляет их lifecycle и capability routing, Event Hub
контролирует event topology, а Telemetry Hub — telemetry policy; Kernel не
становится владельцем business payload или implementation managed services.

### Автомат состояний Kernel

Kernel использует один явный автомат:

```text
cold_start
    ↓
bootstrap
    ↓
recovery_only
    ↓
infrastructure_starting
    ↓
modules_starting
    ↓
ready

modules_starting / ready
    ↔ degraded

recovery_only / infrastructure_starting / modules_starting / ready / degraded
    → quiescing → draining → stopped

любое состояние
    → fatal
```

- `cold_start` — процесс создан, доверенная runtime identity ещё не
  установлена.
- `bootstrap` — проверяются single-instance lock, private directories,
  executable/distribution inventory, локальный control endpoint и минимальная
  техническая конфигурация.
- `recovery_only` — supervisor, локальный Gateway recovery surface и
  diagnostics доступны без PostgreSQL, PgBouncer, NATS, vault и модулей.
  Event Hub показывает declared/last-observed topology, а Telemetry Hub
  использует Collector либо bounded emergency log.
- `infrastructure_starting` — запускаются и проверяются declared managed
  platform services.
- `modules_starting` — manifest graph проверен, runtime запускаются в порядке
  обязательных capabilities.
- `ready` — все обязательные capabilities distribution готовы.
- `degraded` — Kernel и часть capabilities работают, но один или несколько
  необязательных runtime либо scoped capabilities недоступны.
- `quiescing` — новые mutations больше не принимаются.
- `draining` — in-flight work и durable checkpoints завершаются в bounded
  deadlines.
- `stopped` — managed children остановлены в установленном порядке.
- `fatal` — Kernel не может безопасно поддерживать даже доверенный recovery
  control plane.

`fatal` не используется для обычного отказа domain, workflow, integration,
PostgreSQL, PgBouncer, NATS или vault. Такие отказы переводят затронутые
capabilities в `blocked`, Kernel — в `degraded` или `recovery_only` и сохраняют
возможность диагностики и явного восстановления.

### Доступность операций по состояниям

| Состояние | Разрешено | Запрещено |
|---|---|---|
| `bootstrap` | только process-local liveness | client business и recovery operations |
| `recovery_only` | sanitized health/diagnostics, status, authorized retry/start/stop managed service, vault unlock/recovery entrypoints, authorized backup validation/restore entrypoints, shutdown | business queries, commands и mutations |
| `infrastructure_starting` | recovery operations и startup progress | business operations |
| `modules_starting` | recovery operations; запросы только к уже ready runtime, если их contract это разрешает | operation с любой unavailable required capability |
| `ready` | обычные authorized operations | всё, что запрещено contract/capability policy |
| `degraded` | операции healthy runtime с полным набором требуемых capabilities | operation, зависящая от blocked/unavailable capability |
| `quiescing` / `draining` | health, progress, bounded reads, завершение уже принятых работ | новые mutations и новые background claims |
| `stopped` / `fatal` | внешний watchdog может получить только process status/exit evidence | application operations |

Gateway проверяет состояние и required capability set каждого метода до
маршрутизации. Ошибка одного модуля не блокирует несвязанный healthy module.
Kernel не подменяет недоступный runtime другим implementation, transport или
topology.

Recovery surface по умолчанию доступен только через доверенный local channel.
Remote recovery listener не включается автоматически. Операции, способные
изменить data или credentials, требуют отдельной owner authorization policy и
никогда не запускаются из одного health probe.

### Dependency graph и порядок запуска

Каждый module manifest объявляет:

- предоставляемые capabilities;
- обязательные capabilities;
- необязательные capabilities;
- readiness condition;
- lifecycle protocol version;
- публикуемые и потребляемые event contracts;
- required/optional subscriptions и их resource budgets;
- telemetry signal capabilities и quotas.

Kernel строит граф до запуска business runtime. Цикл startup dependencies,
неизвестная обязательная capability, неоднозначный provider capability или
несовместимая protocol version блокируют затронутые runtime до provider call
или domain mutation.

Runtime запускается топологически; независимые ветви могут стартовать
параллельно. Event subscription сама по себе не является startup dependency и
не создаёт скрытый цикл. Поведение при недоступности:

- обязательная capability модуля недоступна — модуль остаётся `blocked`;
- необязательная capability недоступна — модуль может стать `degraded` и обязан
  явно отключить связанную функцию;
- обязательная platform capability всей distribution недоступна — Kernel
  остаётся `recovery_only`;
- необязательный module runtime недоступен — Kernel может обслуживать остальные
  healthy capabilities в `degraded`.

После восстановления capability зависимые runtime возобновляются только через
явный lifecycle transition и повторную readiness-проверку. Silent fallback и
обход dependency graph запрещены.

### Граница конфигурации

Kernel владеет только технической bootstrap-конфигурацией:

- runtime/data directories и single-instance identity;
- distribution manifest и executable inventory;
- ownership mode и endpoints managed/external services;
- local/remote listener profiles;
- startup, health, restart, drain и shutdown budgets;
- process resource budgets, event topology limits, telemetry quotas,
  retention и diagnostics redaction policy.

Каждый модуль владеет собственной product/configuration contract и durable
settings. Kernel может маршрутизировать typed configuration command владельцу,
но не хранит и не интерпретирует его поля.

Secrets, credential leases и bootstrap material не передаются через argv,
environment, logs или diagnostics. Точная модель owner, device, module,
provider-account и agent identities определяется отдельным ADR, не расширяя
предметную роль Kernel.

### Критерии глобального состояния

- `ready` определяется distribution manifest и readiness обязательных
  capabilities, а не числом запущенных процессов.
- Module `ready` требует manifest/handshake, storage compatibility и
  подтверждённую Event Hub readiness обязательных subscriptions.
- Один unhealthy probe не меняет state без классификации и bounded policy.
- Kernel возвращается из `degraded` или `recovery_only` только после повторной
  проверки полного набора обязательных capabilities.
- Глобальный `fatal` допускается только при потере доверия к самому control
  plane: невозможность установить single-instance identity, проверить
  executable/distribution integrity, защитить local control endpoint либо
  продолжать supervisor loop безопасно.

### Решения, остающиеся отдельными

Настоящий ADR фиксирует ownership и state machine, но не выбирает:

- полную identity/capability и pairing model;
- формат подписанного distribution manifest, update и rollback protocol;
- backup/restore format, retention и recovery authorization;
- окончательную topology Android Kernel;
- конкретный формат конфигурационных файлов.

Эти решения требуют отдельных ADR. Их implementations не должны добавляться в
Kernel до фиксации соответствующего contract.

## Отклонённые варианты

### Kernel как приложение со всеми domain services

Отклонено: создаёт общий failure domain, скрытые зависимости и невозможность
независимо перезапускать модули.

### Kernel, который запускается только после PostgreSQL и NATS

Отклонено: при отказе infrastructure исчезает сам control/recovery surface.

### Глобальный shutdown при падении любого модуля

Отклонено: противоречит изоляции runtime и делает необязательную capability
точкой отказа всей системы.

### Best-effort запуск при цикле или неизвестной capability

Отклонено: порядок становится недетерминированным, а ошибка проявляется уже во
время business operation.

### Business read composition внутри Gateway

Отклонено: transport adapter начал бы знать domain semantics и стал бы новым
монолитом.

## Последствия

Положительные:

- Kernel имеет небольшой, проверяемый набор причин изменения;
- recovery доступен до запуска canonical storage и data plane;
- ошибка одного модуля не останавливает несвязанные capabilities;
- startup order и partial availability становятся детерминированными;
- Gateway не превращается в business orchestration layer.

Отрицательные:

- каждый public method должен объявлять required capabilities;
- manifests и distribution inventory становятся критическим contract;
- recovery surface требует отдельной security и UX проверки;
- process-level state-machine tests обязательны с первой реализации.

## Проверка решения

До признания реализации завершённой должны существовать executable tests:

- Kernel достигает `recovery_only` при остановленных PostgreSQL, PgBouncer,
  NATS, vault и всех module runtimes;
- unavailable mandatory platform capability не допускает `ready`;
- crash optional module переводит Kernel в `degraded`, но healthy module
  продолжает обслуживать разрешённые operations;
- crash mandatory capability блокирует только методы с этой dependency либо
  переводит Kernel в `recovery_only` согласно distribution manifest;
- required и optional capability behavior соответствует manifest;
- startup dependency cycle и unknown capability fail closed до запуска модуля;
- independent modules запускаются параллельно после готовности dependencies;
- Gateway отклоняет mutation в `quiescing` и `draining`;
- восстановление capability требует повторного handshake/readiness;
- `fatal` не используется как реакция на обычный module/infrastructure crash;
- Kernel не импортирует domain, provider, workflow, AI, search, storage, vault
  или blob implementations;
- recovery health, diagnostics и audit не содержат secrets или private content;
- Event Hub остаётся вне normal event data path и не читает payload;
- required subscription readiness подтверждается через Event Hub;
- Telemetry Collector работает без PostgreSQL/NATS, а его crash переводит
  Kernel в `degraded`, не останавливая modules;
- remote recovery listener не включается автоматически;
- новый Kernel owner или responsibility ломает executable constitution guard.
