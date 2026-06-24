# backend-full nextest report

- Generated at: 2026-06-24T10:32:25.510Z
- Source JUnit: `target/nextest/default/junit.xml`
- Total tests: 1229
- Failed tests: 0
- Flaky tests: none detected
- Total time: 4939.526s
- Average time: 4.019s
- p95: 10.717s
- p99: 14.037s

## Slowest tests

1. `hermes-hub-backend::graph_api::search::graph_search_returns_matching_nodes` - 42.169s
2. `hermes-hub-backend::graph_api::neighborhood::graph_neighborhood_caps_depth_one_edges_nodes_and_evidence` - 37.572s
3. `hermes-hub-backend::graph_api::neighborhood::graph_neighborhood_caps_evidence_for_returned_edges` - 37.154s
4. `hermes-hub-backend::graph_api::neighborhood::graph_neighborhood_returns_selected_node_neighbors_edges_and_evidence` - 27.933s
5. `hermes-hub-backend::graph_api::search::graph_nodes_returns_connected_picker_nodes_first` - 27.168s
6. `hermes-hub-backend::graph_api::search::graph_summary_returns_empty_state_for_empty_database` - 26.291s
7. `hermes-hub-backend::calendar_api::event_details::calendar_event_relation_manual_create_path_captures_observation_against_postgres` - 15.924s
8. `hermes-hub-backend::calendar_api::event_details::calendar_event_relations_list_returns_empty` - 15.806s
9. `hermes-hub-backend::calendar_api::event_details::calendar_event_reminders_list_returns_empty` - 15.673s
10. `hermes-hub-backend::v1_communications_attachment_translation::v1_attachment_translation_emits_signal_hub_ai_events_against_postgres` - 14.574s
