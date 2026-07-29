# ADR-0330: Provider-neutral communication delivery intent workflow

Статус: Принято

Дата: 2026-07-29

Состояние реализации: typed public request/result contract, pure planning core,
owner-local persistence и independently managed runtime substrate реализованы.
Persistence принимает только sealed ciphertext, хранит owner-scoped
idempotency/state transitions и выдаёт lease/claim с epoch fencing. Runtime
проходит Kernel handshake, поднимает owner-local Storage через Vault-fenced
credential и пока предоставляет только Storage capability. Отдельная assembly
unit создаёт canonical descriptor/settings/Storage bundle и deterministic
unsigned release fragment, не исполняя workflow. Provider command adapters,
client route и live admission ещё не реализованы. Четыре provider-owned
command/result wire contracts и exact route requests реализованы по ADR-0331;
`communication_delivery_intent_v1` остаётся `planned`.

## Контекст

Provider-specific compose принадлежит Mail, Telegram, WhatsApp или Zulip.
Однако delayed delivery, bounded bulk action и explicit cross-channel forward
требуют общий provider-neutral intent. Помещать его в Communications нельзя:
Communications владеет canonical evidence, а не provider execution. Помещать
его в Kernel/Core или в одну integration также нельзя: это создало бы business
facade либо cross-provider owner.

## Решение

Вводится отдельный workflow owner `communication_delivery_intent` с
независимыми единицами сборки:

```text
hermes-communication-delivery-intent-api
hermes-communication-delivery-intent-core
hermes-communication-delivery-intent-persistence
hermes-communication-delivery-intent-runtime
hermes-communication-delivery-intent-assembly
```

Публичный V1 request принимает только:

- idempotent `operation_id`;
- canonical Communications `conversation_id`;
- optional canonical `reply_to_message_id`;
- bounded UTF-8 body.

Caller не выбирает integration, provider account или provider target. Workflow
читает canonical conversation/message через public
`communications_canonical_read_v2`, получает provider provenance и opaque
account/conversation/source cursors, затем передаёт их ровно одному
provider-owned command adapter. Integration самостоятельно разрешает свои
opaque cursors в operational target и создаёт provider command. Workflow не
импортирует integration persistence и не хранит provider operational truth.

Новый provider conversation или arbitrary recipient не входят в V1: это
provider-specific compose соответствующей integration. Расширение требует
отдельного typed target contract, а не generic map или `execute(any)`.

## Согласование с Kernel/Core

Kernel:

- регистрирует workflow как отдельный module owner;
- выдаёт capability grants и runtime generation fences;
- не декодирует request body и provider command payload;
- не выбирает provider и не становится command facade.

Core capability router:

- маршрутизирует exact generated workflow contract;
- возвращает receipt, где `accepted` не означает provider completion;
- доставляет terminal status через query/replayable client realtime;
- не импортирует Mail, Telegram, WhatsApp или Zulip implementation.

Durable workflow events содержат identity, causation, correlation и state, но
не body. Private body хранится только в owner-local encrypted custody,
материализуется по scoped lease и не попадает в subjects, logs, health или
errors.

Persistence unit не принимает `PlannedDeliveryIntentV1`, потому что planning
object содержит plaintext body. Граница persistence принимает отдельно
canonical identity, opaque route metadata и `SealedDeliveryBodyV1`. Ключ
идемпотентности scoped парой `(logical_owner_id, intent_id)`, а worker claim
ограждён `claim_epoch`, owner id и сроком lease. Terminal transition удаляет
ciphertext custody.

Runtime unit принимает plaintext только как consumed planning object и передаёт
его exact `DeliveryIntentBodySealerV1`; persistence command создаётся только
после получения `SealedDeliveryBodyV1`. Текущий descriptor намеренно не
объявляет client RPC, durable publisher/consumer или provider command surface:
эти grants будут добавлены атомарно вместе с соответствующими typed adapters,
а не как неработающие facade routes. Managed process уже реализует Kernel
describe/ready handshake, runtime/grant fences, Vault-issued Storage credential,
PgBouncer connection budget и fail-closed control-frame pump.

Assembly unit зависит только от runtime/persistence и общих release contracts.
Она принимает exact runtime executable, запрещает relative, missing, empty и
symlink input, никогда не перезаписывает output и публикует только unsigned
sorted artifact fragment. Подпись и distribution admission остаются generic
platform responsibility; assembly не является runtime, integration или
business owner.

## Инварианты planning core

- operation, conversation и optional reply identities имеют exact fixed width;
- reply обязан принадлежать той же canonical conversation и быть active;
- account/conversation/source cursors остаются opaque;
- body должен быть non-empty valid UTF-8 и не больше 64 KiB;
- core не логирует и не форматирует body;
- provider acceptance и provider observation являются разными состояниями.

## Completion gate

`communication_delivery_intent_v1` становится `implemented` только после:

1. owner-local idempotent persistence и durable state transitions;
2. managed runtime с exact grants, generation fencing и revoke;
3. typed adapters для каждого admitted provider target;
4. generated Gateway/client command, status и realtime contracts;
5. outage replay, duplicate suppression и terminal provider evidence tests;
6. live managed proof без content leakage и cross-owner storage access.

До закрытия всех пунктов reconstruction matrix остаётся `planned`.

Exact provider-owned event boundary и отсутствие generic provider facade
зафиксированы ADR-0331.
