# ADR-0265: Provider operational client transport admission

Статус: Принято
Дата: 2026-07-24
Состояние реализации: frontend generator выпускает exact owner-specific Mail,
Telegram и Zulip contracts, а provider frontend владеет отдельными
service-specific Connect client units поверх общего Gateway transport. Это
только prerequisite: legacy REST/query surfaces ещё не удалены, поэтому
frontend cutover и соответствующие provider phase gates не закрыты.

Первый exact provider profile:
[ADR-0266: Telegram Kernel admission and event-only Communications handoff](ADR-0266-telegram-kernel-admission-and-event-only-communications-handoff.md).

## Decision

Legacy `/api/v1/communications/*` provider operational routes are not a
transport compatibility layer. Each integration replaces them only after its
own atomic phase admission: exact descriptor, approved grants, signed runtime,
owner-specific generated client contract, Gateway route registration through
the owner-neutral module-client protocol, replay/result semantics and live
conformance.

Telegram, WhatsApp, Mail and Zulip are admitted independently. An admission
does not extend `first_owner_v1`, does not add an integration package to the
Communications owner inventory, and does not allow Gateway to link an
integration implementation. Gateway carries the exact owner-declared module
client envelope; the integration decodes its generated payload. Provider
commands remain integration-owned, while canonical evidence continues through
typed Communications ingress events only.

## Required migration evidence

1. Remove every corresponding frontend legacy REST call and test; no alias,
   proxy, fallback or dual-write remains.
2. Prove generated request/response decoding, deadline, error and replay
   semantics for the admitted integration.
3. Prove compile isolation: Communications does not import the integration and
   Gateway does not link its implementation.
4. Prove signed managed launch, grant/revoke fencing and one live provider
   operational flow plus its neutral evidence event.

Until all evidence exists, the integration client is an open migration gap,
not a reason to keep or recreate Communications routes.
