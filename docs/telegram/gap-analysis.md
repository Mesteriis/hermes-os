# Telegram Gap Analysis

Status date: 2026-06-18.

Base Telegram channel capability status: `COMPLETED`.

This document tracks active Telegram channel capability gaps only. Deferred
initiatives are not base Telegram channel gaps; ADR-0094 and ADR-0097 move them
to separate future work.

## Closure Summary

| Area | Status | Evidence |
|---|---|---|
| Provider reconciliation | CLOSED | Edit, delete, pin, archive, mute, read, unread, reactions, topics, folder add, folder remove and folder reassign use durable provider-write commands and reconcile from provider-observed state or returned provider snapshots. |
| Message lifecycle | CLOSED | Edit versions, tombstones, provider edit/delete evidence and diff metadata are persisted and surfaced through projection APIs. |
| Reply/forward parity | CLOSED | Reply refs are idempotent, reply graph traversal is bounded with cycle guard, forward attribution is idempotent, and forward chains traverse projected local evidence without raw TDLib UI dependency. |
| Topic parity | CLOSED | Topic unread state, realtime topic patching and topic command reconciliation are implemented through `telegram_topics`, runtime topic events and command reconciliation. |
| Dialog parity | CLOSED | Pinned, archived, mute, unread and folder state use provider evidence when TDLib state is available; local projection remains the read model, not a success substitute. |
| Search parity | CLOSED | Message/provider/media/topic/member search routes return projection-backed results; provider search refreshes projection before UI-visible results. |
| Media parity | CLOSED | Gallery, album metadata, preview, attachment lifecycle, upload lifecycle and download lifecycle use the command/query model and shared Communication attachment boundary. |
| Frontend state | CLOSED | Telegram production components use TanStack Query composables and shared realtime bootstrap; no component-level `fetch(` remains in Telegram production UI. |
| Architecture guardrails | CLOSED | Telegram remains a Communication Channel; Memory, Knowledge, Persona, Organization, Project, Obligation and Decision lifecycle stay outside Telegram. |

## Deferred, Not Gaps

| Capability | Capability state |
|---|---|
| Bot Runtime | `planned` |
| Voice Recording | `planned` |
| Voice Send | `planned` |
| Video Recording | `planned` |
| Live Calls | `planned` |
| Session Export | `planned` |
| Session Import | `planned` |
| MTProxy | `planned` |
| SOCKS5 | `planned` |
| AI Summary | `planned` |
| Translation | `planned` |
| Bilingual Reply | `planned` |
| AI Review Flows | `planned` |

## Closure Gates

| Gate | Status |
|---|---|
| Provider writes use outbox | CLOSED |
| Destructive actions use audit | CLOSED |
| Realtime events use shared event bus/bootstrap | CLOSED |
| Polling is not used where realtime path exists | CLOSED |
| Telegram implementation, test, docs and frontend files stay under 700 lines | CLOSED |
| Documentation matches channel capability scope | CLOSED |

Live Telegram validation remains opt-in because TDLib credentials, native
library loading and account QR authorization are local machine concerns.
Fixture/projection/outbox/realtime tests are the deterministic closure gate.
