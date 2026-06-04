# V1 Closure Checklist

## Release Goal

Version 1.0 is complete when a user can run Hermes Hub locally, import email fixture data, inspect canonical messages and contacts, search local memory, import Markdown/PDF files into the document boundary, and open a desktop-first shell connected to backend readiness.

## In Scope

- Local Rust backend with PostgreSQL migrations and readiness checks.
- Event log, projection cursors and audited local API access.
- Email provider account metadata for `gmail`, `icloud` and `imap`.
- Account-scoped credential references and runtime credential resolution boundary.
- Fixture-based first email import path that preserves raw provider records.
- Canonical message projection from raw email records.
- Basic contact projection from message participants.
- Tantivy full text search over messages and imported Markdown text.
- Document import boundary for Markdown text and PDF metadata.
- Desktop-first SvelteKit/Tauri shell with health, readiness, messages, contacts, search and documents screens.

## Out of Scope For V1

- Real Gmail OAuth sync.
- Real iCloud IMAP sync.
- Real generic IMAP networking.
- Mobile UI design, implementation or validation.
- OCR, entity linking and AI summaries.
- Backup/restore.
- Plugin runtime.

## Acceptance Gates

- `make validate` passes from a clean checkout with Docker available.
- `make backend-communication-smoke-dev` imports fixture email records idempotently.
- `make backend-search-smoke-dev` indexes and queries at least one message.
- `make backend-documents-smoke-dev` imports one Markdown file and one PDF metadata record.
- Desktop shell starts and shows backend readiness from `GET /readyz`.
