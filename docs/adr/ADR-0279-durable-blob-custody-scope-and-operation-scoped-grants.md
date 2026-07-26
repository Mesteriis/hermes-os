# ADR-0279: Durable Blob custody scope and operation-scoped grants

Статус: Принято
Дата: 2026-07-26
Состояние реализации: Реализовано. `BlobQuotaRequestV1` объявляет exact
`custody_scope_id` и непустой набор `write` / `read_range` /
`custody_transfer`; Kernel сохраняет declaration в Control Store schema 40,
проверяет requested operation и подписывает custody scope в data/transfer
grants. `HBLBENC2`, `HBLBM002` и Vault content-key scope используют stable
owner custody плюс key schema revision, а current access fence остаётся
ephemeral authorization. Legacy ciphertext/metadata блокируют startup.
Focused storage, descriptor, Kernel admission и architecture regressions
зелёные; live managed Blob/Vault conformance записывает content одной
registration/runtime/grant identity и читает exact bytes другой identity того
же custody scope, отклоняя другой scope.

Уточняет:

- [ADR-0215: module admission and grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0221: capability lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0223: Vault leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0230: Blob opaque references](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0231: private Blob data sessions](ADR-0231-private-blob-data-session-and-vault-route.md);
- [ADR-0257: evidence-backed custody transfer](ADR-0257-event-backed-blob-custody-transfer-for-canonical-evidence.md);
- [ADR-0275: target-bound cross-owner custody](ADR-0275-target-bound-cross-owner-blob-custody-delegation.md).

## Контекст

Blob data session правильно требует current registration, runtime generation,
grant epoch, one-use session ID и exact operation. Но текущая at-rest модель
использует тот же `BlobAccessFenceV1` как durable encryption и accounting
identity:

- `HBLBENC1` AEAD associated data содержит registration ID, runtime instance,
  runtime generation и grant epoch;
- Vault `secret_revision` для content key равен grant epoch;
- `HBLBM001` сохраняет полный access fence и считает quota по
  registration/capability;
- `BlobQuotaRequestV1` задаёт только `max_bytes`, поэтому approved capability
  технически может запросить `write`, `read_range` или `custody_transfer`.

После допустимого restart, revoke/re-approval либо successor registration
current runtime получает новый access fence. Он должен потерять старые leases,
но не durable owner content. Текущая модель одновременно делает корректный
старый ciphertext нечитаемым, исключает его из quota нового registration и
оставляет retention reservation привязанной к умершему process fence.

Outbound Mail attachments обнаружили вторую проблему. Inbound admission должен
иметь bounded write/transfer authority, а delivery — read-only authority. Если
обе операции используют один capability только ради доступа к тем же bytes,
capability перестаёт быть точной единицей approval и SRP.

Это platform contract gap. Его нельзя обходить Mail-owned кешем plaintext,
повторным provider download, Communications RPC, Kernel data proxy или выдачей
integration generic read/write Blob authority.

## Решение

### Две независимые identity

Blob Platform вводит две разные typed identity:

1. `BlobAccessFenceV1` — ephemeral authorization текущего managed runtime:
   registration ID, capability ID, runtime instance/generation и grant epoch.
2. `BlobCustodyScopeV1` — stable at-rest custody:
   logical owner ID и explicit bounded `custody_scope_id`.

Access fence проверяется до любого data-path или Vault route действия. Restart,
revoke, grant epoch change и stale runtime немедленно инвалидируют session и
lease. Они не изменяют custody scope и не делают durable bytes нечитаемыми для
нового current runtime с отдельно approved capability того же scope.

`custody_scope_id` является opaque architecture identifier, а не business
entity, filesystem path, provider locator или client capability. Он объявляется
в exact module descriptor и scoped внутри logical module owner. Совпадение
строки у разных owners не разделяет custody.

### Exact descriptor и Kernel agreement

`BlobQuotaRequestV1` получает additive обязательные поля:

```text
uint64 max_bytes
string custody_scope_id
repeated BlobQuotaOperationV1 allowed_operations
```

Первая revision поддерживает только:

- `write`;
- `read_range`;
- `custody_transfer`.

Пустой scope, пустой operation set, duplicate/unspecified/unknown operation,
неограниченный token или quota вне Kernel bound делают descriptor invalid.
Compatibility default «пусто означает все операции» запрещён.

Kernel Control Store сохраняет exact custody scope и normalized operation set
вместе с registration/capability quota request. При session issuance Kernel
проверяет:

```text
requested operation
∈ descriptor-declared operation set
∩ owner-approved capability
∩ hard Kernel policy
```

Signed `BlobDataSessionGrantV1` и target side
`BlobCustodyTransferGrantV1` содержат exact custody scope. Blob runtime не
принимает scope от unsigned data request и не выводит его из provider identity.

Два capability одного owner могут разделять custody scope, но сохраняют разные
operation sets. Это явная descriptor declaration, а не скрытая привилегия:

```text
mail.attachment.ingest.blob.v1
  custody = mail.attachment.content.v1
  operations = write, custody_transfer

mail.attachment.delivery.blob.v1
  custody = mail.attachment.content.v1
  operations = read_range
```

Owner approval по-прежнему выдаётся capability, а не custody scope. Наличие
другого approved capability того же scope не расширяет operation set текущего
requester.

### Durable encryption and Vault authority

Новый authenticated content format связывает ciphertext с:

