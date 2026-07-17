# ADR-0227: Deployment profiles и server bootstrap pairing

Статус: Принято
Дата: 2026-07-16
Состояние реализации: `server_bootstrap_pairing_v1` открыт. Kernel реализует
one-shot TLS listener с file-backed ES256 proof, коротким TTL, bounded failure
rate, certificate fingerprint descriptor и atomic single-use claim в private
Control Store. Normal Gateway, managed launch и all later platform gates всё
ещё закрыты.

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
| Device proof | File-backed ES256 adapter | File-backed ES256 adapter |

`DeploymentProfileV1`, `RuntimeLifecycleV1`, `DistributionArtifactV1`,
`DeviceProofV1` и `InitialOwnerEnrollmentTransportV1` являются typed
contracts. Их существование в protocol package не открывает route, process или
feature в `kernel_recovery_only_v1`.

macOS initial enrollment передаётся только через inherited private FD от Tauri.
Linux server initial enrollment использует только `RemotePairingEnrollmentV1`;
обычный Gateway, public registration listener и argv/environment secret channel
для этого запрещены.

Vault wrapping key использует отдельный file-backed wrapping-key adapter в
owner-private service-scoped storage. Device signer и Vault wrapping key
остаются разными key slots; компрометация одного не является основанием
смешивать их authority.

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

OCI images фиксируются immutable digest. Cosign release preflight может
дополнительно проверить digest и publisher identity, но является optional
release hardening, а не authority для открытого file-backed managed-launch
gate. Пока нет отдельного file-signed OCI binding, Kernel сверяет только
protocol/service attestation external runtime и не обращается к Docker API.
Docker Compose использует private networks, named durable volumes, resource
limits, mounted service-scoped secrets и health dependencies;
`service_healthy` не заменяет independent Kernel readiness attestation.

macOS update — explicit signed Tauri/platform artifact. Linux rollback —
explicit previously verified immutable digest. Automatic fallback, image tag
identity, PID, container name, UID и path как authority запрещены.

## Последствия

`server_bootstrap_pairing_v1` открыт после ADR decision fields и executable
conformance: wrong token получает rejection, valid P-256 proof создаёт ровно
одного owner и atomically marks pairing consumed; повторный bootstrap rejected.
Gate не открывает `client_gateway_v1`, `first_owner_v1`, normal public
listener, managed launch или Docker API authority. macOS x86_64, Linux ARM64,
Windows, Android и browser UI не входят в эту release matrix.
