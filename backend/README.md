# Hermes backend clean room

The production backend rewrite starts here. `Cargo.toml` is an intentionally
empty virtual workspace: there is no production package yet because contracts,
supported behavior and ownership boundaries must be agreed before code is
introduced.

The previous implementation is available at
`references/backend-legacy/`, but it is not a dependency and is not an
architectural template.

## Entry conditions for new code

Before creating the first crate, the clean-room design must fix:

1. the product capabilities that are actually supported;
2. canonical evidence and event invariants that must remain stable;
3. bounded-context ownership and dependency direction;
4. the frontend-to-backend contract and cutover rule;
5. runtime lifecycle, shutdown and provider isolation;
6. schema ownership and the fresh-database bootstrap;
7. test boundaries that do not compile production composition.

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

They settle the runtime, communication, storage, infrastructure lifecycle and
integration-plugin/client boundary, plus the closed responsibility set and
boot/recovery states of Kernel. Provider-specific operational screens remain
first-class desktop/Android experiences, while context domains consume only
neutral evidence contracts. Canonical domains and the current implementation
allowlist are fixed; owner-specific contracts and unresolved distribution
manifest details still require explicit decisions before their production
packages are introduced.

Legacy evidence reported Mail, Telegram and Zulip as previously working, but no
provider is considered implemented in the clean-room backend until new
executable evidence proves it.

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
