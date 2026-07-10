use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn personas_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_personas_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "personas test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

#[test]
fn persona_domain_does_not_reintroduce_legacy_person_or_contact_paths() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let forbidden_paths = [
        "backend/src/ai/core/semantic/source_persons.rs",
        "backend/src/app/api_support/query_parsing/persons.rs",
        "backend/src/app/error/conversions/persons.rs",
        "backend/src/app/error/response/persons.rs",
        "backend/src/app/handlers/communications/workflow_actions/actions/persons.rs",
        "backend/src/app/handlers/personas/compatibility.rs",
        "backend/src/app/handlers/persons",
        "backend/src/app/router/routes/persons.rs",
        "backend/src/application/organization_contact_links.rs",
        "backend/src/application/person_derived_evidence.rs",
        "backend/src/application/workflow_action_person_projection.rs",
        "backend/src/domains/organizations/core/contact_links.rs",
        "backend/src/domains/persons",
        "backend/src/domains/projects/core/read_model/people.rs",
        "backend/src/workflows/graph_projection/persons.rs",
        "backend/src/workflows/person_derived_evidence.rs",
        "backend/src/workflows/workflow_action_person_projection.rs",
        "backend/migrations/0190_rename_contact_record_observation_kind.sql",
        "backend/tests/characterization_person.rs",
        "backend/tests/person_identity.rs",
        "backend/tests/person_identity",
        "backend/tests/persons.rs",
        "backend/tests/persons",
        "backend/tests/persons_api.rs",
        "backend/tests/persons_api",
        "backend/tests/persons_api_architecture.rs",
        "backend/tests/persons_architecture.rs",
        "crates/testkit/src/factories/contact.rs",
        "docs/domains/persons",
        "frontend/src/app/queries/usePersonsViewSurface.ts",
        "frontend/src/domains/personas/queries/usePersonsPageSurface.ts",
        "frontend/src/domains/personas/queries/usePersonsSurface.ts",
        "frontend/src/domains/personas/views/PersonsPage.boundary.test.ts",
        "frontend/stories/app/Persons.stories.ts",
    ];

    let existing_forbidden_paths = forbidden_paths
        .iter()
        .filter(|path| repo_root.join(path).exists())
        .copied()
        .collect::<Vec<_>>();

    assert!(
        existing_forbidden_paths.is_empty(),
        "legacy person/contact paths must stay retired after the Personas rename:\n{}",
        existing_forbidden_paths.join("\n")
    );
}

#[test]
fn architecture_contract_does_not_list_legacy_persons_domain() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let checker_path = repo_root.join("scripts/check-architecture.mjs");
    let checker = fs::read_to_string(&checker_path)
        .unwrap_or_else(|error| panic!("failed to read {checker_path:?}: {error}"));
    let contract_path = repo_root.join("scripts/architecture-contract.json");
    let contract = fs::read_to_string(&contract_path)
        .unwrap_or_else(|error| panic!("failed to read {contract_path:?}: {error}"));

    assert!(
        !checker.contains("\n\t'persons',"),
        "architecture checker must not treat retired persons as an active backend domain"
    );
    assert!(
        !contract.contains("\"persons\""),
        "architecture contract must not treat retired persons as an active backend domain"
    );
}

