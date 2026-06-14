use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        // ── Legacy /api/v1/persons routes ─────────────────────────────────
        .route("/api/v1/persons", get(get_persons))
        .route("/api/v1/persons/{person_id}", get(get_person))
        .route(
            "/api/v1/persons/owner",
            get(get_owner_persona).put(put_owner_persona),
        )
        .route("/api/v1/persons/search", get(get_person_search))
        .route("/api/v1/persons/health", get(get_persons_health))
        .route("/api/v1/persons/watchlist", get(get_persons_watchlist))
        .route(
            "/api/v1/persons/{person_id}/fingerprint",
            post(post_person_fingerprint),
        )
        .route(
            "/api/v1/persons/{person_id}/favorite",
            post(post_person_favorite),
        )
        .route("/api/v1/persons/{person_id}/notes", put(put_person_notes))
        // ── ADR-0084: /api/v1/personas natively-named routes ──────────────
        .route("/api/v1/personas", get(get_personas))
        .route(
            "/api/v1/personas/{persona_id}",
            get(get_persona).put(put_persona),
        )
        .route(
            "/api/v1/personas/owner",
            get(get_owner_persona).put(put_owner_persona),
        )
        .route("/api/v1/personas/search", get(get_person_search))
        .route("/api/v1/personas/health", get(get_persons_health))
        .route("/api/v1/personas/watchlist", get(get_persons_watchlist))
        .route(
            "/api/v1/personas/{persona_id}/fingerprint",
            post(post_person_fingerprint),
        )
        .route(
            "/api/v1/personas/{persona_id}/favorite",
            post(post_person_favorite),
        )
        .route("/api/v1/personas/{persona_id}/notes", put(put_person_notes))
        .route("/api/v1/identity-candidates", get(get_identity_candidates))
        .route(
            "/api/v1/identity-traces",
            get(get_identity_traces).post(post_identity_trace),
        )
        .route(
            "/api/v1/identity-traces/{identity_id}/assignment",
            put(put_identity_trace_assignment),
        )
        .route(
            "/api/v1/identity-candidates/{identity_candidate_id}/review",
            put(put_identity_candidate_review),
        )
        .route(
            "/api/v1/persons/{person_id}/identity",
            get(get_person_identity),
        )
        .route(
            "/api/v1/persons/{person_id}/identities",
            get(get_person_identities),
        )
        .route(
            "/api/v1/persons/{person_id}/identities",
            post(post_person_identity),
        )
        .route(
            "/api/v1/persons/{person_id}/identities/{identity_id}",
            delete(delete_person_identity),
        )
        .route("/api/v1/persons/{person_id}/roles", get(get_person_roles))
        .route("/api/v1/persons/{person_id}/roles", post(post_person_role))
        .route(
            "/api/v1/persons/{person_id}/roles/{role}",
            delete(delete_person_role),
        )
        .route(
            "/api/v1/persons/{person_id}/personas",
            get(get_person_personas),
        )
        .route(
            "/api/v1/persons/{person_id}/personas",
            post(post_person_persona),
        )
        .route(
            "/api/v1/persons/{person_id}/personas/{persona_id}",
            delete(delete_person_persona),
        )
        .route(
            "/api/v1/persons/{person_id}/facts",
            get(get_person_facts).post(post_person_fact),
        )
        .route(
            "/api/v1/persons/{person_id}/memory-cards",
            get(get_person_memory_cards).post(post_person_memory_card),
        )
        .route(
            "/api/v1/persons/{person_id}/preferences",
            get(get_person_preferences).post(post_person_preference),
        )
        .route(
            "/api/v1/persons/{person_id}/timeline",
            get(get_person_timeline).post(post_relationship_event),
        )
        .route(
            "/api/v1/persons/{person_id}/snapshots",
            get(get_person_snapshots),
        )
        .route(
            "/api/v1/persons/{person_id}/history-diff",
            get(get_person_history_diff),
        )
        .route(
            "/api/v1/persons/{person_id}/enrichment",
            get(get_person_enrichment),
        )
        .route(
            "/api/v1/persons/{person_id}/enrichment/{result_id}/apply",
            post(post_person_enrichment_apply),
        )
        .route(
            "/api/v1/persons/{person_id}/enrichment/{result_id}/reject",
            post(post_person_enrichment_reject),
        )
        .route(
            "/api/v1/persons/{person_id}/expertise",
            get(get_person_expertise),
        )
        .route(
            "/api/v1/persons/search/expertise",
            get(get_person_expertise_search),
        )
        .route(
            "/api/v1/persons/{person_id}/promises",
            get(get_person_promises),
        )
        .route("/api/v1/persons/{person_id}/risks", get(get_person_risks))
        .route(
            "/api/v1/persons/{person_id}/investigate",
            post(post_person_investigate),
        )
        .route(
            "/api/v1/persons/{person_id}/dossier",
            get(get_person_dossier),
        )
        .route(
            "/api/v1/persons/{person_id}/dossier/review",
            put(put_person_dossier_review),
        )
        .route(
            "/api/v1/persons/{person_id}/meeting-prep",
            get(get_person_meeting_prep),
        )
        .route(
            "/api/v1/persons/{person_id}/analytics",
            get(get_person_analytics),
        )
        .route(
            "/api/v1/persons/{person_id}/export",
            get(get_person_export_handler),
        )
        .route("/api/v1/persons/{person_id}/health", get(get_person_health))
        .route(
            "/api/v1/persons/{person_id}/watchlist",
            post(post_person_watchlist_toggle),
        )
}
