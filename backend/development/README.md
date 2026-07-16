# Clean-room development platform

This Compose contour is only for `development_full_platform_v1`. It supplies
loopback PostgreSQL and NATS JetStream so clean-room platform packages can be
implemented and tested without release images, Cosign, TPM/FIDO2 or Docker
socket access from Kernel.

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

`hermes-development-platform-runtime` is a separate development-only workspace
binary. It does not control Docker or run inside Kernel. For an environment
where the endpoints are reachable from the native host, check that boundary
explicitly with:

```sh
cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package hermes-development-platform-runtime -- probe \
  --postgres-address 127.0.0.1:35432 \
  --nats-address 127.0.0.1:34222
```

The runtime only performs bounded TCP probes; service protocol/readiness stays
covered by `development-platform-smoke` inside Compose until Storage Control
and Event Hub are implemented.