#[test]
fn address_book_sync_uses_persona_native_sync_flag_only() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let checked_files = [
        "backend/src/workflows/address_book_sync.rs",
        "backend/tests/settings.rs",
        "frontend/src/domains/settings/queries/useIntegrationsSettingsSurface.ts",
    ];

    let mut violations = Vec::new();
    for relative_path in checked_files {
        let path = repo_root.join(relative_path);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        for legacy_key in [
            "contacts_sync_enabled",
            "contacts_sync_direction",
            "contacts_bidirectional_enabled",
            "contacts_remote_write_enabled",
        ] {
            if content.contains(legacy_key) {
                violations.push(format!("{relative_path}: {legacy_key}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "runtime address-book sync must use address_book_* config, not legacy contacts_* config:\n{}",
        violations.join("\n")
    );
}

#[test]
fn persona_active_source_uses_persona_metadata_name() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let checked_roots = [
        "backend/src/domains/personas",
        "frontend/src/domains/personas",
        "frontend/stories/app/Personas.stories.ts",
        "frontend/stories/app/PersonasComponents.stories.ts",
    ];

    let mut violations = Vec::new();
    for relative_root in checked_roots {
        collect_source_text_violations(
            &repo_root.join(relative_root),
            "person_metadata",
            relative_root,
            &mut violations,
        );
    }

    assert!(
        violations.is_empty(),
        "active Personas source must use persona_metadata, not person_metadata:\n{}",
        violations.join("\n")
    );
}

#[test]
fn active_source_uses_persona_mail_sync_terms() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let checked_roots = [
        "backend/src/workflows/mail_background_sync",
        "frontend/src/domains/communications",
        "frontend/src/shared/communications",
        "frontend/src/shared/mailSync",
    ];

    let mut violations = Vec::new();
    for relative_root in checked_roots {
        collect_source_text_violations(
            &repo_root.join(relative_root),
            "upserted_persons",
            relative_root,
            &mut violations,
        );
        collect_source_text_violations(
            &repo_root.join(relative_root),
            "persons_graph",
            relative_root,
            &mut violations,
        );
    }

    assert!(
        violations.is_empty(),
        "active mail sync source must use Persona terms:\n{}",
        violations.join("\n")
    );
}

#[test]
fn active_source_uses_persona_kind_names_for_indexes_and_evidence() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let checked_roots = [
        "backend/src/ai/core/semantic",
        "backend/src/domains/graph/core",
        "backend/src/domains/personas",
        "backend/src/workflows/review_promotion",
    ];

    let mut violations = Vec::new();
    for relative_root in checked_roots {
        for forbidden in [
            r#"Self::Persona => "person""#,
            r#"Self::PersonaHasEmailAddress => "person_has_email_address""#,
            r#"Self::PersonaSentMessage => "person_sent_message""#,
            r#"Self::PersonaReceivedMessage => "person_received_message""#,
            r#"Self::ProjectInvolvesPersona => "project_involves_person""#,
            r#".bind("person")"#,
            r#""kind": "person_enrichment""#,
            "upsert_person_from_review",
            "person_review_promotion",
            "person_id_for_email",
            "ai_agent_person_id",
            "person_involved_in_project",
            "person:v1:provider_address_book_entry",
            r#"format!("review-item://{}/person","#,
            r#"choose_target_id(target, "person""#,
        ] {
            collect_source_text_violations(
                &repo_root.join(relative_root),
                forbidden,
                relative_root,
                &mut violations,
            );
        }
    }

    assert!(
        violations.is_empty(),
        "active source-kind writers must use persona, not person:\n{}",
        violations.join("\n")
    );
}

#[test]
fn persona_native_api_does_not_advertise_legacy_person_routes() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let checked_root = "backend/src/app/handlers/personas/profile";

    let mut violations = Vec::new();
    for forbidden in [
        "PersonaCompatibilityReadModel",
        "legacy_person_id",
        "legacy_route",
        "/api/v1/persons",
    ] {
        collect_source_text_violations(
            &repo_root.join(checked_root),
            forbidden,
            checked_root,
            &mut violations,
        );
    }

    assert!(
        violations.is_empty(),
        "native Personas API must not expose legacy Persons compatibility metadata:\n{}",
        violations.join("\n")
    );
}

#[test]
fn persona_profile_response_does_not_expose_legacy_person_id_alias() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let checked_root = "backend/src/app/handlers/personas";
    let mut violations = Vec::new();
    collect_source_text_violations(
        &repo_root.join(checked_root),
        "person_id: String,",
        checked_root,
        &mut violations,
    );

    assert!(
        violations.is_empty(),
        "Persona handler API responses must expose persona_id only, not legacy person_id aliases:\n{}",
        violations.join("\n")
    );
}

#[test]
fn persona_dossier_response_uses_persona_field_name() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let relative_path = "backend/src/domains/personas/investigator/models.rs";
    let content = fs::read_to_string(repo_root.join(relative_path))
        .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"));

    assert!(
        content.contains("pub persona: EnrichedPersona,"),
        "Persona dossier model must expose persona, not legacy person"
    );
    assert!(
        !content.contains("pub person: EnrichedPersona,"),
        "Persona dossier model must not expose legacy person field"
    );
}

#[test]
fn persona_router_does_not_expose_legacy_persons_routes() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let checked_paths = [
        "backend/src/app/router/routes/personas.rs",
        "backend/src/app/handlers/personas/profile.rs",
    ];

    let mut violations = Vec::new();
    for relative_path in checked_paths {
        collect_source_text_violations(
            &repo_root.join(relative_path),
            "/api/v1/persons",
            relative_path,
            &mut violations,
        );
        collect_source_text_violations(
            &repo_root.join(relative_path),
            "get_legacy_person",
            relative_path,
            &mut violations,
        );
    }

    let retired_legacy_handler =
        repo_root.join("backend/src/app/handlers/personas/profile/legacy.rs");
    if retired_legacy_handler.exists() {
        violations.push("backend/src/app/handlers/personas/profile/legacy.rs".to_owned());
    }

    assert!(
        violations.is_empty(),
        "Personas router must not expose legacy /api/v1/persons routes:\n{}",
        violations.join("\n")
    );
}

