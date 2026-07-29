# ADR-0329: Full-stack development Attachment scanner contour

- Статус: принято
- Дата: 2026-07-29
- Состояние реализации: реализовано и подтверждено live generation 95
- Уточняет: ADR-0273, ADR-0300, ADR-0327

## Контекст

Attachment Security runtime и ClamAV adapter являются отдельными engine build
units, но реальный verdict требует доступного scanner daemon. Ранее `make dev`
поднимал PostgreSQL, PgBouncer и NATS, а `clamd` на canonical loopback endpoint
`127.0.0.1:3310` отсутствовал. Custody transfer завершался, после чего jobs
накапливали bounded `Scanner` retries; UI не мог получить реальный verdict.

Fake scanner response, embedded domain scanner и ручной запуск внешнего daemon
нарушили бы full-stack contract `make dev` и скрыли бы operational dependency.

## Решение

Authenticated development Compose поднимает pinned
`clamav/clamav:1.5.3-debian13-slim` вместе с остальной инфраструктурой.

- daemon публикуется только на `127.0.0.1:3310`;
- Compose healthcheck обязан подтвердить `clamdscan --ping 1` до продолжения
  assembly;
- Attachment Security runtime сохраняет отдельные core, adapter, persistence и
  runtime build units;
- scanner daemon является development infrastructure, а не domain или
  integration;
- verdict создаётся только из реального scanner response;
- provider content и scanner payload не попадают в Compose configuration,
  Kernel Control Store или logs;
- обычная остановка `make dev` не удаляет owner data и pending scan jobs.

## Retry-policy reconciliation

Jobs, исчерпанные до появления обязательного scanner contour, переоткрываются
ровно один раз owner-local policy revision 3. Reconciliation принимает только
terminal job с подтверждёнными `target_blob_reference_id` и
`target_blob_receipt_sha256`, без verdict outbox и с exact policy revision 2.
Lease, attempts и completion сбрасываются, но custody receipt сохраняется.

Отдельная additive Storage migration revision 5 добавляет partial index только
для этого предиката. Новые jobs сразу создаются с policy revision 3. Generic
terminal requeue API, infinite retry и platform-доступ к owner tables
запрещены.

## Ownership и единицы сборки

- authenticated Compose владеет только lifecycle development infrastructure;
- ClamAV image владеет scanner daemon;
- Attachment Security adapter владеет ClamAV wire protocol;
- Attachment Security persistence владеет durable retry reconciliation;
- runtime координирует owner-local scan use case;
- Communications получает только typed durable verdict event;
- Kernel, integrations и domains не импортируют scanner implementation.

## Evidence

1. architecture gate проверяет pinned image, loopback bind и healthcheck;
2. Storage AST gate принимает additive revision 5;
3. persistence tests подтверждают exact revision predicates;
4. `make dev` ждёт healthy ClamAV до запуска managed modules;
5. live jobs переходят в completed state и создают exact verdict outbox;
6. `make pre-commit` проходит.

Live generation 95 дождался healthy ClamAV на loopback и завершил все восемь
retryable jobs. У всех восьми сохранены target Blob receipt и verdict outbox;
все восемь verdict events опубликованы. Эти строки завершились в пределах
существующего revision 2 retry budget, поэтому recovery revision 3 к ним не
применялся.

## Последствия

`make dev` снова означает полный локальный ансамбль для Attachment Security.
Недоступность scanner остаётся честным fail-closed состоянием, но штатная
development assembly больше не создаёт её сама.
