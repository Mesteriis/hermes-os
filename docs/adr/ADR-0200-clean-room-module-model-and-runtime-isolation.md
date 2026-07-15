# ADR-0200: Модульная модель и изоляция runtime

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Не реализовано

Связанные решения:

- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md).

Исторический контекст находится в
[ADR-0184](../../references/backend-legacy/docs/archive/adr/ADR-0184-backend-clean-room-restart.md),
но legacy ADR не являются действующей policy. Настоящий документ самостоятельно
фиксирует активное решение.

## Контекст

Hermes должен состоять из изолированных доменных, интеграционных и workflow-
модулей. Ошибка одного модуля не должна завершать ядро или останавливать другие
модули. Ядро должно уметь явно запускать, останавливать, дренировать и
перезапускать каждый runtime.

Cargo crate задаёт границу исходного кода и зависимостей, но не обеспечивает
изоляцию памяти, panic, deadlock, утечки ресурсов или аварийного завершения.
Поэтому source-модуль и запущенный module runtime являются разными понятиями.

## Решение

### Термины

- **Модуль** — владелец одной предметной или технической ответственности с
  собственным публичным контрактом.
- **Contract package** — типы, команды, запросы, события и ошибки модуля без его
  реализации, SQL, provider SDK и runtime bootstrap.
- **Module runtime** — отдельный OS-процесс, исполняющий один модуль и
  управляемый supervisor ядра.
- **Ядро** — минимальный runtime host: supervisor subsystem, registry,
  capability router,
  доступ к platform services и внешний API gateway. Ядро не содержит
  предметной логики модулей.
- **Platform service** — техническая capability, например storage, events,
  vault, blobs, clock или scheduler.
- **Workflow module** — process manager, который координирует несколько
  модулей только через их публичные контракты.

Contract packages и pure-библиотеки сами по себе не являются процессами.
Процессом является только independently managed runtime.

### Виды модулей

Hermes использует следующие роли:

- `domain` — владеет bounded context и его durable business truth;
- `integration` — встроенный плагин, владеющий внешним протоколом, auth/session
  runtime, cursor, provider-specific operational contract/projection и
  преобразованием observations в neutral evidence, но не business truth;
- `workflow` — координирует несколько публичных контрактов;
- `engine` — выполняет pure или производные вычисления без мутации business
  truth;
- `platform` — предоставляет техническую capability;
- `api` — переводит внешний контракт в query/command приложения;
- `module_runtime` — исполняет один модуль в отдельном процессе;
- `core_runtime` — является единственным composition root и supervisor.

Один package имеет одну архитектурную роль. Совмещение ролей должно быть
запрещено executable guard, а не соглашением в README.

### Ownership и зависимости

- Domain implementation импортирует только собственный contract, разрешённые
  platform contracts и pure-механизмы.
- Domain implementation не импортирует другой domain, integration, workflow
  или core implementation.
- Integration не импортирует business domain implementation и не создаёт
  durable domain entities. Он может зависеть от собственного operational
  contract и neutral evidence/public platform contracts.
- Domain не импортирует provider operational contracts, provider SDK или
  integration implementation. Provider identity остаётся provenance, а не
  business discriminator.
- Workflow может зависеть от public contracts нескольких модулей, но не от их
  implementation или storage adapter.
- Ядро видит manifests и transport contracts, но не интерпретирует business
  payload.
- Внешний API adapter может зависеть от public contracts, но не от module
  implementations.
- Прямой module-to-module import и прямое runtime-соединение между модулями
  запрещены.

Cross-domain поведение принадлежит workflow:

```text
source module event
        ↓
workflow module
        ↓
target module command
```

Ядро маршрутизирует сообщения, но не принимает предметные решения.

### Иерархия supervision

Supervisor является независимой подсистемой Kernel, а не отдельным
обязательным Hermes-процессом. Он управляет managed PostgreSQL, PgBouncer, NATS
и module runtimes и продолжает работать при их недоступности.

Сам Kernel перезапускается внешним watchdog: Tauri в desktop topology или OS
supervisor в headless topology. Внешний watchdog не управляет отдельными
children Kernel и не принимает business/storage решения. Полная lifecycle и
recovery policy определена в ADR-0203.

### Граница процесса

Каждый runtime, который требуется независимо останавливать или перезапускать,
запускается отдельным native-процессом. Это относится к domain runtimes,
workflow runtimes и integrations с realtime/background lifecycle.

Внутрипроцессное подключение module implementation к core запрещено как
production topology. Оно допускается только в pure unit tests через тот же
семантический contract и не считается доказательством runtime-изоляции.

Контейнер на модуль не является default topology для desktop-приложения.
WASM может быть рассмотрен отдельным ADR для сторонних недоверенных plugins, но
не используется как первая реализация собственных модулей.

Первая distribution model включает только встроенные allowlisted modules и
frontend surfaces. Marketplace, plugin store, runtime download и remote
frontend code не входят в архитектуру; их возможное появление потребует нового
ADR и отдельной trust/sandbox model.

