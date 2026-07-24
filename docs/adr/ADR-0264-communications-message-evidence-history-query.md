# ADR-0264: Communications message evidence history query

Статус: Принято
Дата: 2026-07-24
Состояние реализации: Запланировано этим ADR. Контракт, owner-local port и
runtime route должны быть реализованы одним срезом; legacy REST history/raw
routes не являются переходным API.

Зависит от:

- ADR-0220: canonical durable envelope;
- ADR-0240: canonical Communications owner;
- ADR-0253: Communications legacy surface disposition.

## Decision

Communications adds `ListMessageEvidence` to its generated metadata-query
contract. The request accepts only a canonical `message_id` and a bounded
limit. The response contains existing `EvidenceSummaryV1` values ordered by
observed time and evidence ID.

The persistence port resolves the canonical message's owner-local source
cursor and reads only matching rows from `communications_evidence_summaries`
with the existing owner-local audit lineage. It does not expose raw provider
records, external cursor values, bodies, Blob references, custody proofs,
provider DTOs, reaction payloads or an arbitrary history filter.

This is an evidence history, not a compatibility implementation of historical
`versions`, `raw-evidence`, `reactions` or provider REST endpoints. Provider
operational history remains integration-owned; presentation that needs it must
use that integration's own typed contract. The query is routed through the
existing Communications managed client port and does not introduce a Gateway
wrapper or cross-owner SQL.

## Required evidence

1. Public protobuf contract and schema-bound module client route decode the
   exact request and return only metadata summaries.
2. Persistence query is owner-local and joins no integration table.
3. Regression coverage proves malformed IDs/limits are rejected and that raw
   provider/body/blob fields are absent from the response contract.
4. Architecture and managed Communications runtime conformance remain green.
