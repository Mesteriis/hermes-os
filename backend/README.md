# Hermes backend clean room

The production backend rewrite starts here. The virtual workspace contains the
six Kernel foundation packages, the five-package `vault_v1` and two-package
`clock_v1` inventory accepted by executable policy. With a trustworthy Control Store it
reaches the private module control plane; without one it fails closed to
`recovery_only`. `vault_v1` is open for the file-backed baseline: the current
Vault packages provide the file key, SQLCipher store, isolated runtime, HPKE
lease-to-scope delivery and classified component recovery. `clock_v1` provides
UTC/monotonic contracts and a deterministic fake, not a Scheduler. There are
no normal public listeners or business routes.

The previous implementation is available at
`references/backend-legacy/`, but it is not a dependency and is not an
architectural template.

## Entry conditions for new code

The following questions remain mandatory for later product slices, while
ADR-0225 closes them only for the recovery-only Kernel slice:

1. the product capabilities that are actually supported;
2. canonical evidence and event invariants that must remain stable;
3. bounded-context ownership and dependency direction;
4. the frontend-to-backend contract and cutover rule;
5. runtime lifecycle, shutdown and provider isolation;
6. schema ownership and the fresh-database bootstrap;
7. secret ownership, Vault recovery and scoped credential lease boundaries;
8. test boundaries that do not compile production composition.

The active foundation decisions are:

