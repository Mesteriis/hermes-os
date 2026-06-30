# backend-full nextest report

- Generated at: 2026-06-30T01:57:12.363Z
- Source JUnit: `target/nextest/default/junit.xml`
- Total tests: 1441
- Failed tests: 0
- Flaky tests: none detected
- Total time: 6715.71s
- Average time: 4.66s
- p95: 11.699s
- p99: 16.011s

## Slowest tests

1. `hermes-hub-backend::graph_api::search::graph_summary_returns_empty_state_for_empty_database` - 35.663s
2. `hermes-hub-backend::graph_api::neighborhood::graph_neighborhood_caps_depth_one_edges_nodes_and_evidence` - 35.629s
3. `hermes-hub-backend::graph_api::neighborhood::graph_neighborhood_caps_evidence_for_returned_edges` - 35.489s
4. `hermes-hub-backend::graph_api::neighborhood::graph_neighborhood_returns_selected_node_neighbors_edges_and_evidence` - 34.577s
5. `hermes-hub-backend::graph_api::search::graph_nodes_returns_connected_picker_nodes_first` - 33.346s
6. `hermes-hub-backend::v1_communications_api::v1_message_smart_cc_404` - 27.156s
7. `hermes-hub-backend::v1_communications_api::v1_messages_list` - 25.751s
8. `hermes-hub-backend::v1_communications_api::v1_message_states` - 22.157s
9. `hermes-hub-backend::graph_api::search::graph_search_returns_matching_nodes` - 20.894s
10. `hermes-hub-backend::v1_communications_api::v1_threads_list` - 20.486s
