# Backend-wide source layout analysis

## Scope and policy

The analysis covers active clean-room Rust sources under `backend/src` and the
development-only operator under `backend/development`. Legacy reference code,
generated code, release assets, and build output are out of scope. The approved
Rust SRP policy is `.architecture/policies/rust.yaml`: source and test files
are limited to 500 lines and named functions to 60 lines.

Repository fingerprint after this execution slice: `sha256:e4cbf713c14227e235f55b1e8a0059eaf062e4d4cc03f5755540cd3e65cbfb7e`
over 6062 files. Codebase-memory graph tooling is unavailable in this session;
the analysis uses Cargo workspace manifests, explicit Rust module declarations,
and repository-local source inspection.

## Current shape

The backend has fourteen active Rust crates: Kernel, Control Store contract and
SQLite adapter, Gateway contracts, Events, Runtime Protocol, five Vault
foundation crates, a development operator, and two testkits. The major prior
SRP cuts are already present: Control Store, Vault store, Vault runtime, Vault
protocol and Runtime Protocol are domain-first.

The remaining navigation debt is local rather than a god-package problem:

- `backend/development/runtime/src/kernel_operator` still has ten unrelated
  development-prefixed modules. Artifact trust, capability routing, control
  plane, runtime simulation, settings and pairing receipt formatting must use
  their own semantic namespaces.
- Kernel `identity`, `modules`, and `runtime` contain several sibling files
  that share an owner but not a namespace. Their long prefixes are compensating
  for the missing folders.
- Events has a singleton flat validator that will attract unrelated protocol
  validation as the data-plane is introduced.
- SQLite adapter port implementations sit in a root `ports.rs` rather than an
  adapter namespace.
- Telemetry Collector's `control.rs` now combines inherited descriptor
  admission, aggregate diagnostics relay, and frame encoding. Those are
  separate protocol responsibilities despite sharing one inherited FD.

The analyzed crates already have a valid small-root pattern where `lib.rs`,
`main.rs`, or `build.rs` only composes modules. Those roots must stay flat:
Gateway contracts, Events, Runtime Protocol, Vault key-provider crates, Vault
runtime, Vault protocol, and Vault store do not receive cosmetic folder moves.

## Public API strategy

Production Kernel moves are private-module moves. They get no compatibility
facades. The Events crate keeps its public crate-root validation exports by an
explicit `pub use`; no legacy `envelope_validation` module facade is created.
No other supported external module path is changed.

## Execution slices

1. Move the development operator's root files into `cli`, `identity`,
   `pairing`, and `platform` namespaces.
2. Move the Event validator and SQLite port adapter into their owner
   namespaces, retaining only required crate-root exports.
3. Group Kernel identity, module control plane, and runtime lifecycle into
   named owner directories. Each new directory has a `mod.rs`; no directory is
   named after a generic utility role and no file receives a redundant prefix.
4. Run the architecture, recovery, Vault, target-matrix and whitespace gates.
5. Move the remaining development operator root files into `artifact`,
   `capability`, `control`, `pairing`, `runtime` and `settings` folders. The
   package itself supplies the `development` qualifier, so filenames do not.
6. Split Telemetry Collector control into `handshake`, `diagnostics`, and
   `framing` beneath the existing `control` owner. This preserves the private
   `crate::control::{describe, serve_diagnostics}` facade and avoids a
   compatibility module.

## Explicit non-actions

- No business owner, platform gate, protocol field, database schema or runtime
  behavior changes are part of this layout pass.
- No compatibility aliases are introduced for private Kernel modules.
- Existing domain-first groups such as Control Store schema/recovery and Vault
  actor/database/identity/recovery are not flattened or renamed.
- Test suite internal `part-*` fixtures are retained in this slice because they
  are already under the SRP cap and changing their module paths adds no
  production navigation value.

## Previous execution result

The previously approved layout moves were applied. Production Kernel now has only
`cli.rs` and `main.rs` at its source root; the development operator has only
its composition `main.rs` at its root. Event validation and SQLite port
adapters are in named namespaces. Test-support mirrors the new private Kernel
paths without retaining old flat module aliases. `make -C backend validate`,
the Kernel recovery testkit build, the Vault testkit (31 tests), SRP guard and
`git diff --check` were run after the moves.

## Current execution result

The user explicitly extended the navigation requirement to all active backend
packages. The recorded `MOVE-0030` through `MOVE-0040` and `IMPORT-0008`
actions moved the development operator into `artifact`, `capability`,
`control`, `pairing`, `runtime` and `settings` namespaces. This changes no
behavior, public API or production artifact. `cargo fmt --check`, targeted
operator compilation, its lifecycle simulation, `make -C backend validate`,
the SRP guard and `git diff --check` passed.

## Current analysis update

The user clarified that semantic navigation applies to the entire active
backend, not only the Kernel. Reinspection of all active Rust package roots
finds only composition roots at crate roots; the remaining actionable mixed
responsibility is Telemetry Collector control. `MOVE-0041` is internal and
approved by the user's backend-wide folder-normalization request.

## Risks and blockers

All approved source moves are internal, but Rust module declarations and test
path fixtures can drift. The move plan therefore includes explicit import
actions and full repository checks. The source tree is dirty; execution may
touch only paths named in the approved plans. The plan must be regenerated if
the fingerprint differs before execution.
## Backend-wide semantic navigation follow-up

The previous normalization completed the production Kernel and major Platform
namespaces, but two production package roots still kept a `cli.rs` file and two
test-support packages retained flat, package-prefixed test modules. This is a
repository-wide navigation issue, not a Kernel-only exception.

Approved actions `MOVE-0042` through `MOVE-0054` and `SPLIT-0009` through
`SPLIT-0010` finish this exact remainder. They retain package and Rust module
names only where they are public package roots; private test module prefixes
are removed in favor of `tests/<owner>/`. A new architecture test will enforce
that every Cargo package source root under `backend/src`, `backend/development`
and `backend/tests/support` contains only `lib.rs` and/or `main.rs` directly.

Protocol source files remain directly under their versioned Protobuf package
directories. Four or five tightly versioned wire files are one protocol surface,
not an owner collision; splitting them by transport noun would make include
paths harder to navigate without creating an ownership boundary.
