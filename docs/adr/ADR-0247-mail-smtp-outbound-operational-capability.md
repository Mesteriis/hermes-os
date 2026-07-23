# ADR-0247: Mail SMTP outbound operational capability

Status: Accepted
Date: 2026-07-22
Implementation state: Partial. Mail API/core define bounded SMTP endpoint and
outbound command contracts, a distinct SMTP credential purpose and RFC822
plain-text composition with header-injection rejection. No SMTP package,
runtime command route, provider execution or terminal result exists yet.

## Decision

SMTP send is a Mail integration capability, not a Communications domain
capability. Its clean-room implementation is an explicit extension of the
Mail owner after ADR-0239's IMAP read-only slice.

The exact package split is:

```text
hermes-mail-api
hermes-mail-core
hermes-mail-smtp
hermes-mail-persistence
hermes-mail-runtime
```

`hermes-mail-smtp` may depend on its public Mail API and selected TLS/runtime
libraries only. It must not depend on Mail persistence, Mail runtime,
Communications, Gateway, Blob implementation, Vault implementation or a
provider SDK. Mail runtime is the only composition root and resolves an
SMTP-specific Vault lease. IMAP and SMTP credentials are distinct
`MailCredentialPurpose` values; SMTP never silently reuses an IMAP password.

An outbound request is a typed Mail operational durable command. `accepted`
only means Mail persisted the command; SMTP execution produces a terminal
Mail result. Neither result grants Communications direct access to Mail
operational state. A confirmed send may emit a separate neutral evidence
observation through Communications ingress, with provider receipt identifiers
kept Mail-owned and no SMTP response text, credential, recipient address or
message body in subjects, diagnostics or result errors.

Attachments are read only through an explicit Blob capability lease supplied
to Mail runtime. SMTP does not resolve Communications anchors, read a domain
table, receive provider session state from another integration, or accept a
filesystem path from a client.

## Admission gates

This capability remains blocked until one atomic policy/inventory admission
adds the SMTP package and all its exact dependencies, targets and source roots.
Required evidence includes implicit-TLS SMTP conformance, exact RFC822/MIME
serialization and header-injection rejection, command idempotency and replay,
Vault purpose/revoke fencing, Blob lease-only attachment streaming,
compile-isolation, and generated Core Gateway contract evidence without an
HTTP compatibility facade.

## Consequences

No SMTP code may be hidden in `hermes-mail-imap`, `hermes-mail-core` may not
open sockets, and Communications cannot become a generic outbound provider
dispatcher. The currently accepted IMAP read-only capability remains unchanged
until the admission gates above are implemented and verified.