#[test]
fn persona_runtime_contracts_use_persona_terms() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let checked_roots = [
        "backend/src/app/handlers/communications/workflow_actions",
        "backend/src/app/connectrpc/communications.rs",
        "backend/src/app/error/conversions/personas.rs",
        "backend/src/domains/personas/core",
        "frontend/src/domains/communications/api/connect/messageLifecycle.ts",
        "frontend/src/domains/communications/types/communications.ts",
    ];

    let mut violations = Vec::new();
    for relative_root in checked_roots {
        for forbidden in [
            "WorkflowActionTargetKind::Person,",
            "WorkflowActionTargetKind::Person)",
            "HandlerWorkflowActionTargetKind::Person =>",
            r#""person upserted"#,
            r#"kind: 'compose' | 'message' | 'task' | 'document' | 'calendar_event' | 'person'"#,
            "person identity candidates",
            "person enrichment failed",
            "person memory operation failed",
            "person core operation failed",
            "person manual observation capture failed",
            "person identity not found",
            "person persona not found",
        ] {
            collect_source_text_violations(
                &repo_root.join(relative_root),
                forbidden,
                relative_root,
                &mut violations,
            );
        }
    }

    assert!(
        violations.is_empty(),
        "active Personas runtime contracts must expose Persona terms, not legacy Person terms:\n{}",
        violations.join("\n")
    );
}

