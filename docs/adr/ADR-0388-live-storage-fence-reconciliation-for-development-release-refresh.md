# ADR-0388: Live Storage fence reconciliation для development release refresh

Статус: Принято
Дата: 2026-08-03
Состояние реализации: реализовано в clean-room development assembly; live gate
закрывается повторным `make dev` на сохранённом Control Store после
account-scoped Settings apply и прерванного revocation.

Уточняет:

- [ADR-0224: Storage Control Plane](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0300: Root full-stack development ensemble](ADR-0300-root-full-stack-development-ensemble.md);
- [ADR-0306: Repeatable development release refresh](ADR-0306-repeatable-development-release-refresh-and-successor-fencing.md).

## Контекст

Development assembly хранит checkpoint последней выданной Storage binding,
role epoch и credential revision. Owner-authorized Settings apply может после
этого checkpoint запустить managed successor того же integration runtime и
увеличить все три fence.

Следующий `make dev` раньше пытался revoke binding по checkpoint revision.
Control Store корректно отклонял stale compare-and-swap, поэтому release refresh
останавливался, хотя live binding был active и валиден. Локальный plan-файл не
может быть authority для текущей runtime identity.

## Решение

Перед каждым successor release development assembly читает exact
owner-authorized `GetManagedStorageBindingStatus` для registration и capability.

- live binding revision, role epoch и credential revision не могут быть меньше
  checkpoint;
- successor fences вычисляются только от live epochs;
- active binding переводится в revoking по live binding revision;
- уже revoking binding повторно передаётся тому же public owner operation;
- если первая попытка fencing останавливает Storage process и возвращает
  sanitized `operation_denied`, assembly делает ровно одну повторную попытку:
  остановка process синхронна, поэтому retry завершает durable revocation без
  неограниченного polling;
- только подтверждённая revocation разрешает upgrade registration;
- неизвестное состояние и regression закрывают gate fail closed.

После issue нового binding новый checkpoint по-прежнему сохраняется только из
Kernel receipt. Assembly не читает Control Store напрямую и не получает
credential material.

## SRP и границы

- Kernel/Control Store остаётся authority для live binding и CAS revocation;
- development assembly координирует release successor через public owner
  control contract;
- Settings apply и integration runtime не знают о локальном assembly checkpoint;
- domain, integration и workflow units не импортируют assembly implementation.

## Проверка

1. unit tests покрывают live fences новее checkpoint, regression rejection и
   единственный bounded retry после неполной revocation;
2. повторный `make dev` проходит после Mail account-target apply без reset;
3. новые Mail/Telegram runtimes получают successor generations;
4. Control Store, Vault и provider session state сохраняются.

## Последствия

Development refresh больше не предполагает, что только он меняет runtime
fences. Owner Settings reconfiguration и release replacement образуют один
монотонный lifecycle без stale revoke и без bypass Control Store authority.
