# ADR-0175 iCloud CardDAV Address Book Import

Status: Accepted
Date: 2026-07-10

## Decision

iCloud Contacts are imported through a read-only CardDAV adapter in the Mail
integration. The adapter uses the existing iCloud account identity and
app-specific password held in the host vault, performs CardDAV discovery, and
returns provider-neutral address-book entries to the existing address-book sync
workflow. The workflow remains the owner of Persona promotion and provenance.

Remote iCloud contact writes are intentionally out of scope. They require
ETag-aware CardDAV PUT/DELETE semantics and explicit owner approval.
