# WhatsApp Integration API

Historical surface: [ADR-0182](../../archive/adr/ADR-0182-whatsapp-hidden-webview-runtime-only.md).

`/api/v1/integrations/whatsapp/*` serves the single `whatsapp_web` /
`whatsapp_web_companion` runtime. Account setup accepts that shape only.
Runtime status, lifecycle, capability and protected runtime-bridge routes remain
available for the hidden Tauri WebView and metadata-only observation flow.

There are no Business Cloud webhook/proxy routes, API-token setup fields, Native
MD shape, or external headless-browser endpoint. Provider-neutral user commands
remain under `/api/v1/communications/*` and completion still requires observed
evidence reconciliation.
