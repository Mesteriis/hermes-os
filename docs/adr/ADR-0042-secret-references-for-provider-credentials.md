# ADR-0042 Secret References for Provider Credentials

Status: Proposed

## Context

ADR-0016 requires provider credentials to stay outside ordinary application tables. ADR-0041 adds provider account metadata for Gmail, iCloud Mail and generic IMAP, but real adapters will need credentials:

- Gmail needs OAuth credential material.
- iCloud Mail needs an app-specific password for IMAP.
- Generic IMAP usually needs a mailbox password and may later need SMTP credentials.

Storing credential values in provider account config would make database backups and debugging workflows unsafe.

## Decision

Store only secret references in PostgreSQL, never secret values.

Rules:

- `secret_references` stores non-secret metadata: `secret_ref`, `secret_kind`, `store_kind`, label and JSON metadata.
- Supported initial secret kinds are `oauth_token`, `app_password`, `password`, `api_token`, `private_key` and `other`.
- Supported initial secret store kinds are `os_keychain`, `encrypted_vault`, `external_vault` and `test_double`.
- Communication provider accounts bind to secrets through `communication_provider_account_secret_refs`.
- Supported initial communication secret purposes are `oauth_token`, `imap_password` and `smtp_password`.
- Gmail provider accounts should bind `oauth_token`.
- iCloud and generic IMAP provider accounts should bind `imap_password`.
- Secret values must be written to and read from the configured secret store through a future resolver.
- Provider account config and secret reference metadata must not contain OAuth tokens, app passwords, mailbox passwords, private keys or API tokens.

## Consequences

- PostgreSQL can express which credentials an adapter needs without storing credential values.
- Provider adapters can resolve credentials explicitly at runtime.
- Database backups still need secret reference metadata but do not automatically leak provider credentials.
- A future implementation must add a secret resolver for OS keychain or encrypted vault access before real provider sync can run.
