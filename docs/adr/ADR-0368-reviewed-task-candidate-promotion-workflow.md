# ADR-0368: Reviewed task candidate promotion workflow

Статус: Принято

Дата: 2026-08-01

Состояние реализации: staged. Review-owned terminal promotion-result API,
pure workflow correlation core и owner-local workflow persistence реализованы
как отдельные compile-isolated units. Persistence атомарно связывает approval
inbox с Tasks command outbox и Tasks terminal-result inbox с Review result
outbox, не сохраняя candidate content, Blob proof или provider identity.
Workflow runtime, assembly, Review result consumer и managed E2E ещё не
реализованы. Наличие отдельных контрактов, persistence или прежнего managed
launch не открывает promotion gate.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0207](ADR-0207-canonical-business-domain-registry.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0366](ADR-0366-communication-task-candidate-extraction-and-reviewed-task-promotion.md).

## Контекст

ADR-0366 задаёт правильные owner boundaries: Review владеет human decision,
Tasks владеет Task truth, а взаимодействие происходит через durable events и
commands. Реализованный Review runtime публикует
`TaskCandidateApprovedForPromotionV1`, а Tasks runtime принимает только
`CreateTaskFromReviewedCandidateCommandV1`. Это два разных public contracts.

Kernel, Gateway или один из domains не может преобразовать первый контракт во
второй:

- Kernel и Gateway owner-neutral и не интерпретируют business payload;
- Review не должен импортировать Tasks implementation или выдавать target
  command от имени workflow;
- Tasks не должен импортировать Review implementation или читать Review SQL;
- прямой adapter внутри любого domain снова смешал бы decision и Task mutation.

Поэтому отсутствующий переход является отдельным cross-owner workflow, а не
helper, facade или расширением одного из domains.

## Решение

Ввести owner `reviewed_task_candidate_promotion` с role `workflow` и пять
отдельных build units:

1. `hermes-review-task-candidate-promotion-api` — Review-owned exact terminal
   promotion result contract, доступный producer workflow и consumer Review;
2. `hermes-reviewed-task-candidate-promotion-core` — pure deterministic mapping
   и correlation rules без transport, SQL, Blob и domain implementations;
3. `hermes-reviewed-task-candidate-promotion-persistence` — owner-local
   PostgreSQL inbox/outbox и durable correlation между approval, Tasks command
   и terminal result;
4. `hermes-reviewed-task-candidate-promotion-runtime` — managed event adapter;
5. `hermes-reviewed-task-candidate-promotion-assembly` — descriptor, empty
   typed Settings schema, Storage bundle и unsigned release fragment.

Review promotion contract unit принадлежит Review contract surface, но не
содержит Review core/persistence/runtime. Остальные четыре units принадлежат
workflow. Runtime может импортировать только public Review, Tasks и platform
contracts плюс собственные core/persistence; ни одна domain implementation
dependency не разрешена.

### Event flow

```text
Review approved event
  -> promotion workflow inbox
  -> deterministic Tasks command
  -> promotion workflow outbox
  -> Tasks command consumer
  -> Tasks owner-local mutation or rejection
  -> typed Tasks terminal result
  -> promotion workflow result inbox
  -> Review-owned promotion result
  -> promotion workflow outbox
  -> Review promotion-result consumer
  -> Review owner-local promotion projection + replayable SSE
```

Workflow не читает candidate Blob и не получает Blob capability. Он переносит
opaque Tasks-target-bound receipt из already authenticated Review event в exact
Tasks command без re-encode содержимого. Только Tasks получает custody и читает
candidate bytes.

### Correlation и idempotency

Workflow command identity детерминированно зависит от exact approval event
message ID, Review ID, candidate ID и decision revision. Approval inbox и Tasks
command outbox сохраняются одной транзакцией. Повтор exact approval возвращает
тот же command; reuse event ID с другим envelope hash или payload отклоняется.

Tasks terminal result принимается только если его command ID/message ID и
logical owner совпадают с сохранённой workflow correlation. Result inbox и
Review promotion-result outbox также сохраняются атомарно. Duplicate exact
result replayable; conflicting hash, unknown command или stale correlation
fail closed.

Review promotion result содержит только review/candidate IDs, expected decision
revision, bounded outcome и optional Task ID. Title, hints, source body, Blob
proof, provider/account identity и private content в нём запрещены. Review
сверяет owner, current pending promotion и expected revision до mutation.

### Runtime и admission

Workflow descriptor запрашивает шесть независимых capabilities:

- required consumer Review approved event;
- publisher Tasks create command;
- required consumer Tasks created result;
- required consumer Tasks rejected result;
- publisher Review promotion result;
- owner-local Storage namespace.

Runtime не предоставляет client RPC, realtime или Blob surface. Review client
по-прежнему видит projection только через Review query и shared replayable SSE.
Periodic polling и handwritten REST не вводятся.

## Phase gate

`reviewed_task_candidate_promotion_v1` становится implemented только после:

1. пяти exact build units и compile isolation;
2. versioned typed Review promotion-result contract;
3. atomic workflow approval inbox/Tasks outbox;
4. atomic workflow Tasks-result inbox/Review outbox;
5. Review runtime consumer и owner-local promotion transition;
6. one signed release и distinct managed workflow admission;
7. Gateway approve/reject и shared SSE E2E;
8. доказательства: до approve Task отсутствует, reject не создаёт Task,
   approve создаёт ровно один Task и Review становится `succeeded`;
9. duplicate/conflict, wrong-owner, stale revision, unknown command, restart,
   revoke, generation/grant и privacy negatives;
10. architecture, Cargo, unit, persistence, managed runtime и full pre-push
    gates.

До закрытия этого gate aggregate
`communication_task_candidate_extraction_v1` остаётся planned.

## Последствия

- Domains остаются автономными и не импортируют implementation друг друга.
- Kernel/Gateway/Event Hub не становятся business mediator или facade.
- Workflow имеет собственные release, Storage, restart и revoke boundaries.
- Terminal Tasks result становится наблюдаемым Review projection, а не
  предположением после accepted command.
- Добавляется отдельный runtime, но его responsibility и authority exact и
  проверяемы.

## Отклонённые варианты

### Review напрямую публикует Tasks command

Смешивает human decision owner и cross-owner orchestration, а Review начинает
выбирать target-domain command.

### Tasks напрямую читает Review state или SQL

Нарушает owner-local storage, event-only boundary и независимый restart.

### Kernel или Gateway преобразует payload

Создаёт generic business facade в owner-neutral control/client plane.

### Считать approve успешным созданием Task

Accepted command не является terminal result и скрывает outage/rejection между
Review и Tasks.
