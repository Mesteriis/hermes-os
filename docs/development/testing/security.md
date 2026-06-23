# Security and Dependency Hygiene

Commands:

- `make audit` - RustSec vulnerability scan via `cargo-audit`
- `make deny` - advisory/license/source/multi-version checks via `cargo-deny`
- `make security` - combined audit + deny
- `make udeps` - unused dependency scan via `cargo-udeps` on nightly

Files:

- `deny.toml`

Notes:

- `cargo-udeps` requires nightly Rust to execute.
- `cargo-deny` is broader than `cargo-audit`; it also checks sources, versions, and licenses.
