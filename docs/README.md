# Документация Hermes clean-room

Статус: проектирование; production backend runtime отсутствует
Дата: 2026-07-15

В этой директории находится только документация новой clean-room системы.
Документы предыдущего backend перенесены в
[`references/backend-legacy/docs`](../references/backend-legacy/docs/) и не
являются действующей policy.

## Активные решения

- [ADR index](adr/README.md)
- [ADR-0200: Модульная модель и изоляция runtime](adr/ADR-0200-clean-room-module-model-and-runtime-isolation.md)
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](adr/ADR-0201-core-module-communication-and-nats.md)
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](adr/ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md)
- [ADR-0203: Управление локальной инфраструктурой и восстановление](adr/ADR-0203-managed-infrastructure-supervision-and-recovery.md)
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](adr/ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md)
- [ADR-0205: Core Gateway и транспорт клиентских приложений](adr/ADR-0205-core-gateway-and-client-transport.md)
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](adr/ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md)
- [ADR-0207: Канонический реестр бизнес-доменов Hermes](adr/ADR-0207-canonical-business-domain-registry.md)
- [ADR-0208: Allowlist разработки доменов и запрет проекций](adr/ADR-0208-domain-development-allowlist-and-projection-freeze.md)
- [ADR-0209: Kernel Event Hub и контроль подписок](adr/ADR-0209-kernel-event-hub-and-subscription-control-plane.md)
- [ADR-0210: Telemetry Hub и локальная диагностика](adr/ADR-0210-telemetry-hub-and-local-diagnostics.md)
- [ADR-0211: Backend workspace и физическая структура исходного кода](adr/ADR-0211-backend-workspace-and-source-layout.md)

## Канонические summaries

- [Architecture overview](architecture/architecture-overview.md)
- [Component communication contract](architecture/component-communication.md)
- [Process/container topology](architecture/container-diagram.md)
- [Executable architecture policy](../backend/architecture/README.md)

## Что ещё не определено

Активных подробных product, domain, provider, workflow, AI, vault, UI, testing,
deployment и operations specifications пока нет. Они должны появляться заново
после обсуждения соответствующей границы и не могут восстанавливаться из legacy
documentation как действующий contract.

Принятый ADR сам по себе не означает, что решение реализовано. Фактическое
состояние указывается внутри каждого ADR и меняется только после executable
evidence и targeted validation новой системы.
