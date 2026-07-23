# ADR-0246: Communications attachment admission and safety lifecycle

Status: Accepted
Date: 2026-07-22
Implementation state: Partial. Canonical attachment anchors persist immutable
provider-observed descriptors in the `descriptor_only` state. Mail, Telegram
and WhatsApp publish bounded descriptor observations through Communications
ingress. Blob admission commands/results and scanner verdict events are not
implemented yet; no anchor can reach `blob_admitted`, `safe_for_delivery`,
`quarantined` or `rejected` in the current runtime.

## Decision

Communications owns the canonical attachment anchor, immutable provider-observed
descriptor and attachment safety lifecycle. An attachment descriptor contains
only bounded display and integrity metadata: optional filename, media type,
declared byte length, optional SHA-256 digest and disposition. It never carries
bytes, a provider-local locator, filesystem path, session material, Blob
capability or an executable scan result.

An integration emits an `AttachmentObserved` Communications observation only
after recording its provider-local source projection and durable outbox entry.
The observation is a provider-neutral, hash-scoped descriptor event. It does
not call Communications storage or Blob storage directly.

Blob admission is an explicit platform capability operation initiated by the
owning integration or a dedicated workflow. Its terminal result is published as
a typed event correlated to the canonical attachment anchor. Communications
consumes that event and records one of these closed states:

```text
descriptor_only
blob_pending
blob_admitted
quarantined
safe_for_delivery
rejected
```

The Blob platform owns bytes, content addressing, capability grants and blob
deletion. A security scanner is a distinct platform or engine owner and emits
a bounded verdict event. Communications may project the verdict, but never
loads bytes or invokes scanner implementations. `safe_for_delivery` requires
an admitted Blob anchor and a terminal clean verdict; no missing, failed or
unknown verdict is treated as clean.

## Consequences

- Mail, Telegram and WhatsApp keep provider locators and download state in
  their integration-owned persistence.
- Communications receives only durable events and keeps the cross-provider
  canonical attachment graph.
- No domain imports an integration, Blob implementation or scanner engine.
- Blob and scanner implementations do not import Communications; they publish
  their public result events.
- Legacy attachment files, in-memory scans and direct database joins are not
  compatibility paths and are not restored.
- Body admission under ADR-0245 and attachment admission use separate typed
  anchors; admitting one does not imply admitting the other.
