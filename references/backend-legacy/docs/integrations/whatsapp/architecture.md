# WhatsApp Architecture

The previous architecture was defined by archived
[ADR-0182](../../archive/adr/ADR-0182-whatsapp-hidden-webview-runtime-only.md).

```text
Hidden account-scoped Tauri WebView
  -> metadata-only runtime bridge
  -> raw observation / Signal Hub
  -> Communications canonical evidence
  -> review, timeline and domain promotion
```

The WebView has no arbitrary filesystem capability, no direct PostgreSQL access
and no direct domain calls. It may claim durable commands through the protected
runtime bridge and reports provider observation for canonical reconciliation.
No Native MD, Business Cloud, sidecar/edge proxy or external headless browser
is part of this topology.
