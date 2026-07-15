# ADR-0063 Organization Passive OSINT Boundary

Status: Proposed

## Context

The functional spec requires enrichment from public sources: website, GitHub, LinkedIn, VIES, public registries. This must be done without active scanning, brute force, or access control bypass.

## Decision

Enrichment uses only public APIs and passive observation. Providers include: website (about page, contact info), VIES (VAT validation), GitHub (public org profile), LinkedIn (public company page), public registries. All results go through `organization_enrichment_results` with pending/applied/rejected/conflict status. Auto-apply only for high-confidence results from verified sources. The spec explicitly forbids: active infrastructure scanning, brute force, access control bypass, closed data collection, pentest, mass scraping without control.

## Consequences

- Enrichment is auditable through the results table.
- User retains control over what data enters the organization profile.
- VAT/VIES validation provides high-confidence legal identity verification.
- Technology profile and open source footprint are derived from public data only.
