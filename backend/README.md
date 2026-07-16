# Hermes backend clean room

The production backend rewrite starts here. The virtual workspace now contains
the exact six-package `kernel_recovery_only_v1` inventory accepted by ADR-0225:
three Protobuf contracts, the Control Store port and SQLite adapter, and the
Kernel binary. It is deliberately not a general platform runtime: the only
reachable state is private `recovery_only` without external services, managed
children, network listeners or business routes.

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

They settle the runtime, communication, storage, infrastructure lifecycle and
integration-plugin/client boundary, plus the closed responsibility set and
boot/recovery states of Kernel. Provider-specific operational screens remain
first-class desktop/Android experiences, while context domains consume only
neutral evidence contracts. Canonical domains, the development allowlist and
the shared durable envelope are fixed. The current implementation inventory is
narrower: only the six recovery-only foundation packages from ADR-0225 are
authorized, and no owner module is part of the current slice. Owner-specific
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
grant epochs and desired infrastructure state. It is available before
PostgreSQL, PgBouncer, NATS and Vault, contains no business data or secrets and
is isolated behind a dedicated port/persistence package split.

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
degraded independently. The package contains wire types only; registration and
capability activation remain closed.

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
No production Vault package, storage format or conformance suite exists yet.

ADR-0224 makes Storage Control a separate managed control-plane process beside
PostgreSQL and PgBouncer. Modules send business SQL directly through PgBouncer;
Storage Control owns bootstrap, roles/grants/budgets, immutable migration bundle
admission and readiness, but never proxies business queries. Database
credentials are scoped Vault leases. The target PgBouncer-only runtime path is
not a proven same-UID process boundary until OS socket/network isolation passes
conformance tests. The canonical summary is
[Storage Control Plane](../docs/architecture/storage-control-plane.md). No
production Storage packages, managed binaries or integration suite exists yet.

ADR-0225 authorizes only `kernel_recovery_only_v1`: `hermes-events-protocol`,
`hermes-runtime-protocol`, `hermes-gateway-protocol`, the Control Store port and
SQLite adapter, and `hermes-kernel`. Kernel metadata may declare only
`supervisor` and `core_gateway`; it has no external services, managed children,
NATS or business data plane and cannot reach `ready`. All later capabilities
remain closed behind explicit phase gates.

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

`hermes-kernel` bootstraps an owner-private data directory, anchors its SQLite
Control Store to an installation ID, holds a single-instance lock and exposes
only a private Unix recovery socket. The socket accepts typed `status`,
`validate`, `export` and `shutdown` messages; restore and reset remain explicit
offline CLI operations. SIGTERM/SIGINT and malformed IPC requests remove the
socket before the process exits. Run the lifecycle coverage with:

```sh
make -C backend test-kernel-recovery
```

## Development platform bootstrap

`development_full_platform_v1` deliberately permits a local software ES256
signer, persistent development state and later local platform services without
Apple/TPM/FIDO2 hardware. It is selected explicitly and is never production
evidence. On a new owner-private development data directory, create its first
development owner/device with:

```sh
cargo +1.97.0 run --manifest-path backend/Cargo.toml --package hermes-kernel -- \
  --development-profile --data-dir /absolute/development/data \
  initial-owner-enroll --owner-id dev-owner --device-id dev-device
```

The command emits an insecure-profile warning, stores a `0600` development key
only under that data directory, verifies its ES256 proof in Kernel and rejects
a second first-owner claim. It is not the production inherited-FD enrollment
ceremony and must not receive production credentials or private user data.

The corresponding local PostgreSQL and NATS JetStream contour is documented in
[development platform](development/README.md). It is separate from both legacy
`docker/` assets and the signed/digest-pinned Linux release topology.

For the two deployment profiles, the repository currently includes a macOS
Apple-Silicon sidecar packaging target and a Linux release-manifest preflight.
Linux Compose/systemd descriptors are generated only after each digest-pinned
OCI image passes Cosign verification; neither descriptor gives Kernel Docker
API authority. See [Linux release notes](release/linux/README.md). Hardware
identity, pairing, managed launch, Vault, Storage, NATS, Blob, Clock,
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
