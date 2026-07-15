# ADR-0203: Управление локальной инфраструктурой и восстановление

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Не реализовано

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md).

Уточняется:

- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0209: Kernel Event Hub и контроль подписок](ADR-0209-kernel-event-hub-and-subscription-control-plane.md);
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md).

## Контекст

Hermes должен самостоятельно управлять локальными PostgreSQL, PgBouncer, NATS
JetStream, Telemetry Collector и module runtimes. При этом отдельный
обязательный Host Supervisor рядом с Kernel увеличил бы количество процессов и
создал ещё одну lifecycle-границу до появления доказанной необходимости.

Kernel не может перезапустить собственный процесс. Для этого всегда нужен
внешний watchdog, но в desktop topology эту роль уже может выполнять Tauri, а в
headless topology — операционная система.

Автоматический restart инфраструктуры опасен, если он запускается по любому
красному health check. Connection exhaustion, recovery, disk full, повреждение
данных и configuration mismatch требуют разных действий. Restart не должен
скрывать причину, создавать crash loop или заменять повреждённое хранилище новым
пустым cluster.

## Решение

### Иерархия supervision

Supervisor является независимой подсистемой Hermes Kernel, а не отдельным
обязательным Hermes-процессом:

```text
Tauri или OS watchdog
        ↓
Hermes Kernel process
  ├── supervisor subsystem
  ├── capability router и Event Hub
  ├── Telemetry Hub control surface
  ├── Telemetry Collector
  ├── PostgreSQL
  ├── PgBouncer
  ├── NATS JetStream
  └── module runtimes
```

Ответственность распределяется так:

| Уровень | Ответственность |
|---|---|
| Tauri / OS watchdog | запустить и bounded-перезапустить Kernel process |
| Kernel supervisor | управлять Telemetry Collector, PostgreSQL, PgBouncer, NATS и module runtimes |
| Module runtime | управлять только собственными workers и provider resources |

Tauri не принимает storage, routing или business решения. В headless режиме та
же граница реализуется через `launchd`, `systemd` или другой OS supervisor.

### Независимость supervisor subsystem

Supervisor event loop не зависит от доступности Telemetry Collector,
PostgreSQL, PgBouncer, NATS, module runtime или внешнего API. Он обязан
продолжать работать в recovery mode, когда любой из этих компонентов
недоступен.

Минимальное supervisor state:

- desired service state;
- child process handle и проверенная identity;
- runtime instance ID;
- restart counters и backoff deadline;
- последний sanitized health result;
- control endpoint metadata;
- startup/shutdown phase.

Это состояние находится в памяти. Если для crash reconciliation нужен локальный
runtime-state file, он:

- хранится в private runtime directory;
- записывается атомарно;
- не содержит credentials или private content;
- является rebuildable metadata, а не canonical truth;
- не требуется для восстановления business data.

### Режим владения сервисом

Каждый infrastructure service имеет явный ownership mode:

- `managed_child` — Kernel запустил приватный child process и имеет право его
  останавливать и перезапускать;
- `external` — сервис запущен пользователем или операционной системой; Kernel
  только проверяет health и никогда не посылает ему stop/restart signal.

Ownership mode задаётся конфигурацией. Kernel не определяет его эвристически по
порту, PID или имени процесса.

Desktop default:

- managed Telemetry Collector с private local store;
- приватный managed PostgreSQL cluster в Hermes data directory;
- managed PgBouncer;
- managed NATS с приватным JetStream directory;
- managed module runtimes.

Системный, Homebrew, Docker или иной внешний PostgreSQL не становится managed
автоматически.

### Идентификация managed child

PID сам по себе недостаточен из-за повторного использования PID. Перед adopt,
signal или kill Kernel проверяет одновременно:

- runtime instance ID и owner nonce;
- ожидаемый executable identity;
- process start identity;
- private runtime/data directory;
- control handshake, если он поддерживается сервисом.

Kernel не посылает signal процессу, identity которого нельзя однозначно
подтвердить. После crash Kernel новый instance либо безопасно принимает
подтверждённый managed child под supervision, либо выполняет controlled stop и
restart. Произвольный процесс, найденный только по PID или порту, не adoption и
не завершается.

### Startup order

Стандартный managed startup:

