# Tauri Sidecars

This directory holds generated Tauri sidecar binaries.

macOS release builds expect:

- `hermes-hub-backend-aarch64-apple-darwin`
- `hermes-hub-backend-x86_64-apple-darwin`

Generate the current host sidecar with:

```sh
make backend-sidecar-macos
```

Generated binaries are local build artifacts and are not committed.
