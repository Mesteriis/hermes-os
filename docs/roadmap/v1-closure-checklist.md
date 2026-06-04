# V1 Closure Checklist

## Release Goal

Version 1.0 is complete when a user can run Hermes Hub locally, import email fixture data, inspect canonical messages and contacts, search local memory, import Markdown/PDF files into the document boundary, and open a desktop-first shell connected to backend V1 status.

## In Scope

- Local Rust backend with PostgreSQL migrations and readiness checks.
- Event log, projection cursors and audited local API access.
- Email provider account metadata for `gmail`, `icloud` and `imap`.
- Account-scoped credential references and runtime credential resolution boundary.
- Fixture-based first email import path that preserves raw provider records.
- Canonical message projection from raw email records.
- Basic contact projection from message participants.
- Tantivy search boundary covered by message and document record tests.
- Document import boundary for Markdown text and PDF metadata.
- Desktop/laptop SvelteKit/Tauri status shell connected to `GET /api/v1/status`.

## Out of Scope For V1

- Real Gmail OAuth sync.
- Real iCloud IMAP sync.
- Real generic IMAP networking.
- Mobile UI design, implementation or validation.
- OCR, entity linking and AI summaries.
- Backup/restore.
- Plugin runtime.

## Acceptance Gate Status

- [x] `make validate` passes from a clean checkout with Docker available.
- [x] Fixture email import preserves raw provider records idempotently.
- [x] Canonical messages projection is covered by live PostgreSQL tests.
- [x] Contacts projection is covered by live PostgreSQL tests.
- [x] Tantivy search boundary is covered by message/document record tests.
- [x] Document import stores Markdown text and PDF metadata.
- [x] Desktop shell shows backend V1 status.