### Module manifest

Каждый runtime предоставляет versioned manifest минимум со следующими полями:

- стабильный `module_id`;
- версия runtime protocol;
- версия manifest schema;
- архитектурная роль;
- предоставляемые capabilities;
- требуемые capabilities;
- публикуемые и потребляемые contracts;
- provider-specific operational contracts, neutral evidence contracts и
  bundled frontend `surface_id`, когда это integration-плагин;
- поддерживаемые lifecycle operations;
- storage owner identity;
- NATS publish/subscribe permissions;
- resource budget descriptor;
- версия runtime implementation.

Неизвестная protocol version, незаявленная capability или конфликтующий
`module_id` приводят к отказу запуска до выполнения business operation.

### Lifecycle

Supervisor поддерживает явные состояния:

```text
discovered
  → starting
  → ready
  → quiescing
  → draining
  → stopped

starting / ready / draining
  → degraded
  → failed
  → crash_loop_blocked
```

Runtime не считается `ready` до успешного handshake, проверки manifest,
storage compatibility и готовности обязательных subscriptions.

Стандартная остановка:

1. прекратить принимать новые commands;
2. завершить или checkpoint-нуть in-flight work;
3. подтвердить durable cursors и outbox state;
4. остановить provider/background resources;
5. завершить процесс;
6. по истечении deadline supervisor принудительно завершает процесс и
   сохраняет sanitized shutdown report.

Точные deadlines являются runtime configuration и проверяются тестами; модуль
не может самостоятельно продлевать их бесконечно.

### Restart policy

- Supervisor перезапускает только тот же module runtime.
- Используются bounded exponential backoff, jitter и crash budget.
- После исчерпания budget runtime переходит в `crash_loop_blocked`.
- Автоматическая замена implementation, transport или topology запрещена.
- Очереди и durable state не очищаются при restart.
- Неоднозначный результат внешней non-idempotent операции не повторяется
  автоматически и фиксируется как `unknown_outcome`.
- Состояние ошибки одного модуля отображается как degraded/blocker только для
  затронутых capabilities.

Отказ core supervisor, storage runtime или обязательного control plane является
ошибкой всего приложения. Отказ domain, workflow или integration runtime не
завершает остальные runtime.

Полный автомат состояний Kernel, критерии `recovery_only`, `degraded`, `ready`
и исчерпывающая граница его ответственности определены в ADR-0206.

### Resource isolation

Manifest и composition задают отдельные бюджеты как минимум для:

- одновременных RPC requests;
- PostgreSQL client/server connections;
- NATS consumers и pending acknowledgements;
- in-flight commands;
- spool/outbox backlog;
- CPU-heavy jobs;
- shutdown/drain time.

Бюджеты всегда bounded. Исчерпание ресурса одного модуля создаёт backpressure
или blocker этого модуля, а не неограниченный рост общей очереди.

### Capability security

Runtime получает только явно заявленные capabilities. Module identity
используется при выдаче:

- PostgreSQL role;
- NATS publish/subscribe permissions;
- vault credential lease;
- blob capability;
- control-plane authorization.

Секреты не передаются через argv, environment или logs. Модуль не может
самостоятельно расширить capability set после запуска.

## Отклонённые варианты

### Все модули как Rust traits внутри core

Отклонено, потому что не обеспечивает независимый restart и failure isolation.

### Dynamic libraries и Rust ABI

Отклонено из-за отсутствия стабильного Rust ABI и сохранения общей failure
domain процесса.

### Контейнер на каждый модуль

Отклонено как обязательная desktop topology из-за стоимости упаковки,
обновления и эксплуатации. Контейнеры остаются test/development mechanism.

### Произвольные прямые вызовы между модулями

Отклонено, потому что адресация implementation создаёт скрытый coupling и
обходит capability router, audit и lifecycle state.

## Последствия

Положительные:

- процесс можно перезапустить независимо от остальных;
- contracts и implementations имеют разные причины изменения;
- ядро остаётся техническим supervisor, а не god service;
- permissions, resource budgets и health становятся явными;
- realtime integrations не диктуют topology доменным модулям.

Отрицательные:

- появляются IPC serialization, protocol versioning и process packaging;
- каждый модуль обязан обрабатывать недоступность зависимой capability;
- integration tests должны проверять реальные child processes, а не только
  in-process mocks;
- cross-domain workflows становятся явно asynchronous там, где требуется
  durable coordination.

## Проверка решения

До признания реализации завершённой должны существовать executable checks:

- dependency guard по package role;
- negative test на domain-to-domain и integration-to-domain dependencies;
- protocol/manifest mismatch fails closed;
- crash одного runtime не завершает core и соседний runtime;
- crash-loop budget блокирует только неисправный runtime;
- ordered quiesce, drain, stop и forced kill;
- отсутствие автоматического topology fallback;
- capability denial для незаявленных storage, NATS, vault и blob operations;
- отсутствие secrets и private payloads в health, errors и logs.
