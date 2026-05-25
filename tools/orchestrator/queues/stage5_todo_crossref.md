# Stage 5 todo cross-reference (K-22)

Maps orchestrator continuation queue → live [`STAGE5_TODOS`](../../../src/dev/stage5_live_todos.rs).

| Orchestrator task | STAGE5_TODO | File / system |
|-------------------|-------------|---------------|
| `stage5_full_app_exit` | TODO-01 | `stage5_readiness.rs` — runtime readiness fence |
| viewport authority (complete) | TODO-04 | `map_camera.rs` — view authority / MapCameraDesired |
| render sync | TODO-06 | `CommittedVisualSnapshotFence` |
| fire / GPU | TODO-07–11 | `render/extraction/`, `FireVisualFrame` |
| preview / LOD | TODO-12–13 | world preview, LOD bands |

**Live board:** `Stage5LiveTodoBoard` updated by `sync_stage5_todo_board_predicates` when `Stage5ReadinessProfile::FULL_APP`.

**Proof artifact:** `debug_runs/stage5_full_app_live.json` (visual `--test visual`).
