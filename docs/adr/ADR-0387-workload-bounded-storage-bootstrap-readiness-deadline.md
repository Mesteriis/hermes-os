# ADR-0387: Workload-bounded Storage bootstrap readiness deadline

Статус: Принято
Дата: 2026-08-03
Состояние реализации: запланировано этим ADR; live gate не закрыт до успешного
повторного `make dev` на сохранённом Control Store.

Уточняет:

- [ADR-0203: Managed infrastructure supervision](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0224: Storage Control Plane](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0306: Repeatable development release refresh](ADR-0306-repeatable-development-release-refresh-and-successor-fencing.md).

## Контекст

Общий managed-runtime readiness timeout равен 15 секундам. Storage Control до
ready обязан последовательно получить две platform credentials и по одной
runtime credential для каждой active Storage binding, затем проверить roles,
migrations и PgBouncer configuration.

После admission 16 независимых clean-room units bootstrap выполняет 18
Vault exchanges. Два последовательных `make dev` завершились ровно generic
timeout, хотя child не упал и authenticated infrastructure была healthy.
Повтор или увеличение global timeout скрыли бы различие между простым runtime и
Storage workload.

## Решение

Supervisor сохраняет общий 15-секундный default, но предоставляет caller-owned
bounded `wait_until_ready_with_timeout`.

Storage launch вычисляет deadline только из уже проверенного exact количества
desired active bindings:

```text
15 seconds base + 2 seconds * active binding count, capped at 120 seconds
```

Deadline не берётся из environment, module payload или provider settings. Он не
ослабляет lifecycle fencing: timeout по-прежнему останавливает exact Storage
generation и возвращает startup failure. Другие runtimes сохраняют общий
default.

## SRP

- Supervisor владеет ожиданием и остановкой child;
- Storage launch владеет оценкой своего обязательного bootstrap workload;
- Vault и Storage runtime не знают Kernel deadlines;
- domain/integration/workflow units не могут менять platform timeout.

## Проверка

1. unit tests подтверждают base, 16-binding и cap cases;
2. существующие supervisor timeout tests сохраняют default behavior;
3. `make dev` проходит на сохранённом Control Store с 16 active bindings;
4. повторный запуск не удаляет registrations, settings, Vault или provider state.

## Последствия

Storage bootstrap остаётся bounded и fail closed, но deadline соответствует
admitted workload. Рост числа независимых units больше не превращает корректную
последовательную credential reconciliation в ложный crash.
