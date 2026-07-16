# Linux Docker release manifest

`verify-linux-release-manifest.mjs` accepts only a release manifest for
`linux_docker_server_v1`. Every platform service image must be an immutable
`repository@sha256:<digest>` reference and is verified individually with
Cosign before Compose/systemd is allowed to consume it.

The manifest must declare `service_contract: "hermes_platform_service_v1"`.
This is an image contract, not an assumption about upstream images: every
Hermes-built platform image, including the PostgreSQL and NATS derivatives,
must provide `/usr/local/bin/hermes-platform-healthcheck` and keep its durable
state at `/var/lib/hermes`. A release fails before Compose generation if it
does not opt into that exact contract. Kernel protocol attestation remains
independent from Compose health.

The manifest deliberately has no image-tag fallback, no Docker socket setting,
and no default image values. A release supplies the manifest out of band after
the platform artifacts and their signer identity exist.

`make -C backend render-linux-release-compose MANIFEST=/absolute/release.json
COMPOSE=/absolute/hermes.compose.yaml` verifies every image first, then writes
the Compose descriptor atomically. The generated descriptor uses only a private
network, named durable volumes, bounded process/memory limits, health
dependencies and service-scoped Docker secrets. It does not grant Kernel Docker
API access; Compose health is not Kernel protocol attestation.

The release operator places the PostgreSQL bootstrap secret in the sibling
`secrets/postgres_bootstrap_password` file with restrictive filesystem
permissions. The secret bytes are mounted to `/run/secrets`; the sole
environment value is the PostgreSQL-supported file *path*, never the secret.

`make -C backend render-linux-systemd-unit COMPOSE=/absolute/hermes.compose.yaml
SYSTEMD_UNIT=/absolute/hermes-platform.service` generates the `Type=oneshot`
unit used by systemd to own Compose start/stop. It deliberately has no restart
directive and does not delete named volumes on stop. Updates and rollbacks are
explicit operations using a separately Cosign-verified digest manifest.
