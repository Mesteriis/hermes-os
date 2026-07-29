import { duplicates, violation } from './validation-diagnostics.mjs';

const PHASE_GATE_KEYS = [
  'transitionAuthority',
  'notAuthorized',
  'requires',
  'conditionalRequires',
  'requiredDecisionFields',
  'ownerAdmissionExceptions',
];

const ALL_GATES = [
  'server_bootstrap_pairing_v1',
  'module_control_plane_v1',
  'managed_launch_trust_v1',
  'vault_v1',
  'telemetry_v1',
  'storage_control_v1',
  'nats_data_plane_v1',
  'blob_v1',
  'attachment_security_engine_v1',
  'mail_outbound_mime_attachments_v1',
  'client_blob_v1',
  'communications_content_read_v1',
  'communications_saved_search_v1',
  'communications_sender_insights_v1',
  'clock_v1',
  'scheduler_v1',
  'browser_client_v1',
  'client_gateway_v1',
  'whole_instance_backup_v1',
];

const NOT_AUTHORIZED = ALL_GATES.filter((gate) => ![
  'module_control_plane_v1',
  'server_bootstrap_pairing_v1',
  'managed_launch_trust_v1',
  'vault_v1',
  'telemetry_v1',
  'storage_control_v1',
  'nats_data_plane_v1',
  'blob_v1',
  'attachment_security_engine_v1',
  'mail_outbound_mime_attachments_v1',
  'client_blob_v1',
  'communications_content_read_v1',
  'communications_saved_search_v1',
  'communications_sender_insights_v1',
  'clock_v1',
  'scheduler_v1',
  'browser_client_v1',
  'client_gateway_v1',
  'whole_instance_backup_v1',
].includes(gate));

const REQUIRES = {
  server_bootstrap_pairing_v1: [],
  module_control_plane_v1: [],
  managed_launch_trust_v1: [],
  vault_v1: ['managed_launch_trust_v1'],
  telemetry_v1: ['managed_launch_trust_v1'],
  storage_control_v1: ['managed_launch_trust_v1', 'vault_v1'],
  nats_data_plane_v1: [
    'managed_launch_trust_v1',
    'vault_v1',
    'storage_control_v1',
  ],
  blob_v1: ['managed_launch_trust_v1', 'vault_v1'],
  attachment_security_engine_v1: [
    'managed_launch_trust_v1',
    'vault_v1',
    'storage_control_v1',
    'nats_data_plane_v1',
    'blob_v1',
  ],
  mail_outbound_mime_attachments_v1: [
    'managed_launch_trust_v1',
    'vault_v1',
    'storage_control_v1',
    'nats_data_plane_v1',
    'blob_v1',
    'attachment_security_engine_v1',
    'client_gateway_v1',
  ],
  client_blob_v1: [
    'managed_launch_trust_v1',
    'vault_v1',
    'blob_v1',
    'client_gateway_v1',
  ],
  communications_content_read_v1: [
    'managed_launch_trust_v1',
    'vault_v1',
    'storage_control_v1',
    'nats_data_plane_v1',
    'blob_v1',
    'client_gateway_v1',
    'client_blob_v1',
  ],
  communications_saved_search_v1: [
    'communications_content_read_v1',
  ],
  communications_sender_insights_v1: [
    'communications_content_read_v1',
  ],
  clock_v1: ['module_control_plane_v1', 'managed_launch_trust_v1'],
  scheduler_v1: [
    'module_control_plane_v1',
    'managed_launch_trust_v1',
    'vault_v1',
    'telemetry_v1',
    'storage_control_v1',
    'nats_data_plane_v1',
    'clock_v1',
  ],
  browser_client_v1: [
    'module_control_plane_v1',
    'managed_launch_trust_v1',
  ],
  client_gateway_v1: [
    'browser_client_v1',
    'module_control_plane_v1',
    'telemetry_v1',
    'nats_data_plane_v1',
  ],
  whole_instance_backup_v1: [
    'vault_v1',
    'telemetry_v1',
    'storage_control_v1',
    'nats_data_plane_v1',
  ],
};

const CONDITIONAL_REQUIRES = {
  whole_instance_backup_v1: {
    blob_v1: 'when_blob_state_is_enabled',
    scheduler_v1: 'when_scheduler_state_is_enabled',
  },
};

