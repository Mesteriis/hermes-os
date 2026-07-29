# ADR-0334: Receipt-bound idempotent Blob write retry

Статус: Принято

Дата: 2026-07-29

Состояние реализации: Blob lifecycle и data service принимают повтор exact
write только для session, содержащей полный expected plaintext SHA-256. При
несовпадении существующего content операция завершается fail-closed.

## Контекст

Managed Blob `write_new` намеренно запрещает overwrite. Для обычного write это
остаётся правильным правилом. Но target-bound delivery workflow по ADR-0333
использует deterministic reference: crash после успешной Blob записи и до
PostgreSQL commit обязан повторить те же exact bytes. Без отдельной receipt-bound
семантики retry всегда получал `AlreadyExists` и не мог завершить intent.

Нельзя трактовать любой `AlreadyExists` как успех: reference сам по себе не
доказывает, что сохранённые bytes совпадают с command receipt.

## Решение

Blob lifecycle предоставляет отдельный exact-write path:

1. session содержит полный `expected_plaintext_sha256`;
2. новые bytes проверяются против receipt до записи;
3. первая запись использует прежний create-only `write_new`;
4. при `AlreadyExists` Blob читает полный существующий объект под теми же
   owner/access/custody/key fences;
5. успех возвращается только при полном совпадении SHA-256;
6. mismatch, unreadable object или иной storage error остаётся ошибкой.

Обычный write без receipt остаётся строго create-only. Нового wire operation,
overwrite, generic upsert, cross-owner read или client-visible filesystem path
не вводится.

Custody transfer использует тот же lifecycle primitive, поэтому одинаковые
retry semantics не дублируются между transfer и managed write.

## Границы

- Blob service не интерпретирует provider/domain payload;
- source workflow не получает read capability;
- сравнение выполняется внутри Blob runtime;
- proof и current target/runtime/grant fences не ослабляются;
- hash collision рассматривается в рамках уже принятого SHA-256 receipt
  authority.
