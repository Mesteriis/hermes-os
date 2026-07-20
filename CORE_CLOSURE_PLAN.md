# Core Closure Plan

Статус: в работе. Этот файл описывает только подтверждённое состояние
clean-room Core и оставшиеся admission-работы; он не открывает phase gates.

## Сделано

- `make -C backend ci` — единственная canonical CI-команда; локально она
  проходит, включая architecture/SRP/boundary policy, workspace tests,
  dependency checks, audit и SBOM.
- Architecture evidence связывает текущие policy/docs hashes с результатами
  validation. Release compiler проверяет воспроизводимость unsigned content
  перед signing.
- Control Store содержит operation journal и hardened LAN-development policy.
- Gateway имеет connection, request и deadline budgets; restartable workers
  изолированы от critical recovery/owner control workers.
- Secure-file policy применяется к чувствительным file readers.
- Vault и Blob имеют component-owned offline backup/verify/restore primitives.
  Blob restore использует private staging и atomic publish.
- ADR-0239 фиксирует exact first-owner Mail/IMAP inventory и read-only limits;
  Mail implementation до admission не создаётся.

## Активный срез: Whole-instance recovery

Нужно завершить единый offline recovery contour:

1. Exact signed media layout для Control Store, Vault, PostgreSQL/Storage,
   Blob, Event Hub и conditional Scheduler.
2. Kernel-owned coordinator, который вызывает только verified,
   component-owned offline ports и не импортирует их implementation crates.
3. Quiesce/capture order, retention/encryption, empty-target restore,
   generation/lease/session fencing.
4. Recreate JetStream topology и exact-byte outbox/inbox replay.
5. Disposable full restore evidence: corrupt media, wrong/replayed authority,
   Blob orphan, PostgreSQL migration ledger, replay and fencing cases.

`whole_instance_backup_v1` остаётся закрытым до получения этого evidence.

## Следующий срез: Client Gateway admission

- owner-device proof и session authorization;
- typed ConnectRPC receipts/status/errors;
- SSE replay/gap/reset/disconnect semantics;
- HTTP/2 TLS, HTTP/3 fallback/no-0RTT и abuse/redaction evidence.

`client_gateway_v1` остаётся закрытым.

## После admission: Mail/IMAP → Communications

- Создать только exact ten-package inventory из ADR-0239.
- Реализовать IMAPS/993 read-only contour, Vault ciphertext handoff, bounded
  sync/body extraction, Blob ticket и durable observation ingress.
- Доказать disposable Dovecot E2E до generated Gateway client и read-only UI.

`first_owner_v1` может открыться только после Whole-instance recovery и Client
Gateway admission, а также conditional Blob/Scheduler evidence.

## Финальный closure

- macOS и native Linux validation evidence;
- two-clean-environment reproducible release artifacts and signing provenance;
- Definition-of-Done audit по каждому требованию, без waiver/ignore;
- обновление policy/docs evidence только после фактического соответствия.
