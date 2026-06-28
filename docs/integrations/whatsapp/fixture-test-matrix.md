# WhatsApp Fixture Test Matrix

Status: current fixture/runtime-safe coverage map.
Date: 2026-06-26.

This matrix tracks the current repository evidence for acceptance criterion 11:

```text
Fixture tests cover every source record and command class.
```

It is intentionally scoped to fixture/runtime-safe coverage. Live provider smoke
validation remains separately documented in
[`live-smoke-checklist.md`](./live-smoke-checklist.md).

Fast static guard coverage:

- `backend/tests/whatsapp_signal_hub.rs` checks that every documented source
  event family has a Signal Hub raw/accepted mapping, a sanitized realtime
  event family and matrix coverage.
- The same test verifies that command completion remains tied to
  provider-observed reconciliation metadata, not provider SDK success alone.
- `backend/tests/communications_architecture_target.rs` guards provider-library
  import confinement and rejects direct WhatsApp runtime dependencies from
  domains, engines and workflows.

## Source record kinds

| Source record kind | Current evidence |
|---|---|
| `whatsapp_message` | `whatsapp_fixture_message_ingestion_refreshes_decision_and_obligation_candidates_against_postgres` |
| `whatsapp_message_update` | `whatsapp_fixture_message_update_reconciles_provider_command_via_observed_event` |
| `whatsapp_message_delete` | `whatsapp_fixture_message_delete_reconciles_provider_command_via_observed_event` |
| `whatsapp_reaction` | `whatsapp_fixture_reaction_reconciles_provider_command_via_observed_event`, `whatsapp_fixture_unreact_reconciles_provider_command_via_observed_event` |
| `whatsapp_receipt` | `whatsapp_fixture_receipt_projects_source_record_and_emits_realtime_event` |
| `whatsapp_dialog` | `whatsapp_fixture_dialog_reconciles_archive_command_via_observed_event`, `whatsapp_fixture_dialog_reconciles_mute_and_mark_unread_commands_via_observed_event`, `whatsapp_fixture_dialog_reconciles_unarchive_unpin_unmute_and_mark_read_commands_via_observed_event` |
| `whatsapp_participant` | `whatsapp_fixture_participant_reconciles_join_and_leave_group_commands_via_observed_event` |
| `whatsapp_media` | `whatsapp_fixture_media_reconciles_send_media_command_via_observed_event`, `whatsapp_fixture_media_reconciles_download_media_command_via_observed_event`, `whatsapp_fixture_media_reconciles_send_voice_note_command_via_observed_event` |
| `whatsapp_status` | `whatsapp_fixture_status_reconciles_publish_status_command_via_observed_event` |
| `whatsapp_status_view` | `whatsapp_fixture_status_view_and_delete_project_source_records_and_emit_realtime_events` |
| `whatsapp_status_delete` | `whatsapp_fixture_status_view_and_delete_project_source_records_and_emit_realtime_events` |
| `whatsapp_presence` | `whatsapp_fixture_presence_projects_source_record_and_emits_realtime_event` |
| `whatsapp_call_metadata` | `whatsapp_fixture_call_projects_source_record_and_emits_realtime_event` |
| `whatsapp_runtime_event` | `whatsapp_fixture_runtime_event_is_captured_as_signal_and_sanitized_realtime_event`, `whatsapp_unknown_runtime_event_defaults_to_degraded_warning_markers` |
| `whatsapp_web_session` | `whatsapp_web_session_metadata_is_account_scoped_against_postgres`, `whatsapp_authorized_session_material_is_stored_in_host_vault_against_postgres` |

## Provider command classes

### Provider-observed reconciliation coverage