- exact `BlobRefV1` fields;
- logical owner ID;
- custody scope ID;
- stable Blob content-key schema revision.

Registration ID, runtime instance/generation, grant epoch и requesting
capability не входят в at-rest key derivation или AEAD associated data.
Ephemeral access fence остаётся внутри in-memory key lease и обязан совпасть с
current signed session перед использованием key material.

Vault lease audience по-прежнему process-bound и содержит current
registration/runtime/grant fence. Vault record scope и `secret_revision` для
Blob content key становятся durable:

```text
logical owner
+ custody scope
+ opaque Blob reference
+ Blob content-key schema revision
```

Grant epoch не является secret revision. Изменение content-key schema revision
требует отдельного explicit rewrap/migration решения; обычный restart или
re-approval его не меняет.

### Durable technical ledger

Blob-owned technical metadata сохраняет `BlobCustodyScopeV1`, content format
revision и key schema revision вместо старого full access fence. Aggregate
quota, pending write recovery и deletion reservation принадлежат stable custody
scope. Каждая новая mutation или collector action всё равно требует fresh
operation-scoped access grant; persisted custody metadata не является bearer
authorization.

Quota считается по stable owner + custody scope. Если несколько capabilities
разделяют scope, effective maximum для одной регистрации обязан быть exact и
одинаковым; conflicting quotas делают descriptor invalid. Successor
registration не получает второй независимый quota bucket для уже сохранённых
bytes.

### Format cutover

`HBLBENC1` и `HBLBM001` были созданы до первого released owner-data contour и
не гарантируют restart durability. Production gate этого ADR не может
продолжать записывать эти форматы.

Новая реализация обязана:

- писать только versioned stable-custody content и ledger formats;
- fail closed на неизвестной либо смешанной revision;
- не угадывать custody scope по filename, runtime state или provider metadata;
- иметь explicit offline migration/reset preflight до применения к непустому
  instance.

Автоматический in-place rewrite во время read запрещён: read-only operation не
становится скрытой storage mutation, а partial migration не маскируется как
успешный restart. Пока отдельный offline migration не реализован, непустой
legacy Blob root блокирует readiness с sanitized reason. Test/conformance roots
пересоздаются как воспроизводимые state.

## Границы сборки и SRP

- runtime protocol владеет wire enums и signed grant fields;
- Kernel descriptor parser и Control Store владеют admission persistence;
- Kernel Blob session handler владеет current-grant/operation authorization;
- Blob protocol владеет stable custody value object;
- Blob runtime Vault adapter владеет scoped key lease;
- Blob runtime storage владеет content/ledger format;
- integration descriptor владеет своими capability/scope declarations;
- owner storage владеет filenames, MIME metadata, lifecycle и business meaning.

Kernel, Vault и Blob Platform не импортируют Mail, Communications,
Attachment Security или provider packages. Shared custody scope не создаёт
compile dependency между capabilities и не превращает Blob Platform в domain.

## Обязательное evidence

1. Descriptor validation отклоняет missing/duplicate/unknown operations,
   invalid scope и conflicting quotas одного custody scope.
2. Kernel выдаёт session только для descriptor-declared operation и подписывает
   exact custody scope.
3. Write → runtime restart/re-registration → current read того же custody scope
   возвращает exact bytes; stale old runtime/session/lease отклоняются.
4. Grant epoch change инвалидирует old session, но current re-approved
   read-only capability читает существующий Blob.
5. Capability с write-only set не может читать; read-only не может писать или
   transfer; одинаковый scope не расширяет permission.
6. Другой owner, другой custody scope, wrong reference/digest, altered signed
   grant и replay fail closed.
7. Quota сохраняется через registration/runtime changes и считается один раз
   на stable custody scope.
8. Retention/GC после restart требует fresh delete/collector authority и не
   использует persisted access fence как permission.
9. Legacy/nonempty incompatible root блокирует readiness либо проходит
   отдельный offline migration; silent data reset запрещён.
10. Live managed Blob/Vault/Kernel conformance, architecture, SRP, Cargo,
    Clippy и full backend gates зелёные.

## Следующий owner slice

После evidence этого ADR отдельный Mail MIME ADR может:

- сохранить Mail-owned attachment metadata и opaque Blob reference;
- принимать в delivery request только canonical attachment anchor IDs;
- разрешить отправку только после event-backed canonical
  `safe_for_delivery` projection;
- читать bytes one-use receipt-bound Mail delivery capability;
- собирать bounded MIME внутри Mail integration;
- передавать готовый RFC822 только SMTP/Gmail provider adapters.

Communications при этом не вызывается синхронно, не получает provider command и
не читает Mail Blob. Core/Kernel остаются admission/routing platform.

## Отклонённые варианты

### Считать restart новой Blob custody

Отклонено: runtime lifecycle не является owner retention decision и теряет
durable content.

### Использовать capability ID как единственный custody scope

Отклонено: read-only и write-only responsibilities пришлось бы снова слить в
одну approval unit.

### Пустой operation set означает legacy read/write/transfer

Отклонено: это fail-open descriptor evolution.

### Передать attachment bytes через Communications event/RPC

Отклонено: durable envelope не переносит private payload, Communications не
является Blob facade, а Kernel/Event Hub не являются byte proxy.

### Автоматически переписать legacy ciphertext при первом read

Отклонено: read получает скрытую mutation/failure semantics и не может
атомарно закрыть whole-root migration.
