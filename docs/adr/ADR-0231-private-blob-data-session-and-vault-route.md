# ADR-0231: Private Blob data session и ciphertext-only Vault route

Статус: Принято
Дата: 2026-07-18
Состояние реализации: Реализовано как platform gate. `hermes-blob-service` открывает
private 0600 Unix socket из staged configuration, проверяет one-use P-256
`BlobDataSessionGrantV1` (Kernel instance, Blob/runtime/grant fences, expiry и
32-byte channel binding) и умеет выполнить bounded `write`/`read_range` только
после ciphertext-only Vault lease. Live managed conformance подтверждает
write/read через file-backed Vault, one-use replay rejection и stale Blob
runtime-generation rejection. Kernel передаёт service verification key. Первый
owner ещё не существует, поэтому owner-specific content-session issuer намеренно
не добавлен в Kernel и data socket не становится generic domain API. Это не
препятствует `blob_v1` как platform gate; owner-facing issuance появится только
вместе с exact owner capability contract под `first_owner_v1`.

Уточняет:

- [ADR-0215: открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219: integrity managed modules](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0223: Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0230: Blob Platform](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md).

## Контекст

`BlobContentLifecycleStore` может безопасно хранить encrypted bytes, но не
должен превращаться в filesystem mount для modules. Inherited managed-control
FD принадлежит Kernel supervision и не может переносить private content: это
сделало бы Kernel data proxy. Одного `BlobAccessFenceV1` недостаточно для
socket authentication: его поля не являются capability secret и same-UID
process мог бы их воспроизвести.

## Решение

Blob service открывает отдельный owner-private Unix socket, чей путь приходит
только в one-shot staged runtime configuration. Socket не является public HTTP,
не публикуется в discovery, telemetry или health и удаляется только Blob process
после удержания своего private runtime-directory lock. Regular file, symlink,
wrong owner/mode и stale socket вне controlled shutdown fail closed.

Каждый request содержит exact `BlobRefV1`, `BlobAccessFenceV1`, bounded
operation и `BlobDataSessionGrantV1`. Grant является compact binary token,
подписан current Kernel file-backed session authority и содержит:

- registration ID, runtime instance/generation и grant epoch;
- owner/capability scope, maximum bytes, expiry и random session ID;
- exact Blob service runtime generation и a 32-byte socket-channel binding.

Blob verifies signature, expiry, service generation and all fence/quota fields
before allocating, reading or deleting a byte. A revoked registration, changed
grant epoch, expired grant, replayed session ID or wrong channel binding rejects
the request before Vault routing. A grant never carries content-encryption key,
filesystem path, URL, plaintext hash or generic bearer permission.

For an admitted request Blob service itself creates a `BlobContentKeyFenceV1`
and sends its HPKE-protected `VaultCiphertextRouteV1` over the verified
inherited control FD. Kernel validates caller identity/fence and forwards only
ciphertext to Vault. The returned authority is unwrapped only by Blob service;
the caller and Kernel never receive it. Write/read/delete consequently follow:

```text
module runtime -- private Blob socket --> Blob service -- ciphertext route --> Kernel --> Vault
module runtime <-- private Blob socket -- Blob service <-- ciphertext response -- Kernel <-- Vault
```

Kernel is not on the private byte path. A single request has bounded framing,
one operation, bounded plaintext/range and a sanitized result code. The service
serializes mutations per reference; a crash before metadata commit is recovered
by the existing pending-write protocol. Revoke closes all sessions for the
affected registration/capability, destroys in-memory lease material and makes
new Vault routes fail; it never performs implicit content deletion.

`BlobDataResponseV1` always serializes an explicit result: a successful write
or read carries `accepted=true`; a rejected request carries a non-empty
sanitized error code. This preserves the protocol rule that a framed direct
socket response is never an empty Protobuf payload.

## Required implementation evidence

- service request decoder rejects malformed, oversized, expired, wrong-runtime,
  wrong-binding or replayed grants and does not serialize paths/keys/private
  content into logs; this is covered by unit regressions;
- service creates and removes only its canonical 0600 Unix socket under a 0700
  private directory, rejecting symlink, wrong mode and stale-file attacks;
- live managed-service startup and data-path conformance obtain a current Vault
  key through the inherited ciphertext route, write encrypted bytes, read a
  bounded range, and reject replayed or stale-runtime sessions;
- runtime/grant expiry and revoke invalidate an active session and a cached
  lease; a subsequent request cannot read or write;
- interrupted write, socket restart and concurrent same-reference requests
  preserve the existing metadata/quota recovery invariants.

This ADR opens `blob_v1` only as a platform gate. Owner-local canonical metadata
and the owner-facing session-issuance contract remain deferred to
`first_owner_v1`; encrypted component classification is present, while full
instance restore remains exclusively `whole_instance_backup_v1` work.
