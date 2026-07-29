# ADR-0327: Durable target-bound Blob delegation across source successors

- Статус: принято
- Дата: 2026-07-29
- Состояние реализации: реализовано и подтверждено live generation 95
- Уточняет: ADR-0257, ADR-0274, ADR-0275

## Контекст

Live Mail materialization выявил разрыв между durable event delivery и source
runtime fencing:

- Mail успешно записал attachment в собственную Blob custody;
- Kernel выдал signed target-bound proof для Attachment Security;
- candidate и proof были durable сохранены в owner outbox/inbox;
- до обработки candidate Mail process получил approved successor generation;
- source registration и exact Blob grant epoch остались текущими;
- Kernel отклонил transfer только потому, что issuance runtime generation уже
  не совпадала с текущим process generation.

В наблюдаемом contour восемь pending scan jobs имели один ещё не истёкший proof
от source generation 67, текущая approved Mail generation была 69, а exact
grant epoch оставался 10. Jobs исчерпали retry budget без вызова scanner.

ADR-0275 одновременно требует, чтобы target мог быть offline при source write,
и чтобы source runtime session оставалась current при позднем transfer. Эти
условия несовместимы с durable outbox/event delivery: benign restart source
process превращает уже выданную exact delegation в permanent retry.

Удалять source authority полностью нельзя. Revoked registration, изменённый
grant epoch, другой owner/capability/custody scope или неподписанный proof
должны по-прежнему fail-close.

## Решение

### Issuance fence и durable authority

`BlobCustodySourceProofV1` сохраняет без изменения:

- source owner, registration и capability;
- source runtime instance и generation как provenance момента issuance;
- source grant epoch;
- reference, size, digest, custody scope, TTL и Kernel signature;
- exact public target owner/module/capability.

Proof является bounded durable delegation, выданной текущему source process,
а не lease на продолжительность этого OS-процесса. Runtime provenance
подписывается и сохраняется для audit, но approved successor generation того
же source registration не инвалидирует уже выданную target-bound delegation.

### Transfer-time source validation

При transfer Kernel обязан проверить:

1. Kernel signature, instance, version, TTL и exact source receipt fields;
2. exact public target owner/module/capability из proof;
3. наличие текущего approved source registration с тем же opaque
   `registration_id`;
4. exact current source capability, owner, custody scope и `grant_epoch`;
5. current source grant всё ещё разрешает Blob write;
6. current target registration/runtime instance/generation/grant epoch и exact
   target Blob capability;
7. evidence ID/envelope binding и Blob transfer grant как раньше.

Kernel не требует, чтобы current source runtime instance/generation совпадали
с issuance provenance. Source process не участвует в transfer data plane и не
получает новый credential.

Если source registration заменён, suspended/revoked, capability удалена,
grant epoch изменён или proof истёк, catalog match отсутствует и transfer
остаётся denied. Новый registration не наследует authority старого proof.

### Replay и boundedness

Target binding не становится generic bearer authority:

- transfer может запросить только exact currently fenced target runtime;
- proof нельзя перенаправить другому owner/module/capability;
- source reference/size/digest и current source grant остаются exact;
- deterministic target reference и Blob idempotency сохраняются;
- plaintext, provider identity и storage path не проходят через Kernel или
  event plane.

Benign source successor не меняет target identity и не расширяет права. Он
только перестаёт уничтожать уже опубликованную durable работу.

### Одноразовое восстановление исчерпанной custody-очереди

Уже terminal jobs не должны оставаться потерянными только потому, что прежняя
runtime-session проверка исчерпала их bounded retry budget до исправления.
Attachment Security schema bundle revision 4 содержит две additive-миграции:
revision 3 добавляет `retry_policy_revision` с revision 1 для существующих
строк, revision 4 добавляет owner-local partial recovery index. После Storage
admission owner-local persistence reconciliation один раз возвращает в pending
только строки, для которых:

- `state = 3`;
- отсутствуют target Blob reference и receipt;
- отсутствует verdict outbox message;
- действует прежняя `retry_policy_revision = 1`.

Reconciliation сохраняет original candidate, signed source proof, evidence,
causation и correlation, снимает lease, обнуляет attempt counter и отмечает
job как `retry_policy_revision = 2`. Она не переоткрывает scanner failures:
успешный custody transfer уже оставляет target receipt. Новые jobs сразу
явно создаются с policy revision 2. Storage migration остаётся owner-local
additive DDL, а bounded data transition выполняет только Attachment Security
persistence под собственной runtime role. Recovery не является постоянным
generic requeue-механизмом и не обходит max attempts.

## Ownership и единицы сборки

- integration владеет provider attachment, source Blob write и event outbox;
- public Attachment Security contract владеет target custody audience;
- Blob Platform владеет proof verification и internal rewrap;
- Kernel проверяет current registrations/grants и target runtime fence, но не
  интерпретирует candidate payload;
- Attachment Security engine владеет durable retry/scan/verdict lifecycle;
- Mail и Communications не импортируют engine implementation или Blob store.

Новая domain-to-domain, integration-to-domain RPC или assembly authority не
вводится.

## Admission evidence

1. unit/architecture guard сохраняет current source registration/grant checks,
   но запрещает exact current source runtime-session check в custody transfer;
2. benign source generation successor при неизменном registration/grant epoch
   завершает target-bound transfer и scan;
3. replaced/revoked source registration и изменённый grant epoch остаются
   denied без scanner/verdict;
4. wrong target, expired/altered proof и Blob/Vault outage остаются
   fail-closed;
5. live Mail backlog переходит из pending custody в target receipt/verdict без
   прямого source read;
6. additive schema migration и owner-local reconciliation переоткрывают только
   прежние terminal jobs без target receipt/verdict и не затрагивают scanner
   failures;
7. `make pre-commit` проходит для слайса, полный `make pre-push` остаётся
   integration gate полного переноса.

Live generation 95 завершил все восемь восстановленных jobs: каждая строка
получила target Blob receipt и exact verdict outbox, все восемь verdict events
были опубликованы без повторного чтения source integration.

## Последствия

Durable provider event больше не зависит от времени жизни source OS-процесса.
Source revoke и grant rotation сохраняют authority над ещё не исполненными
delegations. Runtime generation остаётся audit provenance, но не является
скрытым synchronous lease между independently restartable integration и
engine. Уже исчерпанная очередь восстанавливается additive schema revision и
idempotent owner-local persistence reconciliation, а не ручным SQL или
межвладельческим facade.
