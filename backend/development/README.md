# Clean-room development platform

This Compose contour is only for `development_full_platform_v1`. It supplies
loopback PostgreSQL and NATS JetStream so clean-room platform packages can be
implemented and tested without release images, Cosign or Docker socket access
from Kernel.

It intentionally uses PostgreSQL `trust` authentication and unauthenticated
NATS. The named volumes, endpoints and data are development-only and must not
be migrated into a release deployment or populated with production credentials
or private user data.

```sh
make -C backend development-platform-up
make -C backend development-platform-status
make -C backend development-platform-down
```

Normally the loopback endpoints are
`postgres://hermes_development@127.0.0.1:35432/hermes_development` and
`nats://127.0.0.1:34222`. Some Docker Desktop/OrbStack configurations do not
publish fixed host ports even when Compose accepts them; use
`make -C backend development-platform-endpoints` to print the live native-host
addresses in that case. These addresses are not a future production
configuration surface.

Run `make -C backend development-platform-smoke` after startup. It executes a
real `SELECT 1` and NATS health request inside the Compose network, so it also
works where host port forwarding is unavailable.

`hermes-development-kernel-operator` is a separate development-only workspace
binary. It does not control Docker or run inside Kernel. For an environment
where the endpoints are reachable from the native host, check that boundary
explicitly with:

```sh
cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package hermes-development-kernel-operator -- probe \
  --postgres-address 127.0.0.1:35432 \
  --nats-address 127.0.0.1:34222
```

The runtime only performs bounded TCP probes; service protocol/readiness stays
covered by `development-platform-smoke` inside Compose until Storage Control
and Event Hub are implemented.

It also has a development-only remote-pairing simulation. It is not evidence
for `server_bootstrap_pairing_v1` or a production Kernel initial enrollment:

```sh
cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package hermes-development-kernel-operator -- pairing create \
  --state-dir /absolute/private/development-pairing --ttl-seconds 300

cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package hermes-development-kernel-operator -- pairing consume \
  --state-dir /absolute/private/development-pairing --token YOUR_256_BIT_HEX_TOKEN
```

After `create`, start a bounded TLS listener. It prints an endpoint and the
ephemeral certificate SHA-256 fingerprint; the client must compare that
fingerprint before it sends the bearer token or device proof:

```sh
cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package hermes-development-kernel-operator -- pairing listen \
  --state-dir /absolute/private/development-pairing \
  --listen-address 127.0.0.1:0 --idle-timeout-seconds 300
```

The listener exposes a one-request challenge and enrollment protocol over TLS:
`GET /v1/pairing-challenge` with `Authorization: Bearer TOKEN`, then
`POST /v1/initial-owner-enrollment` with the same bearer and owner/device,
public-key and raw-signature headers. The helper below creates or opens the
owner-private `0600` ES256 file and prints only the public key and signature:

```sh
cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package hermes-development-kernel-operator -- pairing proof \
  --key-dir /absolute/private/development-device-key \
  --challenge CHALLENGE_HEX --owner-id owner_1 --device-id device_1
```

The persisted state contains a SHA-256 token digest, expiry, terminal state and
the receipt digest; the owner-private receipt itself contains public proof
material, never the bearer token. A consumed pairing blocks any second initial
enrollment. The listener never accesses Kernel SQLite directly. To apply the
receipt, Kernel independently checks its state hash and ES256 proof before it
writes the initial owner into Control Store:

```sh
cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package hermes-kernel -- --development-profile \
  --data-dir /absolute/private/kernel-data \
  initial-owner-import-pairing \
  --pairing-state-dir /absolute/private/development-pairing
```

This explicit development-only handoff is not the production listener contract
and cannot open `server_bootstrap_pairing_v1`. Never use this development state
directory, key or token with production data.

For an already approved development module, an explicit owner-pinned artifact
record can be created and rechecked without spawning a process:

```sh
cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package hermes-kernel -- --development-profile \
  --data-dir /absolute/private/kernel-data \
  module-owner-pin-artifact \
  --registration-id REGISTRATION_ID \
  --artifact /absolute/path/to/local-artifact

cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package hermes-kernel -- --development-profile \
  --data-dir /absolute/private/kernel-data \
  module-owner-pinned-preflight \
  --registration-id REGISTRATION_ID
```

The record is bound to the canonical path, SHA-256, size, observed file
identity and a file-signer ES256 proof. Changed bytes require a new explicit
revision. It is development evidence only: it does not spawn, supervise or
authorize a managed runtime and does not open `managed_launch_trust_v1`.