| Command class | Current evidence |
|---|---|
| `send_text` | `whatsapp_fixture_message_reconciles_send_text_command_via_observed_event` |
| `reply` | reply branch inside `whatsapp_api_exercises_web_fixture_foundation` |
| `forward` | forward branch inside `whatsapp_api_exercises_web_fixture_foundation` |
| `edit` | `whatsapp_fixture_message_update_reconciles_provider_command_via_observed_event` |
| `delete` | `whatsapp_fixture_message_delete_reconciles_provider_command_via_observed_event` |
| `react` | `whatsapp_fixture_reaction_reconciles_provider_command_via_observed_event` |
| `unreact` | `whatsapp_fixture_unreact_reconciles_provider_command_via_observed_event` |
| `send_media` | `whatsapp_fixture_media_reconciles_send_media_command_via_observed_event` |
| `download_media` | `whatsapp_fixture_media_reconciles_download_media_command_via_observed_event` |
| `send_voice_note` | `whatsapp_fixture_media_reconciles_send_voice_note_command_via_observed_event` |
| `archive` | `whatsapp_fixture_dialog_reconciles_archive_command_via_observed_event` |
| `mute` | `whatsapp_fixture_dialog_reconciles_mute_and_mark_unread_commands_via_observed_event` |
| `mark_unread` | `whatsapp_fixture_dialog_reconciles_mute_and_mark_unread_commands_via_observed_event` |
| `unarchive` | `whatsapp_fixture_dialog_reconciles_unarchive_unpin_unmute_and_mark_read_commands_via_observed_event` |
| `unpin` | `whatsapp_fixture_dialog_reconciles_unarchive_unpin_unmute_and_mark_read_commands_via_observed_event` |
| `unmute` | `whatsapp_fixture_dialog_reconciles_unarchive_unpin_unmute_and_mark_read_commands_via_observed_event` |
| `mark_read` | `whatsapp_fixture_dialog_reconciles_unarchive_unpin_unmute_and_mark_read_commands_via_observed_event` |
| `join_group` | `whatsapp_fixture_participant_reconciles_join_and_leave_group_commands_via_observed_event` |
| `leave_group` | `whatsapp_fixture_participant_reconciles_join_and_leave_group_commands_via_observed_event` |
| `publish_status` | `whatsapp_fixture_status_reconciles_publish_status_command_via_observed_event` |

### Retried background executor coverage

| Command class | Current evidence |
|---|---|
| `send_text` | `whatsapp_background_command_executor_completes_retried_fixture_send_text_command` |
| `reply` | `whatsapp_background_command_executor_completes_retried_fixture_reply_command` |
| `forward` | `whatsapp_background_command_executor_completes_retried_fixture_forward_command` |
| `edit` | `whatsapp_background_command_executor_completes_retried_fixture_edit_command` |
| `delete` | `whatsapp_background_command_executor_completes_retried_fixture_delete_command` |
| `react` | `whatsapp_background_command_executor_completes_retried_fixture_react_command` |
| `unreact` | `whatsapp_background_command_executor_completes_retried_fixture_unreact_command` |
| `send_media` | `whatsapp_background_command_executor_completes_retried_fixture_send_media_command` |
| `download_media` | `whatsapp_background_command_executor_completes_retried_fixture_download_media_command` |
| `send_voice_note` | `whatsapp_background_command_executor_completes_retried_fixture_send_voice_note_command` |
| `archive` / `mute` / `pin` / `mark_unread` | `whatsapp_background_command_executor_completes_retried_fixture_dialog_state_commands` |
| `unarchive` / `unmute` / `unpin` / `mark_read` | `whatsapp_background_command_executor_completes_retried_fixture_inverse_dialog_state_commands` |
| `join_group` / `leave_group` | `whatsapp_background_command_executor_completes_retried_fixture_join_and_leave_group_commands` |
| `publish_status` | `whatsapp_background_command_executor_completes_retried_fixture_publish_status_command` |

## Known gaps

- This matrix tracks fixture/runtime-safe coverage only. It does not prove live
  provider execution.
- Business Cloud runtime behavior remains capability-model and account-shape
  coverage, not official live API execution coverage.
- Exact DB-backed execution of the listed tests may still depend on local
  Docker/Testcontainers health; CI-safe compile coverage remains `--no-run`
  unless a narrower environment-safe test path is added later.
