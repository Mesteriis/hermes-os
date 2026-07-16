# ADR-0227: Deployment profiles и server bootstrap pairing

Статус: Принято
Дата: 2026-07-16
Состояние реализации: Не реализовано; документ и closed executable gate
задают обязательный контракт до появления platform runtime packages.

Зависит от:

- [ADR-0217: Нулевой внешний bootstrap Kernel](ADR-0217-zero-external-dependency-kernel-bootstrap.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0219: Целостность managed modules](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0225: Первый production slice и phase gates](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

## Решение

Первая release matrix состоит ровно из двух deployment profiles:

| Contract | `macos_tauri_embedded_v1` | `linux_docker_server_v1` |
|---|---|---|
| Architecture | Apple Silicon | x86_64 Linux |
| Kernel lifecycle | `managed_child` Tauri | `external_compose` |
| Artifact identity | signed native executable digest | signed immutable OCI digest |
| Process authority | Kernel starts verified native children после соответствующего gate | Compose/systemd starts containers; Kernel не посылает им signals |
| Device proof | Secure Enclave ES256 | TPM2 ES256 или FIDO2/WebAuthn ES256 |

`DeploymentProfileV1`, `RuntimeLifecycleV1`, `DistributionArtifactV1`,
`DeviceProofV1` и `InitialOwnerEnrollmentV1` являются будущими typed contracts;
эти типы не создаются в `kernel_recovery_only_v1`.

macOS initial enrollment передаётся только через inherited private FD от Tauri.
Linux server initial enrollment использует только `RemotePairingEnrollmentV1`;
обычный Gateway, public registration listener и argv/environment secret channel
для этого запрещены.

Linux Vault wrapping key всегда TPM2-sealed. FIDO2/WebAuthn используется только
для owner/device authentication и не является заменой Vault wrapping slot.
FIDO2 assertion хранит и проверяет `credential_id`, exact `clientDataJSON`,
`authenticatorData`, signature, RP ID, origin, UP/UV flags и sign counter; raw
ES256 signature не является взаимозаменяемым форматом.

### One-shot server pairing

Gate `server_bootstrap_pairing_v1` разрешает исключительно короткоживущий
pairing listener Linux server. Он требует:

- одноразовый 256-bit CSPRNG token, purpose-bound, single-use и с коротким TTL;
- console/QR descriptor, не содержащий private key или secret кроме самого
  одноразового bootstrap bearer;
- ephemeral TLS certificate и обязательный fingerprint pinning до передачи
  device proof;
- bounded rate limiting и закрытие listener после первого успешного enrollment;
- rejection replay, concurrent consume, expiry, wrong endpoint и second initial
  enrollment;
- trustworthy Control Store как единственное место consumed/revocation truth.

До открытия gate Kernel не слушает TCP и не получает Docker socket. Normal
listener configuration, TLS policy и remote endpoint сохраняются только в
trustworthy Control Store после соответствующего Gateway gate.

### Distribution и lifecycle boundary

OCI images принимаются только по immutable digest и release preflight с Cosign
verification. Kernel сверяет signed distribution identity и protocol/service
attestation, но не обращается к Docker API. Docker Compose использует private
networks, named durable volumes, resource limits, mounted service-scoped
secrets и health dependencies; `service_healthy` не заменяет independent
Kernel readiness attestation.

macOS update — explicit signed Tauri/platform artifact. Linux rollback —
explicit previously verified immutable digest. Automatic fallback, image tag
identity, PID, container name, UID и path как authority запрещены.

## Последствия

`server_bootstrap_pairing_v1` остаётся в `notAuthorized` до ADR decision
fields и executable conformance. Он не открывает `client_gateway_v1`,
`first_owner_v1`, module registration или managed launch. macOS x86_64, Linux
ARM64, Windows, Android и browser UI не входят в эту release matrix.