1. получить single-instance lock на Hermes runtime/data directory;
2. проверить permissions, ownership, доступное место и version compatibility;
3. проверить отсутствие неподтверждённых orphan children;
4. запустить Telemetry Collector; при отказе включить bounded emergency log и
   продолжить startup в `degraded`;
5. запустить PostgreSQL и дождаться завершения recovery/readiness;
6. проверить cluster identity, schema compatibility, extensions, roles и
   grants;
7. выполнить разрешённый migration/bootstrap phase;
8. запустить PgBouncer и проверить normal pooled connection path;
9. запустить NATS и проверить JetStream storage и stream definitions;
10. выполнить Event Hub reconciliation catalog, streams, consumers и
    permissions;
11. запустить capability routing и outbox relay;
12. запустить module runtimes в порядке declared required capabilities;
13. объявить Kernel ready только после прохождения обязательных checks.

Независимые модули могут запускаться параллельно после готовности их
dependencies. Ошибка необязательного модуля не блокирует readiness остальных
capabilities.

### Ordered shutdown

Стандартный shutdown выполняется в обратном направлении:

1. API и routers прекращают принимать новые mutations;
2. modules получают `Quiesce` и прекращают claim новых работ;
3. in-flight operations завершаются в пределах deadline;
4. cursors, inbox и outbox state фиксируются;
5. module runtimes останавливаются;
6. outbox relay и data-plane consumers выполняют финальный checkpoint;
7. NATS останавливается без удаления JetStream state;
8. PgBouncer прекращает выдавать новые connections и останавливается;
9. PostgreSQL выполняет штатный checkpoint/shutdown;
10. Telemetry Collector сохраняет финальные lifecycle/shutdown records,
    завершает bounded flush и останавливается;
11. supervisor освобождает runtime lock и завершает Kernel.

По истечении deadline следующий уровень может выполнить forced termination.
Каждый forced step попадает в sanitized shutdown report.

### Restart policy по сервисам

| Сервис | Автоматический restart | Ограничение |
|---|---|---|
| Module runtime | да | bounded backoff и crash budget |
| Telemetry Collector | да | bounded restart; fallback только в private emergency log |
| PgBouncer | да | не считать pool exhaustion причиной restart PostgreSQL |
| NATS | да | сохранить JetStream state; PostgreSQL outbox удерживает backlog |
| PostgreSQL | ограниченно | только process exit или классифицированный recoverable failure |
| Kernel | внешний watchdog | bounded restart без reset managed data |

Ни один сервис не получает бесконечный restart loop. После исчерпания budget он
переходит в blocker state до явного действия или изменения внешнего состояния.

### Классификация отказов PostgreSQL

| Состояние | Действие |
|---|---|
| Process exit | bounded restart, затем readiness и recovery checks |
| Временная недоступность | продолжать probes до deadline, не restart сразу |
| Startup/WAL recovery | ждать завершения в пределах recovery policy |
| Connection exhaustion | backpressure и pool diagnostics; не restart database |
| PgBouncer failure | restart PgBouncer; не restart database |
| Disk full | fail closed и blocker; restart запрещён |
| Data/WAL corruption | fail closed и recovery workflow; restart loop запрещён |
| Configuration/auth mismatch | blocker; автоматический restart запрещён |
| Binary/data version mismatch | blocker; автоматический upgrade или reset запрещён |

Красный readiness probe сам по себе не является основанием для restart.

### Координированный restart PostgreSQL

При planned restart:

1. supervisor переводит storage capability в `quiescing`;
2. Core прекращает новые mutations;
3. DB-dependent modules завершают транзакции и checkpoint-ят работу;
4. PgBouncer прекращает выдавать новые server connections;
5. PostgreSQL штатно останавливается и запускается;
6. supervisor дожидается recovery и проверяет cluster identity;
7. проверяются schema version, extensions, roles и grants;
8. проверяется путь через PgBouncer;
9. modules возобновляются;
10. NATS/outbox replay обрабатывает накопленный backlog.

При аварийном падении graceful steps до restart недоступны. Незавершённые
database transactions восстанавливаются PostgreSQL; Kernel не пытается
имитировать их commit или повторять non-idempotent external operations.

### Recovery mode

Kernel остаётся жив и предоставляет sanitized local health/control surface,
даже если storage или NATS недоступны. В recovery mode разрешены только:

