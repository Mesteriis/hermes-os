# Активные ADR clean-room backend

Все ADR предыдущей реализации были вынесены из active documentation 2026-07-15.
Исторический индекс находится в
[legacy documentation reference](../../references/backend-legacy/docs/archive/adr/README.md).
Legacy ADR являются только evidence и контекстом; они не возвращаются в active
policy через ссылки из новых документов.

## Статусы

- `Предложено` — решение обсуждается и ещё не принято.
- `Принято` — решение обязательно для новой реализации.
- `Заменено` — решение полностью заменено более новым active ADR.
- `Отклонено` — решение рассмотрено и не используется.

Поле `Состояние реализации` отделяет принятое решение от факта его реализации.
Статус `Принято` сам по себе не означает, что код уже существует.

## Активные решения

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md)
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md)
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md)
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md)
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md)
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md)
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md)
- [ADR-0207: Канонический реестр бизнес-доменов Hermes](ADR-0207-canonical-business-domain-registry.md)
- [ADR-0208: Allowlist разработки доменов и запрет проекций](ADR-0208-domain-development-allowlist-and-projection-freeze.md)
- [ADR-0209: Kernel Event Hub и контроль подписок](ADR-0209-kernel-event-hub-and-subscription-control-plane.md)
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md)
- [ADR-0211: Backend workspace и физическая структура исходного кода](ADR-0211-backend-workspace-and-source-layout.md)

Эти ADR фиксируют runtime, communication, storage, infrastructure lifecycle и
границу между provider-specific experience и provider-neutral context, а также
единый client gateway для desktop и Android. Конституция Kernel ограничивает
его техническим control plane и фиксирует boot/recovery state machine.
Capability и domain ownership inventory остаётся обязательным до создания
первого production crate. Канонический реестр фиксирует тринадцать начальных
business domains и отделяет их от integrations, workflows и projections.
Текущий implementation allowlist разрешает только Communications, Contacts,
Organizations, Tasks, Calendar, Documents и AI; остальные домены и все
product projections заблокированы.
Event Hub является Kernel control plane над NATS catalog/subscriptions, а
Telemetry Hub обеспечивает независимые от PostgreSQL/NATS локальные logs,
metrics, traces и crash diagnostics через отдельный supervised Collector.
ADR-0211 помещает весь production backend code в `backend/src`, а policy,
scripts, infrastructure и tests — в отдельные backend-owned roots внутри
`backend/`.
