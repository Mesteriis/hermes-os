# Telegram tgcalls media adapter

This integration-owned build unit contains the Rust loader/session adapter and
the narrow native C ABI used by Telegram one-to-one audio calls. It is not a
domain, assembly, independently managed runtime, or generic media service.

The release artifact is built from exact Telegram-iOS, tgcalls, WebRTC, Bazel
and Xcode inputs:

```sh
backend/scripts/build-telegram-tgcalls-bridge-macos.sh \
  --output-dir /absolute/new/output-directory
```

The command refuses an existing output directory, a non-arm64 macOS host, an
Xcode version other than the version pinned by Telegram-iOS, altered upstream
commits, altered tgcalls license bytes, or altered Bazel bytes. Its output
contains the dylib, the upstream LGPL-3.0 license and a provenance manifest.

The pinned Telegram-iOS `tgcalls_core` target omits the macOS implementation of
`AudioDeviceModule::Create` from final consumers. The exact source patch in
`native/patches/` adds only that CoreAudio build unit; the production bridge
still uses the platform-default input and output devices. Fake PCM adapters are
not linked into the production bridge.

This build and its loader conformance do not by themselves open
`telegram_call_media_v1`. Assembly/runtime digest binding, TDLib ready/signaling
wiring, teardown/fence tests, a real audio-loop check and an authorized live
one-to-one call remain separate admission evidence under ADR-0284.