const REQUIRED_DECISION_FIELDS = {
  server_bootstrap_pairing_v1: [
    'deployment_profile_contract',
    'remote_pairing_protocol',
    'one_shot_token_ttl_rate_limit',
    'ephemeral_tls_fingerprint_pinning',
    'linux_signer_vault_separation',
    'pairing_replay_and_second_enrollment_conformance',
  ],
  module_control_plane_v1: [
    'owner_session_conformance',
    'module_descriptor_parser_limits',
    'grant_revoke_epoch_persistence',
    'local_ipc_replay_and_abuse_tests',
    'owner_authorized_mutation_surface',
  ],
  managed_launch_trust_v1: [
    'manifest_binary_encoding',
    'manifest_schema_digest',
    'detached_signature_suite',
    'verification_key_pin_and_rotation',
    'file_release_authority_conformance',
    'toctou_safe_spawn_adapter',
  ],
  vault_v1: [
    'exact_package_inventory',
    'hpke_session_conformance',
    'sqlcipher_and_platform_key_adapter',
    'lease_epoch_expiry_revoke',
    'secret_non_disclosure_tests',
    'backup_restore_classification',
  ],
  telemetry_v1: [
    'exact_package_inventory',
    'private_local_transport',
    'schema_redaction_and_quotas',
    'bounded_retention',
    'collector_failure_isolation',
    'secret_private_content_negative_tests',
  ],
  storage_control_v1: [
    'exact_package_inventory',
    'postgresql_and_pgbouncer_artifacts',
    'role_grant_pool_budget_conformance',
    'migration_ast_admission',
    'vault_credential_fencing',
    'bypass_isolation_evidence',
    'readiness_and_recovery_tests',
  ],
  nats_data_plane_v1: [
    'broker_artifact_version_and_listener',
    'event_hub_adapter_packages',
    'subject_catalog_version',
    'stream_and_consumer_budgets',
    'credential_authority_delivery_rotation_revoke',
    'runtime_and_grant_generation_fencing',
  ],
  whole_instance_backup_v1: [
    'component_inclusion_matrix',
    'quiesce_and_consistency_order',
    'retention_and_encryption',
    'signed_media_manifest',
    'restore_authorization_and_order',
    'generation_epoch_fencing',
    'disposable_full_restore_evidence',
  ],
  blob_v1: [
    'protocol_and_package_topology',
    'opaque_ref_bindings',
    'storage_encryption_and_quotas',
    'retention_and_gc',
    'range_and_path_safety',
    'revoke_and_backup_classification',
  ],
  attachment_security_engine_v1: [
    'exact_package_inventory',
    'event_only_contract_topology',
    'signed_engine_launch_and_settings',
    'owner_local_storage_and_join_replay',
    'evidence_bound_blob_custody',
    'scanner_clean_threat_and_failure_matrix',
    'exact_outbox_nats_communications_cas',
    'source_target_vault_blob_fencing',
    'negative_output_privacy',
    'architecture_srp_cargo_full_backend_gates',
  ],
  mail_outbound_mime_attachments_v1: [
    'exact_package_and_owner_inventory',
    'bounded_generated_attachment_anchor_contract',
    'event_only_canonical_safety_projection',
    'mail_owned_blob_materialization_and_read_range',
    'deterministic_mime_and_provider_adapter_boundary',
    'unsafe_stale_unknown_and_hash_negative_matrix',
    'exact_duplicate_restart_nats_and_provider_ambiguity',
    'smtp_gmail_live_attachment_conformance',
    'negative_output_privacy',
    'architecture_srp_cargo_clippy_full_backend_gates',
  ],
  client_blob_v1: [
    'descriptor_control_store_route_authority',
    'authenticated_bounded_no_query_http_route',
    'owner_runtime_authorization_without_content',
    'digest_bound_one_use_full_blob_read',
    'registration_grant_runtime_revoke_fencing',
    'private_metadata_and_error_non_disclosure',
    'architecture_srp_cargo_clippy_managed_gates',
  ],
  communications_content_read_v1: [
    'exact_content_contract_build_unit',
    'owner_local_current_receipt_authorization',
    'runtime_local_one_use_ticket_lifecycle',
    'read_only_shared_custody_capability',
    'managed_observation_custody_ticket_content_replay',
    'stale_delete_revoke_blob_outage_negative_matrix',
    'generated_frontend_inert_utf8_presentation',
    'provider_fallback_and_blob_metadata_non_disclosure',
    'architecture_srp_cargo_clippy_managed_browser_gates',
  ],
  communications_saved_search_v1: [
    'exact_saved_search_contract_build_unit',
    'owner_local_digest_only_projection',
    'idempotent_create_cas_replace_delete',
    'tombstone_and_mutation_audit',
    'account_scope_and_revision_bound_cursor_fencing',
    'authenticated_gateway_managed_runtime_conformance',
    'generated_frontend_exact_capability_guard',
    'query_plaintext_and_provider_truth_non_disclosure',
    'architecture_srp_cargo_clippy_managed_browser_gates',
  ],
  communications_sender_insights_v1: [
    'exact_sender_insights_contract_build_unit',
    'typed_ingress_actor_and_bounded_display_metadata',
    'atomic_rebuildable_sender_fact_projection',
    'active_incoming_message_only_aggregation',
    'account_scope_count_timestamp_sender_cursor_fencing',
    'authenticated_gateway_managed_runtime_conformance',
    'generated_frontend_exact_capability_guard',
    'provider_locator_content_and_review_truth_non_disclosure',
    'architecture_srp_cargo_clippy_managed_browser_gates',
  ],
  clock_v1: [
    'protocol_and_package_topology',
    'wall_and_monotonic_sources',
    'utc_timezone_dst_semantics',
    'clock_jump_drift_suspend_policy',
    'deadline_timer_contract',
    'deterministic_fake_clock_suite',
  ],
  scheduler_v1: [
    'exact_package_inventory',
    'jobspec_jobkind_and_contract_binding',
    'schedule_run_lease_and_fencing_storage',
    'nats_acceptance_result_and_ack',
    'hot_schedule_reconciliation',
    'retry_idempotency_and_recovery',
    'deterministic_clock_conformance',
    'managed_successor_restart_recovery',
    'revoked_binding_no_resurrection',
    'authenticated_storage_nats_live_conformance',
  ],
  browser_client_v1: [
    'exact_gateway_package_and_local_listener_inventory',
    'browser_device_pairing_and_session_fencing',
    'signed_browser_bootstrap_delivery',
    'same_origin_connect_session_confirmation',
    'same_origin_fetch_and_no_secret_boundary',
    'sse_fail_closed_without_admitted_owner',
    'browser_abuse_privacy_and_redaction_tests',
  ],
  client_gateway_v1: [
    'exact_gateway_package_and_listener_inventory',
    'owner_device_session_authorization',
    'connectrpc_deadline_error_and_receipt_mapping',
    'sse_replay_gap_reset_and_disconnect',
    'http_surface_separation',
    'http2_tls_remote_profile',
    'http3_fallback_and_zero_rtt_conformance',
    'abuse_privacy_and_redaction_tests',
  ],
};

