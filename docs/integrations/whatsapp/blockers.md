# WhatsApp Architectural Boundaries

[ADR-0182](../../adr/ADR-0182-whatsapp-hidden-webview-runtime-only.md) is the
current decision.

- The only runtime is a hidden Tauri WebView owned by the desktop process.
- The WebView cannot reveal/focus itself, use an external headless browser, read
  arbitrary files, access canonical PostgreSQL, or mutate domains directly.
- Session material remains vault-only; logs, events and health stay
  metadata-only.
- Runtime bridge observations are persisted and reconciled before commands are
  treated as completed.
- Business Cloud, Native MD and their edge proxy are retired, not fallback
  topologies.
