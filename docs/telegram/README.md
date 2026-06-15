# Telegram Domain Audit

Статус: канонический audit set на 2026-06-15.

Telegram внутри Hermes - это Communication Channel внутри Communications
Domain. Telegram не является отдельным продуктом.

Hermes не проектируется как Telegram client или messenger clone. Telegram
поставляет source evidence, provider commands и локальный desktop workbench,
которые затем проецируются в Communications, Timeline и shared engines.

```text
Telegram Provider
  -> Raw Records
  -> Communication Projection
  -> Events
  -> Timeline
  -> Shared Engines
```

## Роль в Communications Domain

Telegram использует те же базовые границы, что и остальные communication
channels:

- provider state не является source of truth для Hermes;
- raw provider records сохраняются как append-only evidence;
- canonical `communication_messages` являются проекцией;
- provider writes требуют capability/policy/audit boundary;
- AI output и derived indexes не заменяют source evidence.

Текущая реализация находится в `backend/src/integrations/telegram/` и
`frontend/src/domains/telegram/`. Она уже содержит account/runtime/message
foundation, но не закрывает production parity Telegram.

## Связь с Personas

Telegram users, senders, participants и usernames рассматриваются как identity
traces для Personas. Они не создают отдельный Contacts или Address Book domain.

Текущая реализация сохраняет `sender_id`, `sender_display_name`,
`provider_chat_id` и raw TDLib payload в message metadata. Более глубокая
Persona identity review, relationship scoring и dossier enrichment остаются
точками интеграции, а не частью Telegram Domain scope.

## Связь с Organizations

Groups, channels, bots и organization proxy accounts могут давать evidence для
Organizations. Telegram Channel не владеет Organizations. Он должен передавать
source-backed observations в shared Organization/Persona review paths, когда
такая интеграция реализуется.

## Связь с Projects

Telegram messages могут быть evidence для project context, project links и
decision/task candidates. Current foundation refreshes Decision and
obligation-derived task candidates from projected Telegram messages, but the
review lifecycle remains owned outside Telegram.

## Связь с Attachments

Telegram media is modeled as Communication attachments. Media bytes must stay in
local blob storage, not PostgreSQL. PostgreSQL stores metadata, hashes, scan
state and local references.

Current implementation uses the existing communication attachment/blob safety
scanner boundary through mail-named storage modules. This is an implementation
compatibility label, not a product statement that Telegram belongs to Mail.

## Связь с Timeline

Telegram messages, account lifecycle actions, provider-write audits and future
sync/realtime updates should become ordered Timeline evidence. Current
implementation projects messages into `communication_messages` and records
selected audit events, but Telegram-specific realtime event contracts and a
first-class Timeline feed remain incomplete.

## Навигация

- [Architecture](architecture.md)
- [Modules](modules.md)
- [API Reference](api.md)
- [Status](status.md)
- [Gap Analysis](gap-analysis.md)
- [Blockers](blockers.md)
