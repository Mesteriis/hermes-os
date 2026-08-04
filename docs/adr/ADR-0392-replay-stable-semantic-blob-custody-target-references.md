# ADR-0392: Replay-stable semantic Blob custody target references

Статус: Принято

Дата: 2026-08-04

Состояние реализации: реализовано. Semantic target reference regression и
managed Speech-to-Text/Whisper real-audio restart/replay conformance проходят;
Kernel по-прежнему не получает Blob bytes и не делегирует proof-only переходы.

Уточняет:

- [ADR-0230: Blob platform opaque references](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0257: Event-backed Blob custody transfer](ADR-0257-event-backed-blob-custody-transfer-for-canonical-evidence.md);
- [ADR-0390: Call recording custody and Speech-to-Text boundary](ADR-0390-call-recording-custody-and-speech-to-text-boundary.md);
- [ADR-0391: Whisper STT provider integration](ADR-0391-whisper-stt-provider-integration.md).

## Контекст

Original-write custody proof является короткоживущей подписанной authority.
Повторная выдача proof для того же Blob после runtime restart сохраняет source
reference, receipt, owner, capability, custody scope и target, но меняет
issued/expiry time, runtime fence и signature bytes.

Прежний derivation target reference хешировал exact encoded proof. Поэтому
безопасный refresh одной authority создавал другой target reference. Owner-local
runtime не мог одновременно:

- не сохранять custody proof в PostgreSQL;
- получить fresh proof после restart;
- воспроизвести один и тот же idempotent custody transfer;
- сохранить immutable terminal Blob reference.

Хранить proof ради replay запрещено privacy boundary. Подменять transfer прямым
cross-owner read также запрещено: encrypted Blob связан с owner/custody AAD и
обязан пройти через Blob runtime re-encryption.

## Решение

Target reference для original-write transfer выводится из versioned canonical
semantic binding, а не из volatile encoded proof bytes. Binding включает:

- proof kind и Kernel instance;
- source owner, registration, capability и custody scope;
- source reference, declared size, receipt SHA-256, key revision и backup class;
- exact target owner/module/capability из подписанного proof;
- predecessor evidence ID и envelope SHA-256.

Binding намеренно не включает issued/expiry time, runtime
instance/generation/grant epoch и signature bytes. Эти поля проверяются при
каждом запросе как live authority fences, но их refresh не меняет identity
одних и тех же content/evidence/owner/target semantics.

Любое изменение source reference, receipt, size, source owner/capability/scope,
target или evidence создаёт другой target reference. Kernel по-прежнему
проверяет exact signature, current registration/grant/runtime fences и Blob
catalog operations до выдачи transfer grant.

Current-custodian redelegation сохраняет существующий stable v2 derivation.
Delegation разрешена только для reference, который получен exact custody
transfer из predecessor proof и того же evidence binding. Proof-only direct
cross-owner read не вводится.

## Границы ответственности

- Kernel проверяет authority и детерминированно выводит opaque target reference.
- Blob runtime выполняет source read, receipt verification и target-owner
  re-encryption; Kernel не видит bytes.
- Owner/runtime сохраняет только safe terminal reference/hash/size, но не proof.
- Domain, workflow и integration не импортируют implementation друг друга и не
  вычисляют target reference самостоятельно.

## Проверка

1. Unit regression доказывает одинаковый target reference для semantic-equivalent
   refreshed original-write proofs.
2. Изменение receipt, source/target authority или evidence меняет reference.
3. Delegation отклоняет не materialized current reference.
4. Managed Engine → provider → Engine → caller contour выполняет real Blob
   transfers, restart и replay без direct read или proof persistence.
5. Полный pre-push подтверждает отсутствие compatibility regressions в других
   Blob consumers.

## Последствия

Первый transfer, созданный старым raw-proof derivation, и semantic v3 reference
не совпадают. Clean-room runtime обязан пройти новый transfer и атомарно
зафиксировать его safe metadata; автоматический alias или fallback на старый
reference запрещён. После перехода proof refresh и runtime restart не создают
новую artifact identity при неизменных content/evidence/authority semantics.
