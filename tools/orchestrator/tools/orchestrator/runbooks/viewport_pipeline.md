# Viewport pipeline runbook

## STATUS: migration COMPLETE (2026-05-20)

Witness: `debug_runs/viewport_authority_migration_witness.json`

## File map

| Stage | File | Symbol / resource |
|-------|------|-------------------|
| Measure | `src/gui/authoritative_viewport.rs` | `measure_sim_map_fill_viewport` |
| Semantic | `src/gui/viewport_layout_solver.rs` | `semantic_viewport_from_map_fill` |
| Commit | `src/gui/viewport_layout_solver.rs` | `commit_authority_from_semantic` |
| Rescue floor | `src/gui/viewport_layout_solver.rs` | `viewport_rescue_floor` |
| Debug trace | `src/gui/hud/viewport_authority_debug.rs` | `trace_viewport_authority` |
| Sync trace | `src/gui/hud/sim_view_sync_debug.rs` | `trace_sim_view_sync_state` |
| Camera | `src/gui/map_camera.rs` | `MainWorldCamera`, `MapCameraDesired` |
| Render | `src/render/` | `ResolvedViewports` |

## Authoritative path

1. `sim_map_fill` UI measure → `SemanticViewportRect`
2. `commit_authority_from_semantic` → `AuthoritativeViewport` / `SimulationMapViewport`
3. Camera + render copy — **no** window-chrome re-derive

## Drift reproduction

```text
SIM_VIEW_SYNC_DEBUG=1
STAGE5_VERBOSE=1
--debug-sim-view-sync
RUST_LOG=sim_view_sync=info,sim_view_sync::anomaly=warn
```

## Staging (do not delete)

- `frozen_exceeds_semantic_authority` — heal hud_root overshoot (IN_PROGRESS)
- `sim_view_sync_debug` imports — instrumentation expansion

## Agent rules

| Action | Allowed |
|--------|---------|
| Change semantic solver | only with viewport_migration_agent |
| Delete frozen/rescue helpers | **no** without witness update |
| Visibility tighten | yes (`pub(crate)`) |
