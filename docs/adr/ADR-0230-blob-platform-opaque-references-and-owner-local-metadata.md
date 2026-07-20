# ADR-0230: Blob Platform — opaque references and owner-local metadata

Статус: Принято
Дата: 2026-07-17
Состояние реализации: protocol, encrypted filesystem storage и Kernel Blob
quota-control foundations реализованы: opaque reference/range/fence contracts,
ephemeral Vault-response key lease, authenticated atomic files, atomic
descriptor-declared quota persistence, grant-aware catalog и ciphertext-only
Blob-to-Kernel Vault route adapter существуют. Отдельный `hermes-blob-service`
запускается только из verified release binding, получает one-shot private
configuration и attests its runtime/Vault generations over inherited Kernel FD.
Signed release binding и start разрешены только через current owner-control session;
content route через Kernel этим не создаётся.
Filesystem foundation rejects a reference at its declared expiry before any read/write
and supports only a current-fence/key-authorized atomic delete with directory `fsync`.
Its private technical ledger reserves aggregate bytes before content persistence,
records `pending_write` / `active` / fenced `delete_reserved` states, survives a
reopen, and removes uncommitted ciphertext together with its pending reservation
before accepting a retry. It removes only an uncommitted private staged ledger file
after a crash. It finalizes an owner-marked deletion only after a grace period and a
final matching fence. Its scheduled technical collector enumerates only due
owner-marked reservations and requires a newly resolved current deletion lease;
a revoked or unavailable lease leaves ciphertext and the reservation intact.
This is technical accounting, not owner metadata retention or an owner-liveness
decision.
Отдельный conformance test собирает production `hermes-blob-service` и file-backed
Vault runtime, помещает оба в signed release bundle, запускает через Kernel и
сверяет generation-bound status. Live data-path test дополнительно выпускает
test-only P-256 signed grants, записывает и читает encrypted bytes через private
Blob socket и ciphertext-only Kernel-to-Vault relay, а replayed и stale-runtime
grants получают sanitized denial. Узкий protocol fixture остаётся отдельной
negative-path проверкой. Вместе с full managed-service, live Vault data-path,
replay/stale-generation и encrypted-storage conformance это открывает
`blob_v1` как platform gate. Первый owner не создан: его canonical metadata,
owner-specific issuance of content sessions и whole-instance restore остаются
отдельной работой под `first_owner_v1` и `whole_instance_backup_v1`.

## Контекст

Durable envelopes, telemetry and client transport не могут переносить private
document/media bytes. Hermes нужен локальный Blob Platform, но общий filesystem
path, content hash или provider object key не должны становиться client/module
capability. Blob storage также не должен становиться второй business database.

## Решение

Blob Platform является Kernel-supervised platform service после `vault_v1` и
`managed_launch_trust_v1`. Он хранит encrypted content, а owner хранит
canonical metadata, lifecycle decision и reference semantics в собственной
storage boundary. Kernel не читает private bytes и не принимает business
решения по Blob content.

Public `BlobRefV1` является opaque random reference. Он содержит только stable
reference ID, authorized owner scope, size classification, expiry/revocation
fence и backup classification. В нём запрещены filesystem path, URL, provider
object key, plaintext content hash, filename, MIME description, key material и
capability token.

Каждая read/write/delete операция требует:

- current registration/capability grant и runtime generation;
- owner scope, reference ID и grant epoch;
- Vault-issued content-encryption/key-wrap authority;
- bounded declared quota и request/range limits.

Blob runtime normalizes and contains all disk paths internally. Caller никогда
не задаёт relative path, symlink target или arbitrary range. `start <= end`,
`end <= declared_size`, bounded maximum range и no implicit full-file fallback
являются mandatory для range read.

Content encryption uses per-blob data encryption keys. Key-wrap material и
unwrapped keys остаются за Vault boundary; Blob process получает только
scoped/revocable authority. Revoke, expiry, owner suspension, grant epoch change
или runtime generation change делают existing read/write lease unusable.

Retention/GC имеет two-phase model: owner marks a reference eligible only after
its canonical metadata transition; Blob Platform records a fenced deletion
reservation and deletes encrypted bytes only after grace period and a final
current-fence check. GC never follows symlinks, never escapes its canonical
data root and never infers liveness from access time alone.

Backup classifies encrypted bytes and reference manifests separately. Component
export is not whole-instance backup; whole-instance restore rotates generations
and epochs according to the backup gate.

## Запрещено

- paths, URLs, plaintext bytes or private metadata in events, telemetry, health
  or errors;
- shared module filesystem mount as a Blob API;
- software fallback key outside the current file-backed Vault provider;
- automatic retention deletion without owner metadata transition and fence;
- Blob runtime as generic provider session store or business query service.

## Evidence for `blob_v1`

The gate requires protocol/package topology, opaque-reference validation,
encrypted storage/quota enforcement, range/path/symlink negative tests,
retention/GC/revoke conformance and backup classification evidence for both
deployment profiles.
