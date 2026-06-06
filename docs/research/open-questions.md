# Research and Open Questions

## Provider Integrations

- Which email access mode should be first: IMAP/SMTP, Gmail API or provider-specific OAuth?
- What Telegram API constraints affect long-term archival and sending?
- What is the reliable WhatsApp integration path for a local-first personal product? Answered for V5 foundation by ADR-0051: use a user-visible `whatsapp_web` companion boundary; keep WhatsApp Business Platform Cloud API as a separate future provider shape.
- How should SMS be handled on desktop without unsafe phone bridge assumptions? (V5)

## Storage

- Should event payloads use JSONB, typed tables or both?
- Which graph representation in PostgreSQL gives the best balance of query power and operational simplicity?
- Which vector index is best for local-first deployment with Rust integration?
- What backup format gives reliable restore across machines?

## AI

- Which Ollama models are strong enough for extraction, classification and summarization?
- What local embedding model gives acceptable multilingual retrieval quality?
- How should extraction quality be evaluated over private data without leaking it?

## UI

- How much graph visualization is useful before it becomes noise?
- Which workflows need split-pane navigation first?
- What is the right density for a personal productivity desktop app?

## Security and Privacy

- Which OS-backed secret store should be used per platform?
- What plugin sandbox model is realistic for Tauri plus Rust?
- What confirmation policies are needed for sending messages and deleting data?
