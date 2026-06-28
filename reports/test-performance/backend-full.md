# backend-full nextest report

- Generated at: 2026-06-28T20:41:24.982Z
- Source JUnit: `target/nextest/default/junit.xml`
- Total tests: 1401
- Failed tests: 0
- Flaky tests: none detected
- Total time: 9252.439s
- Average time: 6.604s
- p95: 16.809s
- p99: 33.486s

## Slowest tests

1. `hermes-hub-backend::ai::semantic_store::pgvector_semantic_store_indexes_and_searches_sources_against_postgres` - 65.207s
2. `hermes-hub-backend::ai_control_center::ai_control_center_mutations_record_observation_trail_against_postgres` - 63.502s
3. `hermes-hub-backend::ai_control_center::api_provider_create_with_locked_host_vault_does_not_leave_provider_row` - 63.224s
4. `hermes-hub-backend::ai_control_center::non_api_provider_consent_mutation_is_rejected` - 62.27s
5. `hermes-hub-backend::ai::agents::ai_agents_api_materializes_agent_personas_against_postgres` - 44.552s
6. `hermes-hub-backend::integrations::telegram::runtime::manager::realtime_events::tests::telegram_runtime_event_bridge_skips_broadcast_when_runtime_paused` - 44.418s
7. `hermes-hub-backend::integrations::telegram::runtime::manager::topic_events::tests::publish_topic_event_reconciles_topic_close_and_appends_runtime_events` - 43.994s
8. `hermes-hub-backend::integrations::telegram::runtime::manager::realtime_events::typing_tests::publish_command_reconciled_events_appends_status_and_reconciled_records` - 43.694s
9. `hermes-hub-backend::ai::agents::ai_meeting_prep_returns_briefing_without_calendar_dependency` - 40.253s
10. `hermes-hub-backend::ai::answers::ai_answer_api_returns_source_backed_answer_and_persists_run` - 40.149s
