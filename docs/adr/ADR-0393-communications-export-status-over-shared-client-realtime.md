# ADR-0393: Communications export status over shared client realtime

Статус: Принято

Дата: 2026-08-04

Состояние реализации: implemented. Owner-local migration/replay, managed
Gateway SSE, frontend no-polling и полный pre-push gate подтверждены этим
слайсом.

Уточняет:

- [ADR-0205: Core Gateway and client transport](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0318: Communications evidence export workflow](ADR-0318-communications-evidence-export-workflow.md);
- [ADR-0337: Capability-routed managed client realtime](ADR-0337-capability-routed-managed-client-realtime.md);
- [ADR-0338: Client system status over shared realtime](ADR-0338-client-system-status-over-shared-realtime.md).

## Контекст

Frontend evidence export после `StartEvidenceExport` выполняет до 80
`GetEvidenceExportStatus` запросов по таймеру. Это дублирует принятый один
authenticated replayable SSE stream и делает refresh transport-ом вместо
явного recovery/query action.

Export status принадлежит workflow `communications_export`. Communications
domain и provider integrations не должны публиковать или читать эту truth.
Artifact/body bytes, Blob proofs и provider metadata в realtime запрещены.

## Решение

`communications_export` добавляет в существующую capability
`communications.export.v1` один exact `client_realtime` surface:

```text
contract_name    = communications.export.status_changed
contract_version = 1
event_kind       = communications.export.status_changed
```

Typed payload содержит только export ID, status, requested/completed counts,
bounded artifact byte count, occurred time и public error code. PostgreSQL
workflow owner атомарно записывает sanitized transition вместе с каждым job
state change. Runtime публикует owner-scoped replay window через Kernel managed
client-realtime request. Restart повторяет непереданные transitions; Gateway
deduplicate/replay authority остаётся platform-owned.

Frontend открывает существующий `BrowserGatewayRealtimeHub` до Start, затем
привязывает полученный export ID и применяет только exact typed events. Один
initial/recovery `GetStatus` допустим; periodic polling, второй SSE endpoint и
новый EventSource запрещены. Replay gap/protocol error оставляют explicit
Refresh как recovery action и не запускают скрытый timer.

## Границы

- workflow не импортирует Communications или integration implementation;
- domain/integration не импортируют export runtime/persistence;
- realtime не переносит artifact bytes, private content, Blob reference/proof,
  provider/account identity или internal durable envelope;
- Gateway не интерпретирует export semantics;
- migration, runtime publisher, frontend adapter и presentation остаются
  отдельными responsibilities/build units.

## Проверка

1. storage successor атомарно сохраняет ordered owner-local transitions;
2. descriptor предоставляет exact ClientRealtime contract;
3. managed restart/replay достигает pre-opened Gateway SSE без status polling;
4. wrong owner, malformed payload, replay gap и duplicate event fail closed;
5. frontend controller не содержит timer/poll loop и использует shared hub;
6. private-content/privacy scan и полный `make pre-push` проходят.

## Последствия

Status transitions приходят сразу и переживают reconnect/restart. `GetStatus`
остаётся query snapshot для initial/manual recovery, а не transport. Telegram QR
authorization и provider-side long polling не смешиваются с этим workflow
client realtime contract.
