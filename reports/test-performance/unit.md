# unit nextest report

- Generated at: 2026-06-23T20:05:37.037Z
- Source JUnit: `target/nextest/default/junit.xml`
- Total tests: 261
- Failed tests: 0
- Flaky tests: none detected
- Total time: 166.72s
- Average time: 0.639s
- p95: 6.133s
- p99: 7.19s

## Slowest tests

1. `hermes-hub-backend::integrations::telegram::runtime::manager::chat_events::tests::archive_reconciliation::publish_chat_position_event_reconciles_archive_command_when_provider_chat_is_archived` - 7.203s
2. `hermes-hub-backend::integrations::telegram::runtime::manager::chat_events::tests::archive_reconciliation::publish_chat_position_event_reconciles_unarchive_command_when_provider_chat_is_unarchived` - 7.198s
3. `hermes-hub-backend::integrations::telegram::runtime::manager::chat_events::tests::mark_unread_reconciliation::publish_chat_marked_as_unread_event_marks_mark_unread_as_mismatch_when_provider_disagrees` - 7.19s
4. `hermes-hub-backend::integrations::telegram::client::participants::tests::marks_stale_tdlib_participants_as_absent_from_exhaustive_roster` - 7.068s
5. `hermes-hub-backend::integrations::telegram::runtime::manager::chat_events::tests::mark_unread_reconciliation::publish_chat_marked_as_unread_event_reconciles_mark_unread_command_and_emits_events` - 7.007s
6. `hermes-hub-backend::integrations::telegram::client::reactions::tests::provider_state_sync_deactivates_absent_self_reactions` - 6.885s
7. `hermes-hub-backend::integrations::telegram::runtime::manager::chat_events::tests::archive_reconciliation::publish_chat_position_event_marks_unarchive_command_as_mismatch_when_provider_disagrees` - 6.808s
8. `hermes-hub-backend::integrations::telegram::runtime::manager::chat_events::tests::archive_reconciliation::publish_chat_position_event_marks_archive_command_as_mismatch_when_provider_disagrees` - 6.806s
9. `hermes-hub-backend::integrations::telegram::runtime::manager::chat_events::tests::publish_chat_folders_event_emits_chat_updated_for_folder_label_projection_changes` - 6.334s
10. `hermes-hub-backend::integrations::telegram::runtime::manager::chat_events::tests::publish_chat_folders_event_refreshes_unknown_labels_when_folder_snapshot_disappears` - 6.333s
