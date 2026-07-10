# ADR-0062 Organization Identity and Resolution

Status: Proposed

## Context

Organizations have multiple identifiers: domains, VAT/CIF/NIF numbers, GitHub orgs, LinkedIn pages, phone numbers, and portal URLs. These must be stored with provenance and used for deduplication. ADR-0019 established identity resolution for Personas; organizations need the same capability.

## Decision

`organization_identities` stores all identifiers with type, value, source, confidence, and status. Identity resolution compares organizations by domain overlap, VAT match, legal name similarity, and shared contacts. Merge candidates are generated with confidence scoring and user-confirmed. Merge is reversible — confirming a merge materializes a split candidate.

## Consequences

- Domain intelligence enables automatic linking of emails and contacts to organizations.
- VAT/VIES validation provides high-confidence identity matching.
- Aliases (`organization_aliases`) capture brand/trading/former names.
- Merge/split workflow mirrors the Persona identity resolution from ADR-0019.