const OWNER_ADMISSION_EXCEPTIONS = [];

function hasExactKeys(value, expectedKeys) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
  const keys = Object.keys(value);
  return keys.length === expectedKeys.length
    && keys.every((key) => expectedKeys.includes(key));
}

function isExactOrderedStringList(value, expected) {
  return Array.isArray(value)
    && value.length === expected.length
    && duplicates(value).length === 0
    && value.every((entry, index) => entry === expected[index]);
}

function isExactRequires(requires) {
  return hasExactKeys(requires, ALL_GATES)
    && ALL_GATES.every((gate) => isExactOrderedStringList(
      requires[gate],
      REQUIRES[gate],
    ));
}

function isExactConditionalRequires(conditionalRequires) {
  const gates = Object.keys(CONDITIONAL_REQUIRES);
  return hasExactKeys(conditionalRequires, gates)
    && gates.every((gate) => {
      const expected = CONDITIONAL_REQUIRES[gate];
      const actual = conditionalRequires[gate];
      return hasExactKeys(actual, Object.keys(expected))
        && Object.entries(expected).every(([dependency, condition]) => (
          actual[dependency] === condition
        ));
    });
}

function isExactOwnerAdmissionExceptions(exceptions) {
  return Array.isArray(exceptions)
    && exceptions.length === OWNER_ADMISSION_EXCEPTIONS.length
    && exceptions.every((exception, index) => {
      const expected = OWNER_ADMISSION_EXCEPTIONS[index];
      return hasExactKeys(exception, ['id', 'adr', 'packages', 'requires'])
        && exception.id === expected.id
        && exception.adr === expected.adr
        && isExactOrderedStringList(exception.packages, expected.packages)
        && isExactOrderedStringList(exception.requires, expected.requires);
    });
}

function isAcyclicGateGraph(requires, conditionalRequires) {
  const gates = new Set(ALL_GATES);
  const edges = new Map(ALL_GATES.map((gate) => [
    gate,
    [
      ...Object.keys(conditionalRequires?.[gate] ?? {}),
      ...Object.values(requires?.[gate] ?? {}),
    ],
  ]));
  if ([...edges.values()].flat().some((dependency) => !gates.has(dependency))) {
    return false;
  }

  const visiting = new Set();
  const visited = new Set();
  function visit(gate) {
    if (visiting.has(gate)) return false;
    if (visited.has(gate)) return true;
    visiting.add(gate);
    if (!edges.get(gate).every(visit)) return false;
    visiting.delete(gate);
    visited.add(gate);
    return true;
  }

  return ALL_GATES.every(visit);
}

export function validatePhaseGatePolicy(policy) {
  const phaseGates = policy?.phaseGates;
  const decisionFields = phaseGates?.requiredDecisionFields;
  const valid = hasExactKeys(phaseGates, PHASE_GATE_KEYS)
    && phaseGates.transitionAuthority === 'adr_policy_and_executable_evidence'
    && isExactOrderedStringList(phaseGates.notAuthorized, NOT_AUTHORIZED)
    && isExactRequires(phaseGates.requires)
    && isExactConditionalRequires(phaseGates.conditionalRequires)
    && isExactOwnerAdmissionExceptions(phaseGates.ownerAdmissionExceptions)
    && isAcyclicGateGraph(phaseGates.requires, phaseGates.conditionalRequires)
    && hasExactKeys(decisionFields, ALL_GATES)
    && ALL_GATES.every((gate) => (
      isExactOrderedStringList(decisionFields[gate], REQUIRED_DECISION_FIELDS[gate])
    ));

  return valid ? [] : [violation(
    'phase_gate_policy',
    'phaseGates',
    'every post-recovery phase must remain unauthorized until its exact ADR-0225 decision and evidence gate is satisfied',
  )];
}
