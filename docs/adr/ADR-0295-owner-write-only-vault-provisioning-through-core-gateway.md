# ADR-0295: Owner write-only Vault provisioning through Core Gateway

Статус: Принято
Дата: 2026-07-26
Состояние реализации: Backend implemented. First-party client gate остаётся
planned.

Уточняет:

- [ADR-0205: Core Gateway and client transport](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0215: module registration and grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0223: encrypted Vault and credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0232: browser device session](ADR-0232-browser-client-device-identity-and-same-origin-session.md);
- [ADR-0236: integration configuration instances](ADR-0236-integration-owners-protocol-adapters-and-configuration-instances.md);
- [ADR-0292: managed integration settings apply](ADR-0292-managed-integration-settings-apply-and-credential-binding.md);
- [ADR-0294: Mail account lifecycle and portability](ADR-0294-mail-account-credential-lifecycle-and-portability.md).

## Контекст

Vault уже поддерживает exact scoped `create`, `replace_cas`, `retire` и
`delete` через HPKE и одноразовые leases. Этот transport принадлежит managed
runtime audience. First-party desktop/Android client не имеет production path
для первоначального ввода или замены provider credential.

Прямой plaintext method в Mail/Zulip/Telegram/WhatsApp, private owner-control
socket из frontend или общий secrets facade нарушили бы одновременно:

- client-only Core Gateway boundary;
- distinction между platform и integration;
- capability approval;
- запрет Kernel/Gateway на credential plaintext;
- write-only semantics ADR-0223.

Disposable live tests могут seed-ить Vault напрямую, но test support не
является product implementation.

## Решение

### Owner and capability authority

Provisioning является platform control operation. Оно не принадлежит
Communications domain и не является integration command.

Клиент выбирает только:

```text
target registration
exact approved Vault-purpose capability
opaque configuration instance
declared purpose and secret class
action
expected secret revision
client operation ID
```

Kernel выводит logical secret owner из approved target registration и
проверяет exact intersection:

```text
current authenticated owner device
∩ approved registration and current GrantSet
∩ descriptor-declared Vault purpose
∩ hard Vault action/class/scope policy
```

Provider identity не интерпретируется. Свободный purpose, arbitrary secret
class, raw logical owner и generic write grant запрещены.

### Three-step public ceremony

Generated platform contract через Core Gateway использует:

```text
Prepare
  -> operation-bound random challenge
Authorize
  -> fresh P-256 browser/device-key proof
  -> exact short-lived one-action Vault lease
Commit
  -> client-sealed HPKE command
  -> sanitized revision/state receipt
```

Challenge связывает session, device, target registration, capability,
configuration instance, purpose, class, action, expected revision, client
operation ID и response-recipient key. Смена любого поля инвалидирует proof.

Обычная cookie session необходима, но сама по себе не является fresh proof.
LAN development mode не допускает provisioning.

### Ciphertext and receipt boundary

Credential plaintext входит только в HPKE plaintext, sealed для current Vault
runtime. Gateway и Kernel видят bounded ciphertext и non-secret fences, но не
credential bytes.

Vault:

- сверяет Kernel authorization и HPKE binding;
- применяет exact action к exact scope;
- сохраняет durable idempotency receipt вместе с mutation;
- не возвращает record ID, wrapping key или credential bytes;
- возвращает только version, operation ID, revision и sanitized state.

Повтор с тем же operation ID и тем же exact intent возвращает прежний receipt.
Reuse operation ID с другим scope, action или payload отклоняется. Неизвестный
результат можно безопасно продолжить новым fresh-proof ceremony с тем же
operation ID.

Публичных `list/get/read/decrypt` операций нет. Integration после успешного
receipt получает только revision через собственный typed account-binding
contract.

### Units of assembly

```text
Gateway provisioning contract/router
  generated wire contract, session admission and bounded transport

Kernel owner provisioning authority
  fresh proof, descriptor/grant checks and opaque Vault relay

Vault provisioning protocol/service/store
  HPKE command, exact mutation and durable idempotency receipt

first-party client host adapter
  non-extractable device proof key, HPKE seal/open and zeroization

integration account lifecycle
  provider-specific binding of sanitized revision

release assembly
  immutable artifacts only
```

Gateway не становится Vault client facade, Kernel не становится credential
store, integration не становится platform, а assembly не реализует ни одну из
этих responsibilities.

## Failure semantics

- stale browser/device session, revoked device или wrong owner: denied;
- stale registration/grant/Vault generation: denied;
- expired challenge/session/lease: retry from `Prepare`;
- CAS mismatch: rejected without mutation;
- response loss after commit: repeat same operation ID through a new ceremony;
- Vault restart: pending ceremony expires; durable receipt remains authority;
- runtime or Settings replacement не запускается автоматически.

Accepted provisioning не означает provider readiness. App отдельно вызывает
integration binding, generic Settings apply и provider readiness query.

## Phase gates

### `owner_vault_provisioning_backend_v1`

Состояние: Implemented.

1. generated Prepare/Authorize/Commit contracts through Core Gateway;
2. authenticated non-development owner device session;
3. operation-bound fresh P-256 proof;
4. exact descriptor/GrantSet/action/class/configuration admission;
5. HPKE plaintext invisible to Gateway/Kernel;
6. durable Vault idempotency and CAS;
7. sanitized revision/state receipt without record IDs;
8. replay, wrong-owner, revoked-device and stale-generation negatives;
9. live managed Vault restart/retry conformance;
10. architecture, SRP, Cargo, Clippy and workspace gates.

### `owner_vault_provisioning_v1`

Состояние: Planned.

1. `owner_vault_provisioning_backend_v1`;
2. desktop host adapter with non-extractable device key and zeroization;
3. Android host adapter with the same generated descriptor;
4. generated frontend client;
5. browser code receives neither platform private key nor Vault root/wrapping
   material;
6. integration-owned credential setup UI using only sanitized receipt.

## Последствия

- Mail portability может быть resumable без экспорта credential material.
- Zulip, Telegram и WhatsApp могут использовать тот же platform ceremony
  только через собственные separately admitted exact capabilities.
- Новые provider integrations не требуют нового generic secrets endpoint.
- Полный `mail_account_portability_v1` остаётся закрыт до client gate, даже
  если backend provisioning реализован.