#[test]
fn persona_serialized_read_models_expose_persona_id() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let checked_files = [
        "backend/src/domains/personas/analytics.rs",
        "backend/src/domains/personas/api/models.rs",
        "backend/src/domains/personas/core/identities.rs",
        "backend/src/domains/personas/core/roles.rs",
        "backend/src/domains/personas/enrichment/models.rs",
        "backend/src/domains/personas/enrichment_engine.rs",
        "backend/src/domains/personas/expertise.rs",
        "backend/src/domains/personas/health.rs",
        "backend/src/domains/personas/intelligence.rs",
        "backend/src/domains/personas/investigator/models.rs",
        "backend/src/domains/personas/memory/cards.rs",
        "backend/src/domains/personas/memory/facts.rs",
        "backend/src/domains/personas/memory/preferences.rs",
        "backend/src/domains/personas/memory/relationship_events.rs",
        "backend/src/domains/personas/memory/snapshots.rs",
        "backend/src/domains/personas/trust/models.rs",
    ];

    let mut violations = Vec::new();
    for relative_path in checked_files {
        let content = fs::read_to_string(repo_root.join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"));
        let lines = content.lines().collect::<Vec<_>>();
        for (index, line) in lines.iter().enumerate() {
            if !line.trim_start().starts_with("pub person_id:") {
                continue;
            }
            let previous = index
                .checked_sub(1)
                .and_then(|previous_index| lines.get(previous_index))
                .copied()
                .unwrap_or_default();
            let renamed_to_persona_field = previous.contains(r#"#[serde(rename = "persona_id""#)
                || previous.contains(r#"#[serde(rename = "source_persona_id""#);
            if !renamed_to_persona_field {
                violations.push(format!("{relative_path}:{}", index + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "serialized Personas read models must expose persona_id, not legacy person_id:\n{}",
        violations.join("\n")
    );
}

#[test]
fn persona_identity_candidate_models_expose_persona_id_names() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let models_path = "backend/src/domains/personas/identity/models.rs";
    let models = fs::read_to_string(repo_root.join(models_path))
        .unwrap_or_else(|error| panic!("failed to read {models_path}: {error}"));
    assert!(
        models.contains("pub left_persona_id: String,"),
        "Persona identity candidate model must expose left_persona_id"
    );
    assert!(
        models.contains("pub right_persona_id: Option<String>,"),
        "Persona identity candidate model must expose right_persona_id"
    );
    assert!(
        !models.contains("pub left_person_id: String,")
            && !models.contains("pub right_person_id: Option<String>,"),
        "Persona identity candidate model must not expose legacy left/right person_id fields"
    );

    let upsert_path = "backend/src/domains/personas/identity/upsert.rs";
    let upsert = fs::read_to_string(repo_root.join(upsert_path))
        .unwrap_or_else(|error| panic!("failed to read {upsert_path}: {error}"));
    assert!(
        upsert.contains(r#""left_persona_id": &payload.left_persona_id"#)
            && upsert.contains(r#""right_persona_id": &payload.right_persona_id"#),
        "persona_identity.candidate.detected payload must use persona-native field names"
    );
    assert!(
        !upsert.contains(r#""left_person_id": &payload.left_persona_id"#)
            && !upsert.contains(r#""right_person_id": &payload.right_persona_id"#),
        "persona_identity.candidate.detected payload must not emit legacy left/right person_id fields"
    );
}

#[test]
fn persona_derived_evidence_events_and_observations_use_persona_id_payloads() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let checked_files = [
        "backend/src/domains/personas/api/store/email_projection.rs",
        "backend/src/domains/personas/command_service.rs",
        "backend/src/domains/personas/core/identities.rs",
        "backend/src/domains/personas/core/interaction_contexts.rs",
        "backend/src/domains/personas/core/roles.rs",
        "backend/src/domains/personas/enrichment/commands.rs",
        "backend/src/domains/personas/health.rs",
        "backend/src/domains/personas/investigator/service.rs",
        "backend/src/domains/personas/memory/cards.rs",
        "backend/src/domains/personas/memory/facts.rs",
        "backend/src/domains/personas/memory/preferences.rs",
        "backend/src/domains/personas/memory/relationship_events.rs",
        "backend/src/domains/personas/trust/promises.rs",
        "backend/src/application/organization_persona_links.rs",
        "backend/src/app/handlers/organizations/core_records.rs",
        "backend/src/domains/organizations/core/persona_links.rs",
        "backend/src/domains/organizations/service.rs",
        "backend/src/workflows/persona_derived_evidence.rs",
    ];

    let mut violations = Vec::new();
    for relative_path in checked_files {
        let content = fs::read_to_string(repo_root.join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"));
        for forbidden in [
            r#""person_id":"#,
            r#""person_id": &"#,
            r#""person_id": persona_id"#,
            r#""person_id": promise"#,
            "person_role_assigned:",
            "person_role_removed:",
            "person_role:",
            "person_trust_score_changed:",
            "filename=person_",
            "post_person_fingerprint",
        ] {
            if content.contains(forbidden) {
                violations.push(format!("{relative_path}: {forbidden}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "new Persona-derived events/observations must emit persona_id payloads and persona event ids:\n{}",
        violations.join("\n")
    );
}

#[test]
fn persona_docs_do_not_advertise_retired_persons_api_or_person_id_payloads() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend crate has repository parent")
        .to_path_buf();
    let checked_files = [
        "docs/domains/personas/api.md",
        "docs/domains/personas/README.md",
        "docs/domains/personas/architecture.md",
        "docs/domains/personas/blockers.md",
        "docs/domains/personas/status.md",
        "docs/product/development-roadmap.md",
        "docs/refactoring/naming-conflicts-inventory.md",
    ];
    let forbidden_phrases = [
        "payloads still expose `person_id`",
        "backend payloads still expose `person_id`",
        "compatibility payload fields remain",
        "compatibility route set",
        "remains legacy route set",
        "routes remain compatibility-only",
        "legacy `/api/v1/persons/*` routes remain",
        "`/api/v1/persons/*` остаётся",
        "`/api/v1/persons/*` сохраняется",
        "`/api/v1/persons/*` routes remain",
    ];

    let mut violations = Vec::new();
    for relative_path in checked_files {
        let path = repo_root.join(relative_path);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"));
        for forbidden in forbidden_phrases {
            if content.contains(forbidden) {
                violations.push(format!("{relative_path}: {forbidden}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Personas docs must describe persona-native read payloads and retired /api/v1/persons routes:\n{}",
        violations.join("\n")
    );
}

fn collect_personas_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_personas_test_violations(&path, violations);
            continue;
        }
        if !is_personas_test_file(&path) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let line_count = content.lines().count();
        if line_count > MAX_TEST_FILE_LINES {
            violations.push(format!("{}: {line_count}", path.display()));
        }
    }
}

fn collect_source_text_violations(
    path: &Path,
    forbidden: &str,
    display_root: &str,
    violations: &mut Vec<String>,
) {
    if path.is_dir() {
        let entries = fs::read_dir(path)
            .unwrap_or_else(|error| panic!("failed to read source path {path:?}: {error}"));
        for entry in entries {
            let entry = entry.expect("failed to read source directory entry");
            collect_source_text_violations(&entry.path(), forbidden, display_root, violations);
        }
        return;
    }

    if !path.is_file() {
        return;
    }

    let content =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
    if !content.contains(forbidden) {
        return;
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    violations.push(format!("{display_root}/{file_name}: {forbidden}"));
}

fn is_personas_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "personas.rs" || file_name == "personas_architecture.rs" {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "personas")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