- [module ownership and runtime isolation](../docs/adr/ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [core/module IPC and NATS communication](../docs/adr/ADR-0201-core-module-communication-and-nats.md);
- [PostgreSQL ownership, roles/grants and PgBouncer](../docs/adr/ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md).
- [managed infrastructure supervision and recovery](../docs/adr/ADR-0203-managed-infrastructure-supervision-and-recovery.md).
- [bundled integration plugins and the provider-neutral context boundary](../docs/adr/ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md).
- [Core Gateway and desktop/Android client transport](../docs/adr/ADR-0205-core-gateway-and-client-transport.md).
- [Kernel constitution and boot/recovery state machine](../docs/adr/ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md).
- [canonical business domain registry](../docs/adr/ADR-0207-canonical-business-domain-registry.md).
- [domain development allowlist and projection freeze](../docs/adr/ADR-0208-domain-development-allowlist-and-projection-freeze.md).
- [Kernel Event Hub](../docs/adr/ADR-0209-kernel-event-hub-and-subscription-control-plane.md).
- [Telemetry Hub and local diagnostics](../docs/adr/ADR-0210-telemetry-hub-and-local-diagnostics.md).
- [backend workspace and source layout](../docs/adr/ADR-0211-backend-workspace-and-source-layout.md).
- [Cargo package topology and compile isolation](../docs/adr/ADR-0212-crate-topology-and-compile-isolation.md).
- [code ownership and module autonomy](../docs/adr/ADR-0213-code-ownership-and-module-autonomy.md).
- [durable Job Platform, Scheduler and runtime reconfiguration](../docs/adr/ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md).
- [open module registration and capability grants](../docs/adr/ADR-0215-open-module-registration-and-capability-grants.md).
- [private SQLite Kernel Control Store](../docs/adr/ADR-0216-private-kernel-control-store-with-sqlite.md).
- [zero external dependency Kernel bootstrap](../docs/adr/ADR-0217-zero-external-dependency-kernel-bootstrap.md).
- [owner/device identity, enrollment and offline recovery](../docs/adr/ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md).
- [managed module integrity, distribution manifest and explicit updates](../docs/adr/ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md).
- [canonical durable envelope and contract evolution](../docs/adr/ADR-0220-canonical-durable-envelope-and-contract-evolution.md).
- [ModuleDescriptorV1 and capability-level lifecycle](../docs/adr/ADR-0221-module-descriptor-and-capability-lifecycle-contract.md).
- [Kernel Settings Registry and supervised reconfiguration](../docs/adr/ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md).
- [encrypted SQLite Vault and scoped credential leases](../docs/adr/ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md).
- [Storage Control Plane, owner-scoped PostgreSQL and migration lifecycle](../docs/adr/ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md).
- [first production recovery-only Kernel slice and phase gates](../docs/adr/ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).
- [AI context acquisition through use-case workflows](../docs/adr/ADR-0226-ai-context-acquisition-through-use-case-workflows.md).
- [deployment profiles and server bootstrap pairing](../docs/adr/ADR-0227-deployment-profiles-and-server-bootstrap-pairing.md).
- [full-platform development simulation profile](../docs/adr/ADR-0228-development-simulation-profile.md).
- [platform Clock contract and deterministic conformance](../docs/adr/ADR-0229-platform-clock-contract-and-deterministic-conformance.md).

They settle the runtime, communication, storage, infrastructure lifecycle and
integration-plugin/client boundary, plus the closed responsibility set and
boot/recovery states of Kernel. Provider-specific operational screens remain
first-class desktop/Android experiences, while context domains consume only
neutral evidence contracts. Canonical domains, the development allowlist and
the shared durable envelope are fixed. The current implementation inventory
contains six Kernel packages and five Vault packages
are authorized, and no owner module is part of the current slice. Owner-specific
product, evidence and provider contracts still require explicit decisions
before their phase gate is opened.

The compile-isolation policy is already executable: Kernel and Gateway cannot
depend on owner modules, modules cannot depend on Kernel, integrations can use
only the exact Communications ingress contract, and aggregate backend/common/
provider packages are forbidden. The current-inventory guard is intentionally separate from the
reusable dependency validator: the former accepts only the exact ADR-0225
package set in the real workspace, while the latter can still prove future
owner graphs without authorizing those packages now.

ADR-0213 additionally requires every production module to prove independent
build, test, lifecycle and failure behavior. Its Cargo/storage/test-layout
rules are already partly executable; code-shape and process-lifecycle evidence
must be added with the first production slice where they apply.

ADR-0214 applies one Job Platform contract to every integration, domain, AI,
workflow and platform owner. Scheduler owns timing and reconciliation;
executable handlers and durable execution state stay owner-local. No Job
Platform production package exists yet.

ADR-0215 defines open local registration with zero rights before explicit
approval. Runtime rights are a typed intersection of requested, approved and
hard-policy grants. Managed and external module lifecycles are distinct.

ADR-0216 selects a private kernel-owned SQLite Control Store for registrations,
grant epochs and desired infrastructure state. A single `ControlStoreHandle`
actor owns the only online SQLite connection behind a 64-request queue, with
2-second normal and 30-second maintenance deadlines. It exposes narrow
health/recovery, owner identity, module registry, settings registry and runtime
trust ports. Registration and its effective grant set are read as one atomic
`ModuleGrantSnapshot`, so authorization never combines different actor
snapshots. The store contains no business data or secrets and is available
before PostgreSQL, PgBouncer, NATS and Vault.

ADR-0217 removes a mandatory bootstrap configuration file. Kernel derives its
private data directory from the OS standard or one explicit `--data-dir`.
Untrusted SQLite blocks managed infrastructure and the business data plane but
keeps online recovery read-only. ADR-0218 defines per-device ES256 identities,
platform signing and inherited-FD first enrollment. Destructive restore/reset
is offline-only under an exclusive instance lock.

ADR-0219 keeps external registration open and unsigned, while every Kernel
managed launch must verify exact executable bytes. Bundled entries come from a
signed distribution manifest; a promoted external executable requires an
owner-pinned digest. Kernel never downloads code and never rolls back
automatically.

ADR-0220 defines the exact `hermes-events-protocol` package for binary
`DurableEnvelopeV1`. It keeps owner payload opaque, binds it to exact contract
revision/schema SHA-256, preserves outbox bytes through NATS publication and
separates durable Ack, broker ACK, terminal result, technical DLQ and client
SSE semantics. The recovery-only package provides the accepted V1 wire
contract; NATS, outbox relay and owner runtimes remain closed.

ADR-0221 defines the exact `hermes-runtime-protocol` package for
`ModuleDescriptorV1`. Distribution manifest, descriptor, GrantSet and
RuntimeState have separate authority; capabilities are approved, resolved and
degraded independently. The package contains wire types only; the current
private module control plane performs registration, approval and capability
routing, while owner modules and public data-plane activation remain closed.

ADR-0222 makes Settings Registry an exclusive Kernel component. Module owners
declare typed schemas, while Kernel persists desired/effective revisions in the
private Control Store and supervises hot apply or restart. Settings never carry
secrets, business/runtime state, cursors or Scheduler records. No production
settings schema, API or runtime implementation exists.

ADR-0223 makes Vault a separate verified managed process. Kernel supervises it,
computes grants and routes only HPKE ciphertext; it never receives credential
plaintext or Vault keys. SQLCipher plus record-level AEAD protects bounded
credential material, while process-bound leases are fenced by runtime
generation and grant epoch. Large or high-churn provider session stores remain
integration-owned. The canonical operational summary is
[Vault and credential leases](../docs/architecture/vault-and-credential-leases.md).
The exact five-package `vault_v1` inventory implements the
file-backed key provider, SQLCipher storage through a bounded single-writer
actor, an explicit encrypted recovery key slot and the isolated runtime
boundary, fenced online lease/runtime delivery and classified component
backup/restore. This is not whole-instance backup or a business data plane.

ADR-0224 makes Storage Control a separate managed control-plane process beside
PostgreSQL and PgBouncer. Modules send business SQL directly through PgBouncer;
Storage Control owns bootstrap, roles/grants/budgets, immutable migration bundle
admission and readiness, but never proxies business queries. Database
credentials are scoped Vault leases. The target PgBouncer-only runtime path is
not a proven same-UID process boundary until OS socket/network isolation passes
conformance tests. The canonical summary is
[Storage Control Plane](../docs/architecture/storage-control-plane.md). No
production Storage adapters, managed binaries or integration suite exists yet;
the six-package foundation, `StorageBundleV1` contract and fail-closed additive
DDL admission are implemented, but do not open `storage_control_v1`.

ADR-0225 establishes the six-package `kernel_recovery_only_v1` baseline:
`hermes-events-protocol`,
`hermes-runtime-protocol`, `hermes-gateway-protocol`, the Control Store port and
SQLite adapter, and `hermes-kernel`. Later accepted foundation work opens
private module-control/managed-launch trust and adds the five Vault packages,
but it still has no NATS or business data plane and cannot reach `ready`.
`clock_v1` adds only the standalone Clock contract; all data-plane gates remain closed.

ADR-0226 keeps AI from becoming a database superuser or cross-domain
orchestrator. Cross-owner context is assembled by a typed use-case workflow
through explicit public owner contracts and passed in a distinct generated
use-case request with common `AiContextReceiptV1` metadata. A global fragment
union, opaque payload bytes, `Any`, generic maps, direct owner reads and durable
Context projections remain forbidden.

Legacy evidence reported Mail, Telegram and Zulip as previously working, but no
provider is considered implemented in the clean-room backend until new
executable evidence proves it.

## Current recovery runtime

`hermes-kernel serve` bootstraps an owner-private data directory, anchors its
SQLite Control Store to an installation ID and holds one single-instance lock.
With a trustworthy store, one Kernel process owns four `0600` private Unix
sockets: recovery, owner control, module registration and external runtime
sessions. With an unavailable or untrusted store it exposes only recovery.
The recovery socket accepts typed `status`, `validate`, `export` and `shutdown`;
restore and reset remain explicit offline CLI operations. SIGTERM/SIGINT and
recovery shutdown close every active private socket before process exit. After
an unclean exit, Kernel removes a stale control socket only when it is a Unix
socket owned by the current user; a symlink or regular file fails closed. Run
the lifecycle coverage with:

```sh
make -C backend test-kernel-recovery
```

Offline restore/reset reserves monotonic generation, identity and grant fences
in `.hermes-recovery-fence-v1` before atomically replacing SQLite. Restored
registrations are suspended and runtime attestations, sessions and managed
launch records are removed. A fence/store mismatch therefore boots
recovery-only instead of accepting rollback state.

## Linux target compilation

`make -C backend ci TARGET=x86_64-unknown-linux-gnu` runs its shared policy and
runtime gates on the invoking host, then compiles the Linux target in the
`linux/amd64` `rust:1.97.0-bookworm` container. The source bind mount is
read-only and Cargo writes build output only to the container's temporary
directory, so macOS does not need `x86_64-linux-gnu-gcc`. This is compile
conformance, not evidence for a real Linux release host, TPM/FIDO hardware or
release preflight.

## File-backed initial owner bootstrap

The first signer implementation is a local `FileDeviceSigner`: an owner-private
`0600` key file behind the replaceable `DeviceSigner` boundary. It is exportable
by design: protecting this file is equivalent to protecting the owner-device
signing authority. Generate it explicitly, then enroll the first owner/device:

```sh
cargo +1.97.0 run --manifest-path backend/Cargo.toml --package hermes-kernel -- \
  --data-dir /absolute/hermes/data device-key-generate

cargo +1.97.0 run --manifest-path backend/Cargo.toml --package hermes-kernel -- \
  --data-dir /absolute/hermes/data \
  initial-owner-enroll --owner-id owner-1 --device-id desktop-1
```

The generator never prints private material and never overwrites an existing
key. Enrollment verifies its ES256 proof in Kernel and rejects a second
first-owner claim. This is the current file-adapter baseline, not an inherited
FD ceremony or a hardware-protected signer.

The corresponding local PostgreSQL and NATS JetStream contour is documented in
[development platform](development/README.md). It is separate from both legacy
`docker/` assets and the signed/digest-pinned Linux release topology.

For the two deployment profiles, the repository currently includes a macOS
Apple-Silicon sidecar packaging target and a Linux release-manifest preflight.
Linux Compose/systemd descriptors are generated only after each digest-pinned
OCI image passes Cosign verification; neither descriptor gives Kernel Docker
API authority. See [Linux release notes](release/linux/README.md). Hardware
identity, managed launch, Vault, Storage, NATS, Blob, Clock,
Scheduler, Gateway and whole-instance backup remain closed gates.

## Validation

Run the backend-owned architecture surface from the project root:

```sh
make -C backend architecture-check
make -C backend test-architecture
make -C backend validate
```

Production packages may live only below `backend/src/`; test-support packages
may live only below `backend/tests/support/`. Tests, fixtures and inline Rust
test modules are forbidden in the production tree by executable policy.
