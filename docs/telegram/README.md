# Hermes Communications — Telegram Channel

Статус: канонический audit/spec набор на 2026-06-15.

Telegram в Hermes — это **Communication Channel** внутри Communications Domain.
Telegram не является отдельным продуктом, отдельным мессенджером и не владеет
Memory, Knowledge, Obligations, Decisions, Projects, Organizations или Personas.

Hermes не проектируется как Telegram-клиент-клон. Telegram поставляет:

- source evidence;
- provider records;
- provider commands;
- локальный desktop workbench;
- медиа и вложения;
- realtime-события;
- identity traces для Personas;
- материал для Timeline и shared engines.

```text
Telegram Provider
  -> Raw Records
  -> Communication Projection
  -> Events
  -> Timeline
  -> Shared Engines
```

## Роль в Communications Domain

Telegram использует те же базовые границы, что и другие каналы коммуникации:

- provider state не является единственным source of truth для Hermes;
- raw provider records сохраняются как append-only evidence;
- canonical `communication_messages` являются проекцией;
- provider writes требуют capability, policy и audit boundary;
- AI output и derived indexes не заменяют source evidence;
- realtime является частью канала, а не косметическим обновлением UI.

Текущая реализация находится в:

```text
backend/src/integrations/telegram/
frontend/src/domains/telegram/
```

Она уже содержит account/runtime/message foundation, но не закрывает production parity Telegram.

## Ключевые принципы Telegram Channel

### Evidence First

Любое Telegram-сообщение, действие провайдера, медиафайл или runtime-событие
должны сохраняться как проверяемое evidence. Производные AI/UX-поля не должны
заменять исходные данные.

### Capability Gated

Каждая provider-side операция должна иметь capability state до появления в UI:

```text
available
blocked
degraded
unsupported
```

Особенно для destructive/high-risk действий:

- delete;
- edit;
- pin/unpin;
- reactions;
- join/leave;
- admin actions;
- call/recording/audio capture;
- export/session/proxy operations.

### Local First

Личные данные, история, raw evidence, медиа metadata, audit и derived context
должны оставаться локальными. Provider используется как transport/source boundary.

### Owner Controlled

AI, automation и provider-write commands предлагают действия, но владелец или
явная policy подтверждают исполнение.

### No Hidden Recording

Любая работа с calls, audio capture, voice/video recording и STT требует явного
permission boundary. Скрытая запись не поддерживается.

## Связь с Personas

Telegram users, senders, participants, usernames, phone traces и sender metadata
рассматриваются как identity traces для Personas.

Telegram Channel не создаёт отдельный Contacts/Address Book domain.

Текущая реализация сохраняет:

- `sender_id`;
- `sender_display_name`;
- `provider_chat_id`;
- raw TDLib payload;
- message metadata.

Более глубокие Persona flows остаются внешними точками интеграции:

- identity resolution;
- relationship scoring;
- trust;
- dossier;
- communication DNA.

## Связь с Organizations

Telegram groups, channels, bots и organization proxy accounts могут быть evidence
для Organizations. Telegram не владеет Organization lifecycle.

Будущие integration points:

- group/channel → organization candidate;
- bot/service account → organization system identity;
- channel announcements → organization timeline evidence;
- project group → organization/project relationship evidence.

## Связь с Projects

Telegram messages могут быть evidence для:

- project context;
- task candidates;
- decision candidates;
- meeting/event hints;
- obligation candidates;
- document/media references.

Текущая foundation уже может обновлять Decision/task candidates из projected
Telegram messages, но review lifecycle принадлежит внешним shared engines.

## Связь с Attachments

Telegram media моделируется как Communication attachments.

Правила:

- media bytes не хранятся в PostgreSQL;
- PostgreSQL хранит metadata, hash, scan state и local refs;
- blob storage остаётся локальным;
- scanner boundary общий для всех communication attachments;
- Telegram не должен зависеть семантически от Mail storage naming.

Текущая реализация использует mail-named compatibility boundary. Это технический
долг, а не продуктовая модель.

## Связь с Timeline

Telegram messages, account lifecycle events, provider-write audits, sync events,
edit/delete/reaction events и future call/media events должны становиться
ordered Timeline evidence.

Текущая реализация:

- проецирует сообщения в `communication_messages`;
- сохраняет selected audit events;
- имеет Telegram-specific realtime contracts for message, chat, reaction, sync,
  typing, topic, media and command-status events;
- не имеет полноценного first-class Timeline feed для Telegram.

## Главные незакрытые области

- detailed capability contract;
- remaining Telegram-specific realtime provider reconciliation;
- edit/delete/tombstone/version schema;
- reply/forward/reaction/topic projection;
- provider-write command model beyond send;
- Telegram media gallery/search/preview UX;
- voice/video/calls native permission boundary;
- Bot API runtime;
- session/proxy bundle management;
- provider search/export parity.

## Навигация

- [Architecture](architecture.md)
- [Modules](modules.md)
- [API Reference](api.md)
- [Status](status.md)
- [Gap Analysis](gap-analysis.md)
- [Blockers](blockers.md)
