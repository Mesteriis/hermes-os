# ADR-0343: Capability-routed Blob custody release

Статус: Принято

Дата: 2026-07-30

Состояние реализации: реализовано. Typed runtime protocol, отдельная
descriptor operation, Kernel-staged 24-hour grace policy, Kernel issuance и
current-binding admission, correlated managed-control Blob client,
Kernel-signature verification, crash-safe Blob-owned idempotency ledger и exact
deletion reservation реализованы. Delayed-delivery terminal transition
атомарно создаёт owner-local durable cleanup job; Blob failure сохраняет
bounded retry state и не откатывает business outcome. Managed runtime
маршрутизирует exact accepted/rejected/cancelled reason через current
capability/grant fence, а успешный ответ Blob завершает job. Disposable
PostgreSQL conformance доказывает retry после нового connection, а live managed
contour — committed deletion reservation для delivery acceptance и
cancellation. Gate `communication_delayed_delivery_v1` открыт как
`implemented`.

Уточняет:

- [ADR-0215](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0223](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0257](ADR-0257-event-backed-blob-custody-transfer-for-canonical-evidence.md);
- [ADR-0327](ADR-0327-durable-target-bound-blob-delegation-across-source-successors.md);
- [ADR-0334](ADR-0334-receipt-bound-idempotent-blob-write-retry.md);
- [ADR-0341](ADR-0341-scheduled-communication-delivery-workflow.md).

## Контекст

Blob является единственным владельцем encrypted bytes, metadata ledger,
custody scopes, grace-period deletion и orphan reconciliation. Managed module
может получить scoped write/read/custody-transfer session, но текущий
`BlobDataOperationV1` не имеет операции освобождения custody.

Scheduled delivery обязана удалить private body после terminal cancellation,
rejection или delivery-intent acceptance. Прямое удаление файла, доступ к Blob
metadata либо hidden filesystem API из workflow нарушают process, storage и
capability boundaries. Бессрочное хранение после terminal outcome нарушает
data-minimization invariant.

## Решение

Runtime protocol получает отдельную control-plane операцию
`ReleaseBlobCustodyV1`. Это не data-plane `delete` и не выдача filesystem
доступа.

Запрос содержит только:

- exact non-zero `operation_id`;
- exact `reference_id`;
- declared size и receipt SHA-256;
- exact target custody owner/module/capability;
- custody source proof;
- bounded reason code;
- current runtime generation, session и grant epoch из authenticated managed
  control channel, а не из module payload.

Blob service проверяет:

1. current managed binding и exact release capability;
2. caller является текущим target custody owner;
3. reference, size, digest и proof совпадают с ledger;
4. custody не была передана другому target;
5. operation ID либо новый, либо byte-identical replay.

После acceptance Blob атомарно сохраняет idempotency receipt и создаёт
deletion reservation. Физическое удаление выполняет только Blob lifecycle
после policy grace period. Повторный exact request возвращает existing receipt;
другой payload с тем же operation ID fail closed.

Grace policy является Kernel-staged platform configuration, а не module setting:
production Kernel передаёт 24 часа, runtime protocol принимает только
положительное значение не более семи суток. `delete_not_before` вычисляется от
подписанного `issued_at`, поэтому retry одного signed grant не двигает retention
window.

Ответ содержит только sanitized outcome:

```text
accepted | existing | already_released | denied | unavailable
```

Он не содержит paths, encryption metadata, Vault lease, raw proof или private
content.

## Границы

- Kernel маршрутизирует opaque typed control request и проверяет grant; он не
  интерпретирует custody semantics.
- Blob service/runtime являются единственной authority release и deletion.
- Workflow хранит только release operation ID и sanitized durable retry state.
- Integration или domain не получает Blob implementation dependency.
- Release failure не откатывает уже committed business terminal outcome.
- Cleanup retry выполняется durable job/outbox, а не in-process timer.
- Subjects, logs, health, SSE и client status не содержат reference/proof.

## Units и SRP

```text
runtime protocol
  typed release request/response

Blob client
  correlated managed-control adapter

Blob service/runtime
  admission, idempotency ledger and deletion reservation

owner workflow runtime
  terminal cleanup orchestration only
```

## Conformance gate

Admission требует:

1. generated protocol validation и hard bounds;
2. exact current-binding/grant checks;
3. byte-identical replay и conflicting replay negatives;
4. foreign custody, stale generation/epoch и revoked grant negatives;
5. crash between receipt and reservation recovery;
6. grace-period deletion и missing-file reconciliation;
7. no paths, proofs, secrets or content in diagnostics;
8. architecture, Cargo, SRP, Clippy and full tests.

Эти условия выполнены. Protocol/Kernel/Blob conformance покрывает hard bounds,
current binding, replay/conflict, foreign/stale/revoked access, crash recovery,
grace deletion и sanitized diagnostics. Delayed-delivery conformance отдельно
покрывает durable retry после reconnect и настоящий managed release route для
accepted/cancelled terminal outcomes.

## Отклонённые варианты

### Data-plane delete session

Смешивает byte transport с authority mutation и позволяет caller выбирать
физическое удаление.

### Workflow удаляет файл

Обходит encrypted store, metadata ledger, grace policy и recovery.

### Автоматическое удаление сразу после read

Ломает ambiguous retry до durable delivery-intent acceptance.

### Бессрочный orphan cleanup

Не даёт bounded data-minimization guarantee и скрывает terminal cleanup loss.
