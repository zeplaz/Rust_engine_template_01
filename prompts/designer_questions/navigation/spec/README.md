# Navigation — spec index

**Parent topic docs:** [`../pathfinding_hierarchical_v1.md`](../pathfinding_hierarchical_v1.md)  
**Checklist:** [`../implementation_questions_v1.md`](../implementation_questions_v1.md)  
**Layers:** `prompts/matrix/repo/repo_boundary_matrix_v1.md`

| # | File | Contents |
|:---|:---|:---|
| 00 | [`00_scope.md`](00_scope.md) | Road/rail/ped, LOD scope, MP |
| 01 | [`01_graph_representations.md`](01_graph_representations.md) | Transport graph, fields, weights |
| 02 | [`02_pathfinding_lod_handoff.md`](02_pathfinding_lod_handoff.md) | Hierarchical pathing, stuck upgrade |
| 03 | [`03_client_server_pathing.md`](03_client_server_pathing.md) | Propose/validate, payloads |
| 04 | [`04_validation_perf.md`](04_validation_perf.md) | Caches, rebuild rules, anti-cheat hooks |
| 05 | [`05_integration_tests.md`](05_integration_tests.md) | Determinism, chunk boundary cases |

**Code touchpoints:** `src/systems/navigation/` (e.g. potential-field file named in `implementation_questions_v1.md` §11), `terrain_world/chunks_streaming_v1.md` for chunk seams.
