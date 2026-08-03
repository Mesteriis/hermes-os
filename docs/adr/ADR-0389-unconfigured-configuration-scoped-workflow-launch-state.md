# ADR-0389: `unconfigured` launch state для configuration-scoped workflows

Статус: Принято
Дата: 2026-08-03
Состояние реализации: реализовано в Kernel owner control и development assembly;
live gate закрывается повторным `make dev` без child restart exhaustion.

Уточняет:

- [ADR-0214: Durable job platform, Scheduler и runtime reconfiguration](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0222: Kernel Settings Registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0306: Repeatable development release refresh](ADR-0306-repeatable-development-release-refresh-and-successor-fencing.md);
- [ADR-0377: Mail Contacts Sync managed workflow](ADR-0377-mail-contacts-sync-managed-workflow.md).

## Контекст

Обычный workflow без public Settings может запускаться сразу после admission.
Mail Contacts Sync, напротив, имеет только configuration-instance settings и не
имеет корректной runtime composition до первого owner-authorized target.

Development assembly раньше вызывал общий workflow start с пустым target.
Kernel создавал child без settings snapshot, после чего executable корректно
завершался и исчерпывал bounded restart attempts. Registration при этом
ошибочно печаталась как `accepted`.

## Решение

Kernel перед initial workflow launch проверяет signed Settings schema:

- если schema не содержит configuration-instance definitions, применяется
  обычный settings-free workflow launch;
- если schema требует configuration instance и существует `current` target,
  применяется обычный configured successor path;
- если schema требует configuration instance, но `current` target отсутствует,
  child не запускается, а owner control возвращает `launch_state=unconfigured`
  и `runtime_generation=0`.

Development assembly принимает `unconfigured` только от exact workflow start
response, печатает это состояние и продолжает ансамбль. Owner settings apply
по-прежнему создаёт новую runtime identity, передаёт exact snapshot/catalog и
подтверждает `current` только после ready.

## SRP и границы

- Settings schema объявляет target scope;
- Kernel решает, существует ли допустимая runtime configuration;
- assembly отображает результат, но не интерпретирует Mail account IDs;
- workflow runtime получает только собственный settings catalog;
- Mail, Contacts и Communications implementations не импортируются друг в
  друга, координация остаётся event/command based.

## Проверка

1. unit test отличает configuration-instance schema от registration/no-schema;
2. owner-control client принимает только exact пары `accepted/+generation` или
   `unconfigured/0`;
3. `make dev` сообщает Mail Contacts Sync как `unconfigured`, не запускает
   падающий child и остаётся ready;
4. после UI apply runtime запускается с account-scoped configuration target.

## Последствия

Admission больше не маскирует отсутствие обязательной workflow configuration.
Не настроенные workflows остаются видимыми и настраиваемыми без crash loop, а
готовность runtime возникает только после фактического owner apply.
