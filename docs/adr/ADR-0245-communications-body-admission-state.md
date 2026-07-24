# ADR-0245: Communications body admission state

Status: Accepted
Date: 2026-07-22
Implementation state: Partially implemented. `Unavailable` and the admitted
Blob-backed canonical-evidence path are implemented through the event-backed
custody transfer in ADR-0257: an integration emits only a typed opaque source
receipt, Communications records `PendingBlob`, and the owner-local worker
commits `AdmittedBlob` only after Blob Platform rebinds it to Communications.
This does not admit a generic provider-body fetch, a direct cross-owner Blob
read, or every future integration body producer.

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

An admitted integration body producer may move a specific evidence record from
`PendingBlob` to `AdmittedBlob` only through the ADR-0257 custody-transfer
workflow. Mail and Telegram paths not separately admitted for body write still
publish `Unavailable`; WhatsApp host observations remain metadata-only. No path
may mutate provider storage or bypass the Blob boundary.
