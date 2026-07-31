# ADR-0359: Bounded attachment archive inspection engine

Статус: Принято

Дата: 2026-07-31

Состояние реализации: частично реализовано. Отдельные
`hermes-attachment-archive-inspection-api`,
`hermes-attachment-archive-inspection-core` и
`hermes-attachment-archive-inspection-zip` units реализуют provider-neutral
client contract, pure bounded policy и ZIP metadata adapter без extraction.
`hermes-attachment-archive-inspection-persistence` реализует owner-local
request idempotency, exact message/hash inbox, порядок-независимый join,
bounded report/realtime storage и job lease fencing по worker, runtime
generation, grant epoch и monotonic fence. Managed runtime, release assembly,
event decoding, Blob custody/read, live PostgreSQL/NATS/Gateway conformance и
production gate `attachment_archive_inspection_v1` остаются открыты.

Зависит от:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0230](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0246](ADR-0246-communications-attachment-admission-and-safety.md);
- [ADR-0273](ADR-0273-attachment-security-engine-and-event-only-verdict-authority.md);
- [ADR-0274](ADR-0274-attachment-security-evidence-bound-blob-custody.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md).

## Контекст

Legacy Communications содержал bounded ZIP metadata inspection прямо внутри
domain implementation. Он отклонял path traversal, password-protected и nested
archives, excessive entry count/depth и excessive declared uncompressed size.
Такое поведение требуется восстановить, но переносить parser, Blob read и
archive policy внутрь clean-room Communications нельзя: это создаст вторую
причину изменения domain и вернёт content-processing facade.

Attachment Security уже является отдельным engine и остаётся единственным
authority для `safe_for_delivery`. Archive inspection не является scanner
verdict, не изменяет safety lifecycle и не получает право объявлять вложение
безопасным.

## Решение

Вводится отдельный engine:

```text
owner_id  = attachment_archive_inspection
module_id = hermes-attachment-archive-inspection-runtime
kind      = engine
```

Engine запускается и обновляется независимо от Communications, Attachment
Security и provider integrations. Он меняется по причине изменения bounded
archive policy/parser support.

### On-demand flow

Пользовательский запрос адресуется через Core Gateway в exact typed archive
engine client contract. Он содержит только stable operation ID и canonical
attachment anchor. Blob reference, filename, MIME, provider/account identity,
path и bytes клиент не передаёт.

Engine durably объединяет три независимо поступающих факта:

1. client request для exact attachment anchor;
2. существующий provider-neutral
   `attachment_security_scan_candidate_observed.v1`;
3. canonical Communications transition в `safe_for_delivery`.

Факты могут прийти в любом порядке. Runnable job появляется только после
exact anchor join, exact candidate/safety event identities и transition
`blob_admitted -> safe_for_delivery`. Safety event не содержит выдуманного
correlation field. Exact replay идемпотентен; collision, stale generation,
revoked permit или mismatched candidate fail closed.

```text
client -> Gateway -> archive engine request_rpc
provider scan candidate event -----------\
Communications safe_for_delivery event ----> owner-local durable join
                                            -> target-bound Blob custody
                                            -> one-use bounded Blob read
                                            -> ZIP metadata adapter
                                            -> owner-local result/outbox
```

Engine не вызывает Communications RPC, не читает Communications или
integration storage и не получает shared filesystem path. Cross-owner source
bytes передаются только через evidence-bound Blob custody. Kernel выдаёт и
fence-ит capability, но не читает bytes и не интерпретирует report.

### Единицы сборки

```text
hermes-attachment-archive-inspection-api
  typed Start/Get/realtime client contract and bounded report schema

hermes-attachment-archive-inspection-core
  pure limits, path normalization, entry policy and terminal decisions

hermes-attachment-archive-inspection-zip
  reviewed ZIP central-directory metadata adapter; never extracts files

hermes-attachment-archive-inspection-persistence
  owner-local request/event inbox, join, fenced jobs, result and exact outbox

hermes-attachment-archive-inspection-runtime
  managed control, request/query, Event Hub, Blob custody/read and orchestration

hermes-attachment-archive-inspection-assembly
  descriptor/settings/Storage artifacts and unsigned release fragment only
```

API/core/parser не зависят от Communications implementation, Attachment
Security implementation, integrations, Kernel, Storage, Blob implementation
или runtime packages. Persistence является единственным SQL owner surface.
Runtime не материализует release artifacts. Assembly не запускается Kernel и
не подписывает manifest.

### Bounded ZIP policy

Первый production parser поддерживает только ZIP. Он читает central-directory
metadata и не распаковывает entry bytes на диск или в память.

Hard limits принадлежат typed engine settings и не расширяются request/event
payload:

- source archive bytes;
- entry count;
- total declared uncompressed bytes;
- per-entry declared uncompressed bytes;
- normalized UTF-8 path bytes;
- path depth.

Fail-closed отклоняются:

- absolute, drive-prefixed, traversal и control-character paths;
- duplicate normalized paths;
- encrypted entries/archives;
- nested `.zip`, `.rar` и `.7z`;
- symlink и другие non-regular/non-directory Unix entry types;
- malformed ZIP metadata;
- любой limit overflow.

Отказ имеет bounded enum code. Raw parser error, original entry name, Blob
reference, source bytes, private socket/path и provider identity не попадают в
logs, health, telemetry или realtime error.

RAR/7z detection/parsing, recursive sandbox inspection, extraction и CDR не
входят в этот gate. Они требуют отдельного parser adapter и phase gate.

### Result boundary

Ready report содержит только bounded ZIP kind, counts/sizes и normalized entry
paths. Он является derived inspection evidence, а не canonical attachment
safety truth. Search index и UI projection rebuildable; source hash/generation
binding обязаны предотвращать reuse результата после смены Blob evidence.

Client получает Start receipt, а terminal state читает через Get и общий
replayable SSE status. `accepted` не означает completion.

## Phase gate `attachment_archive_inspection_v1`

Gate открывается атомарно только после:

1. exact six-unit topology и executable dependency policy;
2. reviewed exact ZIP dependency profile;
3. request/status contract без source Blob/private/provider fields;
4. bounded path/type/encryption/nested/size/count/depth policy tests;
5. owner-local request + candidate + safety-state join в любом порядке;
6. exact replay/collision and lease/generation fencing;
7. target-bound Blob custody and one-use read;
8. successful real ZIP metadata inspection without extraction;
9. traversal, duplicate, encrypted, nested, symlink/special-entry, malformed,
   entry-count/depth/per-entry/total-size negative matrix;
10. restart/NATS outage replay without second Blob transfer or parser run;
11. Gateway Start/Get and shared SSE terminal replay;
12. privacy-negative output and architecture/SRP/Cargo/full backend gates.

До выполнения всех пунктов inventory state остаётся `planned`.

## Отклонённые варианты

### Вернуть parser в Communications

Отклонено: domain получил бы content parser, Blob data-plane и archive policy.

### Добавить archive parsing в Attachment Security

Отклонено: malware verdict и archive inventory имеют разные причины изменения,
failure semantics и release cadence.

### Передать Blob reference из клиента

Отклонено: клиент не является custody/evidence authority и мог бы выбрать
произвольный source.

### Распаковать во временную директорию

Отклонено: metadata inspection не требует extraction; disk writes увеличивают
path/symlink/race surface без продуктовой необходимости.

### Считать malformed/unsupported archive clean

Отклонено: archive result не является safety verdict, а parser failure не
может повышать trust.
