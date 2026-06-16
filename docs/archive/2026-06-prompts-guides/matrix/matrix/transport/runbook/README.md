# Transport — implementation runbooks

Anchor matrix: [`../road_rail_migration_matrix_v1.md`](../road_rail_migration_matrix_v1.md).

| Pack | Intent |
|:---|:---|
| [`r9_authoring_bake_order_steps_v1.md`](r9_authoring_bake_order_steps_v1.md) | **R9**: authoring order vs bad lexicographic bake; splines + G4 snapshot alignment |
| [`../../gap_remediation/runbook/g4_transport_r8_network_slice_steps_v1.md`](../../gap_remediation/runbook/g4_transport_r8_network_slice_steps_v1.md) | **G4** + **R8**: persisting `TransportNetworkSnapshot` / hydrate boundary |

Code: `src/systems/transport/`, `src/gui/editor/map_editor/mod.rs`.
