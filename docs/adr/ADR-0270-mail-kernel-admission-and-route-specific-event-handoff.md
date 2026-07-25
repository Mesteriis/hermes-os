# ADR-0270: Mail Kernel admission and route-specific event handoff

Статус: Принято
Дата: 2026-07-24
Состояние реализации: Частично реализовано. Generated Mail API и descriptor
теперь имеют независимые `mail.sync.v1`/`mail.delivery.v1` routes, три
provider-purpose credential capabilities и один canonical module ID
`hermes-mail-runtime` во всех Mail-produced envelopes. Umbrella `mail.client`
удалён из production code; assembly повторно доказала signed exact descriptor.
Signed managed launch теперь проходит через exact Kernel registration,
owner-approved IMAP sync subset и Kernel-issued Storage/Vault/Blob/Event Hub
bindings. Kernel до relay отклоняет отсутствующий delivery grant и stale
runtime generation. Revoke, live provider sync, event/outage replay и
attachment conformance ещё не реализованы, поэтому
`mail_runtime_admission_v1` закрыт.

Уточняет:

- ADR-0201: Core/module communication and NATS;
- ADR-0204: integration and provider-neutral context boundary;
- ADR-0205: Core Gateway and client transport;
- ADR-0215: module registration and capability grants;
- ADR-0219: managed distribution integrity;
- ADR-0221: module descriptor and capability lifecycle;
- ADR-0256: owner-declared ClientRpc route admission;
- ADR-0261: Communications attachment-anchor handoff;
- ADR-0262: Mail attachment Blob-admission extension;
- ADR-0263: Mail settings and Storage admission artifacts;
- ADR-0269: Mail release assembly unit.

## Контекст

Mail является integration owner. Он взаимодействует с Kernel/Core для
registration, managed launch, settings, Storage, Vault, Blob, Event Hub и
provider-operational ClientRpc routing. Это platform control plane, а не
business-вызов Communications.

Provider-neutral Mail evidence пересекает owner boundary только через durable
typed events. Kernel проверяет grant, runtime generation и route metadata, но
не декодирует Mail payload и не вызывает Communications.

Текущий Mail operational contract объединяет две независимые операции в одном
`mail.client`:

- inbox sync, который читает внешний provider и создаёт observations;
- delivery, которая изменяет внешний provider.

Один grant на обе операции выдаёт лишнее право. Аналогично один
`mail.credentials.v1` объединяет IMAP password, Gmail access token и SMTP
password, хотя configuration instance использует только необходимое
подмножество.

Attachment Blob-admission producer также обязан использовать exact admitted
module identity `hermes-mail-runtime`; сокращённый `mail-runtime` создаёт
вторую identity и нарушает runtime/grant fencing.

## Решение

### Owner и единицы сборки

Production runtime owner:

```text
owner_id  = mail
module_id = hermes-mail-runtime
```

Mail source unit:

```text
hermes-mail-api
hermes-mail-core
hermes-mail-imap
hermes-mail-gmail
hermes-mail-smtp
hermes-mail-persistence
hermes-mail-runtime
```

`hermes-mail-assembly` является отдельной integration-owned build-time unit.
Она создаёт unsigned release input, не запускается Kernel и не входит в
runtime inventory или GrantSet.

Communications остаётся отдельным domain owner. Единственная разрешённая
integration → domain compile dependency — typed neutral contract
`hermes-communications-ingress`. Ни один Mail package не импортирует
Communications domain/persistence/runtime/API, а Communications не импортирует
Mail packages.

### Kernel/Core control plane

Kernel:

- регистрирует exact descriptor bytes как `pending`;
- применяет explicit owner-approved capability subset;
- проверяет signed executable/descriptor/settings/storage bindings;
- выдаёт monotonic runtime generation и grant epoch;
- создаёт fenced Storage, Vault, Blob и Event Hub routes;
- маршрутизирует opaque ClientRpc bytes по exact approved contract;
- отзывает routes и leases при suspend, revoke, binding replacement или stale
  runtime identity.

Kernel не:

- декодирует Mail API или Communications ingress payload;
- выбирает provider, mailbox, sync window или delivery recipient;
- хранит provider credential/session или Mail projection;
- создаёт Communications evidence;
- вызывает Communications runtime или SQL.

### Route-specific provider operational contracts

Generated Mail Protobuf descriptor set предоставляет два независимых routes:

| Capability | Contract | Connect path | Responsibility |
|---|---|---|---|
| `mail.sync.v1` | `mail.sync.v1` | `/hermes.mail.v1.MailSyncService/Sync` | bounded inbound sync |
| `mail.delivery.v1` | `mail.delivery.v1` | `/hermes.mail.v1.MailDeliveryService/Send` | outbound provider mutation |

Оба contracts имеют `major = 1`, `revision = 1` и exact SHA-256 одного
generated descriptor set. Общий digest не объединяет capabilities: route,
payload type, authority и failure mode различны.

Mail runtime получает exact contract reference из routed
`ModuleClientRequestV1` и декодирует только соответствующий generated request.
Oneof umbrella `MailOperationalService/Execute`, decode probing, REST alias и
fallback запрещены.

Inbox sync и delivery остаются разными functional ports, даже если один
managed process реализует оба.

### Provider credential capabilities

Credential grants разделяются по purpose:

```text
mail.imap.credentials.v1  -> mail_imap_password
mail.gmail.credentials.v1 -> mail_gmail_access_token
mail.smtp.credentials.v1  -> mail_smtp_password
```

