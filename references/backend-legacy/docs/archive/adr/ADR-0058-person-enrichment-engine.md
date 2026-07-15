# ADR-0058 Person Enrichment Engine Boundary

Status: Proposed

## Context

The functional spec describes automated enrichment from GitHub, LinkedIn, public web, and internal communication analysis. Enrichment must be auditable, reversible, and never silently overwrite user-confirmed data.

## Decision

A dedicated `EnrichmentEngine` service orchestrates data acquisition with pluggable providers. All results go through `enrichment_results` with status tracking (`pending`, `applied`, `rejected`, `conflict`). The engine never auto-applies enrichment without user confirmation for medium/low confidence results. High-confidence facts from verified sources may auto-apply with audit trail. Profile verification uses cross-source correlation.

## Consequences

- Enrichment is auditable and reversible.
- Providers (GitHub, LinkedIn, web) are pluggable behind a common trait.
- User retains control over what data enters the person profile.
- Auto-discovery from email domain probes GitHub, LinkedIn, and company website.
