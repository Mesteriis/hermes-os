# Persons — Архитектурные блокеры

## Блокеры текущего backend/API scope

Блокеров для текущего backend/API scope нет: person projection, identity review, memory, timeline, health, analytics, investigator и export реализованы как локальные PostgreSQL-backed модули с protected API.

## Deferred / вне текущего scope

Эти пункты не считаются закрытыми и должны оставаться видимыми в roadmap/status:

- Relationship Map (§75) — зависит от graph traversal/UI слоя.
- Mutual Connections (§76) — зависит от graph traversal/UI слоя.
- Digital Twin (§78) — композитный read-side view, требует отдельного UI/API контракта.
- Реальные enrichment provider adapters — GitHub/LinkedIn/web провайдеры спроектированы как pluggable boundary, но live API adapters не реализованы в этом slice.
- Opaque UUID `person_id` migration — текущий контракт зафиксирован в ADR-0074 как text ID `person:v1:email:{len}:{email}`; UUID migration требует отдельного ADR и миграционного плана.