Каждый capability optional в descriptor и становится effective только после
explicit approval. Configuration instance с IMAP не получает Gmail/SMTP
credential route; Gmail не получает IMAP; SMTP выдаётся только вместе с
approved delivery configuration.

Runtime не может расширить права через settings: settings содержат только
revision, а Vault route дополнительно проверяет exact capability, purpose,
configuration instance, runtime generation и grant epoch.

### Event-only handoff

Inbound evidence:

```text
External Mail provider
        ↓
Mail runtime
        ↓
Mail-owned PostgreSQL outbox
        ↓ exact DurableEnvelopeV1 bytes
NATS JetStream
        ↓
Communications inbox/deduplication
        ↓
Communications-owned state and events
```

Attachment continuation:

```text
Mail source observation
        ↓ event
Communications attachment anchor
        ↓ communication_attachment_anchor_recorded.v1
Mail owner-local mapping
        ↓ one-use Blob lease
Mail owner-local outbox
        ↓ communication_attachment_blob_admission_observed.v1
Communications CAS projection
```

Во всех Mail-produced envelopes `source.module_id` равен
`hermes-mail-runtime`. Causation, non-zero correlation, exact contract,
runtime generation и grant epoch сохраняются. Kernel маршрутизирует и
ограничивает transport, но не является business producer/consumer.

Запрещены direct Mail → Communications RPC, runtime socket, shared handler,
cross-owner SQL, anchor derivation в Mail и provider download в
Communications.

## Phase gate `mail_runtime_admission_v1`

Backend gate открывается атомарно только при наличии:

1. route-specific generated Protobuf contracts и exact descriptor references;
2. split sync/delivery and IMAP/Gmail/SMTP capability units;
3. signed Mail runtime/descriptor/settings/storage artifacts из ADR-0269;
4. pending registration без прав и explicit owner-approved subset;
5. managed launch с exact runtime generation/grant epoch;
6. exact Storage/Vault/Blob/Event Hub issuance и stale/revoke fencing;
7. live sync route через Core capability router без Mail dependency в Kernel;
8. Mail outbox → NATS → Communications inbox delivery с deduplication и outage
   replay;
9. attachment anchor handoff → Mail mapping → Blob terminal observation с CAS
   conflict/replay evidence;
10. отсутствие provider bodies, credentials, locators и sessions в subjects,
    route metadata, logs, errors и health.

Delivery capability и frontend cutover не доказываются inbound sync gate.
Они требуют отдельного live provider mutation evidence перед включением.
Frontend не используется как proof backend admission.

Открытие gate:

- не расширяет `first_owner_v1`;
- не добавляет Mail в Communications inventory;
- не превращает integration в domain;
- не разрешает Telegram/WhatsApp/Zulip;
- не доказывается одним ADR или только static tests.

## Порядок реализации

1. Разделить generated routes, client ports и credential capabilities.
2. Исправить exact module identity во всех Mail-produced envelopes.
3. Обновить descriptor/assembly regression evidence.
4. Добавить signed managed launch и Kernel fence conformance.
5. Доказать event-only sync и attachment lifecycle.

Каждый крупный slice является отдельным commit и проходит owner tests,
Clippy, architecture/SRP/Cargo boundaries и relevant live conformance.

## Evidence 2026-07-24

Реализованный managed admission slice:

- signed Mail executable/descriptor/settings binding проверяется при managed
  launch;
- explicit approved subset содержит только Blob, Events, Storage, IMAP
  credential и sync; delivery/Gmail/SMTP остаются без grant;
- Mail Storage bundle и все runtime SQL используют owner-scoped
  `hermes_data.mail_*`;
- Mail получает IMAP credential только через exact Vault purpose
  `mail_imap_password` для своей configuration instance;
- focused live test поднимает disposable PostgreSQL, PgBouncer, NATS и реальные
  managed Vault, Storage, Blob, Communications и Mail processes;
- Kernel отклоняет ungranted delivery и stale sync generation до runtime relay.

Проверки:

```text
HERMES_STORAGE_MANAGED_TEST_FILTER=managed_mail_runtime_uses_kernel_leases_and_route_specific_admission node scripts/test-authenticated-storage.mjs 1.97.0
cargo +1.97.0 clippy --locked -p hermes-mail-persistence -p hermes-mail-runtime -p hermes-kernel-recovery-testkit --all-targets -- -D warnings
cargo +1.97.0 test --locked -p hermes-mail-runtime -p hermes-mail-persistence
make -C backend architecture-policy-check architecture-evidence-check srp-policy-check cargo-boundaries-check test-architecture fmt-check
```

Evidence не открывает gate: active sync route не вызывался против provider
fixture, revoke/worker stop не доказан, Mail observation не прошла live
NATS/Communications/outage replay, attachment continuation не выполнен.

## Отклонённые варианты

### Оставить один `mail.client`

Отклонено: sync grant не должен разрешать outbound delivery.

### Оставить один credential capability

Отклонено: активный IMAP account не должен получать Gmail или SMTP secret
purpose.

### Пусть Kernel вызывает Communications

Отклонено: Kernel стал бы owner-specific business facade и интерпретировал бы
payload.

### Пусть Mail вызывает Communications API

Отклонено: direct cross-owner dependency обходит durable event, inbox
deduplication и failure isolation.

## Последствия

Mail получает узкие, независимо выдаваемые capability units и точную границу:
integration общается с Kernel/Core только для platform control/routing, а с
Communications — только durable typed events. Цена — миграция umbrella
contract и дополнительные conformance slices; это обязательная стоимость
least privilege и SRP.
