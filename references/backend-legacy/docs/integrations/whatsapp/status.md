# WhatsApp Implementation Status

Historical policy: [ADR-0182](../../archive/adr/ADR-0182-whatsapp-hidden-webview-runtime-only.md).

Implemented topology: a single hidden, account-scoped Tauri WebView with
metadata-only runtime-bridge dispatch. It is not a general WhatsApp client and
does not expose a visible login window or headless browser automation.

The retired Business Cloud and Native MD implementations, their credentials,
workers, edge proxy and routes have been removed. Live provider actions remain
subject to explicit operator smoke validation using sanitized fixtures/evidence.
