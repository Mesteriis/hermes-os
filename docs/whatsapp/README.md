# Hermes Communications — WhatsApp Channel

Статус: стартовый audit/spec набор на 2026-06-17.

WhatsApp в Hermes — это **Communication Channel** внутри Communications Domain.
WhatsApp не является отдельным продуктом, отдельным мессенджером и не владеет
Memory, Knowledge, Tasks, Projects, Personas, Organizations, Decisions или
Obligations.

Invariant: A channel is never a domain. A channel is an integration. A
communication is the domain object.

Hermes не проектируется как WhatsApp-клиент-клон. WhatsApp поставляет:

- source evidence;
- provider records;
- provider commands;
- attachments;
- media;
- identity traces;
- realtime events;
- timeline evidence.

```text
Hidden desktop WebView
  -> WhatsApp Adapter
  -> Communication Projection
  -> Events
  -> Timeline
  -> Shared Engines
```

`Hidden desktop WebView` означает контролируемую desktop companion-сессию с
явным owner-visible lifecycle, permissions и status UI. Это не headless scraping,
не невидимая запись и не попытка заменить WhatsApp Web неофициальным API.

## Роль в Communications Domain

WhatsApp использует те же базовые границы, что и другие каналы коммуникации:

- provider state не является source of truth для Hermes;
- raw provider records сохраняются как append-only evidence;
- canonical `communication_messages` являются проекцией;
- provider writes проходят через capability, outbox, policy и audit boundary;
- AI output, indexes и summaries не заменяют source evidence;
- realtime является частью канала, а не косметическим обновлением UI.

Первичный provider source:

```text
WhatsApp Web
```

Первичная реализация:

```text
Local First
Desktop First
Personal Use
```

Meta Business API не является основным источником архитектуры. Он допускается
только как отдельный будущий provider shape с отдельным capability и ADR
решением.

## Ключевые принципы WhatsApp Channel

### Evidence First

Любое WhatsApp-сообщение, статус, медиафайл, реакция, удаление, provider update
или runtime event должны сохраняться как проверяемое evidence. Производные поля
для UI, AI, поиска или Timeline не должны заменять исходные provider records.

### Capability Gated

Каждая provider-side операция должна иметь capability state до появления в UI:

```text
available
degraded
blocked
unsupported
```

Минимальный capability set:

- send;
- reply;
- forward;
- reaction;
- delete;
- media upload;
- media download;
- join group;
- leave group;
- status read;
- status publish;
- voice send.

### Local First

Личные данные, raw evidence, attachment metadata, local blobs, audit и derived
context остаются локальными. WhatsApp Web используется как provider/source
boundary, а не как долговременное хранилище Hermes.

### Owner Controlled

Provider-write commands предлагают действия, но исполняются только через
backend-controlled capability and confirmation boundary. UI не вызывает WebView
или provider adapter напрямую для отправки, удаления, реакции, публикации
статуса или загрузки медиа.

### No Hidden Recording

Calls, voice capture, screen capture, recording and STT не могут быть скрытыми.
Первая версия поддерживает только call metadata, call evidence и timeline
entries. Audio/video capture, call control, recording, live call handling и STT
остаются out of scope до отдельного ADR.

## Provider Accounts

Поддерживаемые account kinds на уровне продукта:

```text
whatsapp_personal
whatsapp_business
```

Для первой архитектурной формы оба account kinds используют WhatsApp Web
companion boundary. `whatsapp_business` означает аккаунт WhatsApp Business App,
связанный через Web companion. Это не Meta Business Platform Cloud API.

Provider kind для первой реализации:

```text
whatsapp_web
```

Future provider kind для Meta Business API должен быть отдельным, например:

```text
whatsapp_business_cloud
```

## Dialogs

WhatsApp Channel должен распознавать dialog source records для:

- private chat;
- group;
- community;
- broadcast;
- status.

Dialog projections являются Communication projections. Они не создают Project,
Persona, Organization или Memory lifecycle.

## Messages

Целевые message classes:

- text;
- reply;
- forward;
- reaction;
- delete;
- edit, если provider/runtime reliably supports it.

Edit history не должен реконструироваться задним числом. Hermes может хранить
только observed versions и source-backed update evidence.

## Media

Целевые media classes:

- photo;
- video;
- document;
- audio;
- voice note;
- contact;
- location;
- sticker;
- gif.

Media bytes не должны храниться в PostgreSQL. PostgreSQL хранит metadata,
hashes, scanner state и local blob references.

## Identity

WhatsApp является phone-centric provider.

WhatsApp Channel сохраняет identity traces:

- phone number;
- `wa_id`;
- display names;
- contact-card evidence;
- group member evidence;
- admin/member role evidence;
- community member evidence.

Эти traces могут создавать Persona candidates, relationship candidates и contact
resolution evidence. WhatsApp Channel не реализует Persona Domain и не объявляет
номер телефона окончательной Persona truth.

## WhatsApp Statuses

WhatsApp Status рассматривается как:

```text
source evidence
timeline evidence
identity signal
```

Statuses не являются отдельным доменом. Они входят в WhatsApp Channel как
provider evidence и могут создавать Timeline evidence, identity signals and
review candidates.

## Voice Notes

Voice notes поддерживаются архитектурно как:

- voice metadata;
- voice attachment;
- local playback;
- future transcript integration point.

STT не входит в первую версию. Transcript integration должен быть отдельным
shared-engine flow с source references и explicit permission boundary.

## Calls

Первая версия поддерживает только:

- call metadata;
- call evidence;
- call timeline entries.

Out of scope:

- audio capture;
- video capture;
- call control;
- recording;
- live call handling;
- STT.

Эти возможности требуют будущего ADR.

## Связь с Timeline

WhatsApp messages, statuses, reactions, edits, deletes, media downloads,
provider-write commands, account lifecycle events and call metadata должны
становиться ordered Timeline evidence.

Timeline не владеет provider adapter logic. WhatsApp Channel поставляет
source-backed events and projections.

## Главные незакрытые области

- WhatsApp Web desktop companion runtime;
- account/session lifecycle and local WebView storage policy;
- operation-level capability contract;
- durable provider-write outbox;
- dialog/message/status/media projections;
- phone-centric identity trace model;
- media download/upload pipeline;
- voice-note playback and future transcript boundary;
- call metadata and explicit no-recording boundary;
- realtime `whatsapp.*` event contracts;
- local validation fixtures and smoke tests.

## Навигация

- [Architecture](architecture.md)
- [Modules](modules.md)
- [API Reference](api.md)
- [Status](status.md)
- [Gap Analysis](gap-analysis.md)
- [Blockers](blockers.md)
