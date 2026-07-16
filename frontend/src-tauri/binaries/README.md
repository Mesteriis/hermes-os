# Tauri Sidecars

This directory holds generated Tauri sidecar binaries.

Apple Silicon release builds expect the signed Kernel artifact named according
to the Tauri external-binary contract:

- `hermes-kernel-aarch64-apple-darwin`

The desktop host starts it only as `hermes-kernel serve`. It passes no API
secret, provider credential, database address or other environment-derived
configuration to the child. The host can perform at most three bounded restart
attempts and never supervises Kernel children.

Generate the sidecar from the clean-room backend with:

```sh
make -C backend package-kernel-macos
```

Generated binaries are local build artifacts and are not committed.
