# backend-full nextest report

- Generated at: 2026-06-28T00:18:29.545Z
- Source JUnit: `target/nextest/default/junit.xml`
- Total tests: 1357
- Failed tests: 0
- Flaky tests: 3
- Total time: 5663.595s
- Average time: 4.174s
- p95: 12.558s
- p99: 19.029s

## Slowest tests

1. `hermes-hub-backend::graph_api::neighborhood::graph_neighborhood_caps_depth_one_edges_nodes_and_evidence` - 30.099s
2. `hermes-hub-backend::graph_api::search::graph_summary_returns_empty_state_for_empty_database` - 29.562s
3. `hermes-hub-backend::graph_api::search::graph_search_returns_matching_nodes` - 29.562s
4. `hermes-hub-backend::graph_api::neighborhood::graph_neighborhood_caps_evidence_for_returned_edges` - 27.092s
5. `hermes-hub-backend::graph_api::neighborhood::graph_neighborhood_returns_selected_node_neighbors_edges_and_evidence` - 25.228s
6. `hermes-hub-backend::tasks_api::mutations::task_post_provider` - 23.678s
7. `hermes-hub-backend::v1_communications_regressions::messages_threads::v1_extract_tasks_skips_llm_candidates_when_ai_source_is_muted` - 22.442s
8. `hermes-hub-backend::graph_api::search::graph_nodes_returns_connected_picker_nodes_first` - 22.301s
9. `hermes-hub-backend::persons_api::dossier_owner::person_owner_get_and_put_uses_owner_persona_against_postgres` - 21.019s
10. `hermes-hub-backend::whatsapp::whatsapp_background_command_executor_completes_retried_fixture_join_and_leave_group_commands` - 20.302s
