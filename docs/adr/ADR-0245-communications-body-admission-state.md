# ADR-0245: Communications body admission state

Status: Accepted
Date: 2026-07-22
Implementation state: Partial. `Unavailable` is implemented; Blob-backed body
admission remains a separate platform-gated slice.

## Decision

Communications evidence represents body availability as a closed typed state:
`MetadataOnly`, `PendingBlob`, or `Unavailable`.

An integration may emit `PendingBlob` only after a concrete owner-authorized
body admission operation has created a Blob-backed anchor. If provider content
is observed but no such operation is admitted, it emits `Unavailable`.

Raw body text, HTML, MIME, media bytes and provider-local file paths never
cross the Communications ingress. The domain does not fabricate an empty body,
perform a legacy fallback fetch, or treat provider readability as Blob success.

## Consequences

Mail and Telegram currently publish `Unavailable` for readable provider text.
WhatsApp host observations remain metadata-only. A future Blob slice may move a
specific evidence record from `PendingBlob` to an admitted body anchor through
an explicit typed event; it must not mutate provider storage or bypass the Blob
boundary.
