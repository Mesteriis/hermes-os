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
- `oauth_token` bindings require `secret_kind = oauth_token`; `imap_password` and `smtp_password` bindings require `secret_kind = app_password` or `password`.
- Multiple accounts for the same provider kind are supported. Credential lookup must use the provider `account_id` and secret purpose, not provider kind alone.
- Provider adapters should use the account-scoped `ProviderCredentialReader` path instead of reimplementing credential joins.
- Secret values must be written to and read from the configured secret store through a `SecretResolver` boundary.
- The in-memory resolver is valid only for `test_double` references in tests and local adapter tests. It must not resolve `os_keychain`, `encrypted_vault` or `external_vault` references.
- Provider account config and secret reference metadata must not contain OAuth tokens, app passwords, mailbox passwords, private keys or API tokens.

## Consequences

- PostgreSQL can express which credentials an adapter needs without storing credential values.
- Provider adapters can resolve credentials explicitly at runtime.
- Missing credential bindings, incompatible secret kinds and resolver failures are reported explicitly before provider network calls begin.
- Provider adapters can support multiple Gmail, iCloud or IMAP accounts without shared global credentials.
- Database backups still need secret reference metadata but do not automatically leak provider credentials.
- A future implementation must add a secret resolver for OS keychain or encrypted vault access before production account setup can resolve non-test credentials.
