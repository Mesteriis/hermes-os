# macOS Tauri release manifest

`verify-macos-release-manifest.mjs` checks the signed Tauri application bundle
and the packaged `hermes-kernel-aarch64-apple-darwin` sidecar on an Apple
Silicon macOS release host. The manifest requires the expected Apple Team ID
and the exact SHA-256 digest of the sidecar. The tool rejects symlinks before
hashing, then runs `codesign --verify --strict`, verifies the team identity,
and runs Gatekeeper assessment for the app bundle.

It has no software-signing fallback and does not claim notarization from a
static test. A release manifest is produced only after signing/notarization;
the release host validates it with:

```sh
make -C backend verify-macos-release MANIFEST=/absolute/macos-release.json
```