- status и diagnostics без private content;
- controlled restart/stop для managed services;
- backup/recovery operations с отдельной authorization policy;
- shutdown приложения.

Business commands и writes fail closed до восстановления обязательных
capabilities.

ADR-0206 расширяет это правило до явного `recovery_only` state, который обязан
достигаться без PostgreSQL, PgBouncer, NATS, vault и module runtimes, и задаёт
доступность Gateway operations во всех состояниях Kernel.

### Неприкосновенность данных

Никакой автоматический restart не имеет права:

- удалять или переименовывать PostgreSQL data directory;
- выполнять `initdb` поверх отсутствующего или повреждённого cluster;
- создавать новый пустой cluster как fallback;
- очищать JetStream directory, streams или consumer state;
- применять destructive migration;
- сбрасывать roles/grants для обхода ошибки;
- удалять vault credentials или provider sessions;
- переключать managed service на другой endpoint, storage driver или topology;
- восстанавливать backup без явного авторизованного действия.

Restart меняет только состояние процесса, но не identity и содержимое storage.

### Внешний watchdog

В desktop topology Tauri запускает Kernel как sidecar, следит за его process
exit и применяет bounded restart policy. Tauri не интерпретирует внутренние
module или storage errors и не перезапускает отдельные children Kernel.

Внешний watchdog не очищает runtime/data directories. После restart Kernel сам
выполняет managed-child reconciliation и полную readiness sequence.

Отдельный постоянный `hermes-host` process не вводится. Он может быть рассмотрен
новым ADR только при доказанной необходимости, например:

- headless runtime должен переживать закрытие Tauri;
- несколько клиентов подключаются к одному постоянно работающему Hermes;
- supervisor должен переживать полный crash Kernel process;
- требуется независимое обновление Kernel без остановки managed services.

## Отклонённые варианты

### Обязательный отдельный Host Supervisor process

Отклонён для первой topology как дополнительная lifecycle и packaging boundary
без доказанной необходимости. Логическая supervision boundary сохраняется
внутри Kernel.

### Supervisor, зависящий от PostgreSQL или NATS

Отклонён, потому что при отказе зависимости он потеряет возможность управлять
её восстановлением.

### Restart по любому failed health check

Отклонён: маскирует resource/configuration/corruption failures и создаёт crash
loops.

### Автоматический reset или fallback storage

Отклонён из-за риска необратимой потери canonical data и provider state.

## Последствия

Положительные:

- Kernel самостоятельно управляет всей приватной local infrastructure;
- отдельный обязательный Hermes supervisor process не нужен;
- отказ PostgreSQL или NATS не выключает control/recovery loop;
- отказ Telemetry Collector не останавливает Kernel или modules;
- managed и external services не смешиваются;
- restart policy не превращается в скрытый data reset;
- Tauri и headless watchdog имеют одну узкую ответственность.

Отрицательные:

- Kernel supervisor subsystem должен быть независим от остальных core tasks;
- требуется безопасная child identity и orphan reconciliation;
- startup/shutdown/recovery становятся отдельным test surface;
- полный crash Kernel всё равно требует внешнего watchdog;
- одна physical database остаётся общей infrastructure failure domain.

## Проверка решения

- kill каждого module runtime не завершает Kernel и соседние modules;
- kill Telemetry Collector включает bounded emergency log и scoped degraded
  state без остановки modules;
- kill PgBouncer приводит к его bounded restart без restart PostgreSQL;
- kill NATS сохраняет PostgreSQL outbox и JetStream state после restart;
- kill PostgreSQL приводит к recovery checks и возобновлению через PgBouncer;
- connection exhaustion не перезапускает PostgreSQL;
- disk full, corruption и version mismatch переходят в blocker без restart
  loop;
- planned PostgreSQL restart выполняет quiesce/checkpoint/resume sequence;
- external service никогда не получает signal от Kernel;
- PID reuse и неподтверждённый orphan не приводят к завершению чужого процесса;
- Kernel restart reconciles managed children без удаления state;
- ordered shutdown останавливает PostgreSQL после NATS/PgBouncer, а Telemetry
  Collector — последним managed service;
- Tauri restart policy ограничена и не управляет children Kernel напрямую;
- recovery health surface не содержит secrets или private content;
- ни один restart test не удаляет PostgreSQL, JetStream, vault или provider
  session state.
