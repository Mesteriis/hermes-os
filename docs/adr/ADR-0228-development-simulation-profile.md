# ADR-0228: Development simulation profile

Статус: Принято
Дата: 2026-07-16
Состояние реализации: executable policy фиксирует full-platform development
contract; отдельный `hermes-development-platform-runtime` выполняет bounded
native TCP probe явно переданных development PostgreSQL/NATS endpoints, а
Compose contour даёт real service smoke внутри private network. Runtime
adapters и services ещё реализуются последовательно по milestones; production
platform adapters и их gates остаются закрыты.

Зависит от:

- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0225: Первый production slice и phase gates](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md);
- [ADR-0227: Deployment profiles и server bootstrap pairing](ADR-0227-deployment-profiles-and-server-bootstrap-pairing.md).

## Контекст

Разработка не должна зависеть от наличия Apple Developer account, Secure
Enclave signing entitlement, TPM2, FIDO2 hardware, OCI registry или release
signing environment. В то же время их отсутствие не должно тихо превращать
software key в production device identity либо открывать remote/server surface.

## Решение

Добавляется единственный непроизводственный contract
`development_full_platform_v1`. Он выбирается только явным development
invocation и предназначен для полного local platform development: Kernel,
Gateway, Vault, Storage Control, NATS, Blob, Clock, Scheduler, client flows и
remote-pairing protocol могут работать с development adapters и local services.

В этом profile разрешён software ES256 signer c development-local filesystem
storage, persistent development data/credentials, network listeners, remote
pairing, Vault и external services. Он может симулировать оба release target:
`macos_tauri_embedded_v1` и `linux_docker_server_v1`. Каждый запуск обязан
явно показывать insecure warning.

Development profile не является третьим deployment profile и не изменяет
release matrix ADR-0227. Он не выпускает release artifacts и не позволяет
использовать результаты simulation как production gate evidence. В нём должны
использоваться только development credentials и data: реальные owner keys,
production provider credentials и private user content не копируются в этот
контур.

Runtime adapter обязан fail closed вне explicit development invocation. Он не
может быть активирован Tauri release bundle или production Compose/systemd
deployment и не является automatic downgrade после platform signer failure.

Первый development runtime — отдельный Cargo package с role `development`,
owner `development`, surface `runtime`; это не Kernel component и не получает
Docker socket. Его единственная текущая операция — bounded TCP reachability
probe для явно заданных PostgreSQL и NATS addresses. Compose service health
проверяется отдельно из Compose network; эти checks не являются Storage Control
или Event Hub readiness attestation.

Development Control Store также хранит `ExternalRuntimeAttestation`: она
привязана к approved module registration, exact distribution SHA-256, runtime
generation и текущему grant epoch. Повтор или откат generation отклоняется;
approval, suspension или revoke делают прежнюю attestation ineffective. Это
не Docker identity и не production service attestation: development CLI лишь
принимает явно переданный simulated evidence и никогда не сигналит контейнеры.

## Последствия

Разработчик может строить и тестировать весь local platform flow без hardware.
После появления реального platform adapter те же protocol vectors должны
выполняться через Secure Enclave/TPM2/FIDO2; simulated evidence не заменяет
hardware/release conformance.
