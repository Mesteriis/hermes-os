# backend-full nextest report

- Generated at: 2026-06-23T22:28:02.538Z
- Source JUnit: `target/nextest/default/junit.xml`
- Total tests: 1223
- Failed tests: 0
- Flaky tests: none detected
- Total time: 4535.109s
- Average time: 3.708s
- p95: 10.325s
- p99: 14.988s

## Slowest tests

1. `hermes-hub-backend::graph_api::neighborhood::graph_neighborhood_caps_depth_one_edges_nodes_and_evidence` - 35.308s
2. `hermes-hub-backend::graph_api::neighborhood::graph_neighborhood_caps_evidence_for_returned_edges` - 35.23s
3. `hermes-hub-backend::graph_api::search::graph_summary_returns_empty_state_for_empty_database` - 34.647s
4. `hermes-hub-backend::graph_api::neighborhood::graph_neighborhood_returns_selected_node_neighbors_edges_and_evidence` - 32.185s
5. `hermes-hub-backend::graph_api::search::graph_nodes_returns_connected_picker_nodes_first` - 31.815s
6. `hermes-hub-backend::signal_hub::signal_hub_profile_application_sets_active_profile_and_managed_policies` - 19.976s
7. `hermes-hub-backend::signal_hub::signal_hub_person_derived_evidence_projection_replay_rebuilds_relationships` - 19.949s
8. `hermes-hub-backend::signal_hub::signal_hub_pause_policy_buffers_raw_signal_without_accepted_publication` - 18.985s
9. `hermes-hub-backend::graph_api::search::graph_search_returns_matching_nodes` - 18.212s
10. `hermes-hub-backend::signal_hub::signal_hub_health_check_updates_durable_health_state` - 16.519s
