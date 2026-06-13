# Hermes Implementation Status

Короткий статус по приведению кода к текущей документации.

## Сделано

- `DONE` PersonaType и единственный Owner Persona в compatibility `persons`.
- `DONE` AI agents как `ai_agent` Personas, включая `HEPHAESTUS` и `name@sh-inc.ru`.
- `DONE` AI run attribution через agent Persona и Owner Persona.
- `DONE` Relationship persistence baseline и graph projection.
- `DONE` Polygraph / Contradiction Observation baseline.
- `DONE` Decisions и Obligations persistence baseline.
- `DONE` Persona identity traces для `document_mention`, `message_participant`, `disputed` и unattached create/list/assign API.
- `DONE` UI review workflow для unattached identity traces.
- `DONE` Persona-native `/api/v1/personas` read/write compatibility API bridge baseline.
- `DONE` AI workspace UI context через Owner Persona.
- `DONE` Persona Dossier UI read-model wiring.
- `DONE` Persona Dossier cache/review workflow baseline.
- `DONE` Cross-domain Review shell для Relationships, Decisions, Obligations и Polygraph suggested queues.
- `DONE` Cross-domain Review action dispatch policy для confirm/reject routing.
- `DONE` Risk Engine attention-status baseline for Persona health compatibility cache.
- `DONE` Risk Engine source-backed Persona observation baseline.
- `DONE` Trust Engine compatibility score baseline for Owner Persona trust Relationships.
- `DONE` Trust Engine source reliability signal baseline for reviewable evidence.
- `DONE` Memory Engine compatibility notes-to-memory-card baseline.
- `DONE` Enrichment Engine compatibility favorite-to-preference baseline.
- `DONE` Enrichment Engine source-backed pending Persona candidate baseline.
- `DONE` Timeline Engine compatibility policy baseline for bounded source-backed timeline views.

## Осталось

- `TODO` Physical Persona-native schema migration beyond compatibility storage.
- `TODO` Broader Timeline Engine event-log replay, cross-domain timelines, summaries and gap detection.
- `TODO` Broader Enrichment Engine approved-source policy, conflict routing and cross-domain candidates.
- `TODO` Broader Memory Engine context/review workflow beyond compatibility notes.
- `TODO` Broader Trust Engine contradiction inputs, review recommendations and cross-domain reconciliation.
- `TODO` Broader Risk Engine cross-domain observations, review routing and health/watchtower terminology migration.
- `TODO` Broader live-provider ingestion for Decisions, Obligations and Polygraph.
- `TODO` Broader review policy beyond current queue aggregation and confirm/reject dispatch.
