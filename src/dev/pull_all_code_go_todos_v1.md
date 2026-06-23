# Pull-all-code-go — active todo board `v1`

| Field | Value |
|:---|:---|
| **Ritual** | sync → test → mark witness-green → resync |
| **Date** | 2026-06-11 |
| **Owner** | `@coder` / Auto session |

## Ritual (every pull)

```powershell
python tools/orchestrator/scripts/sync_dispatch_subqueues.py
cargo test -p proc_A_dine01 --lib power_grid
cargo test -p proc_A_dine01 --lib power_map_overlay
cargo test -p proc_A_dine01 --lib minimap_topology
cargo test -p proc_A_dine01 --lib veg_ship
cargo test -p proc_A_dine01 --lib fire_ecology
cargo test -p proc_A_dine01 --lib power_node_hover
python tools/orchestrator/scripts/mark_pick_done_from_witness.py
python tools/orchestrator/scripts/sync_dispatch_subqueues.py
python tools/orchestrator/scripts/scan_queues_hub.py
```

## Drain pass 2026-06-11

| Task | Owner | Action | Witness | Result |
|:---|:---|:---|:---|:---|
| COD-POWER-ISLAND-HIGHLIGHT-001 | coder | witness refresh + mark done | `power_grid_track_bd_live.json` | **done** |
| COD-POWER-TOOL-RAIL-001 | coder | witness refresh + mark done | `power_grid_track_bd_live.json` | **done** |
| COD-UTILITY-ACTIVATION-LINK-001 | coder | witness refresh + mark done | `power_grid_track_bd_live.json` | **done** |
| COD-POWER-DAMAGE-SEGMENT-001 | coder | witness refresh + mark done | `power_grid_track_c_live.json` | **done** |
| COD-POWER-OVERLAY-RENDER-001 | coder_b | witness refresh + mark done | `power_map_overlay_live.json` | **done** |
| CMCP-GRAM-SWEEP-PROCESS-001 | coder-mcp | witness refresh + mark done | `grammar_sweep_process_live.json` | **done** |
| CDR-B-VEG-MINIMAP-LEGEND-UI-001 | coder_b | witness refresh + mark done | `minimap_topology_legend_live.json` | **done** |
| DES-POWER-NODE-HOVER-001 | designer | spec PASS + mark done | `design_power_node_hover_v1.md` | **done** |
| VEG-F01-ATLAS-SHIP-001 | coder_a | pilot close witness | `veg_ship_close_live.json` | **done** (pilot scope; G4 art-ship still operator) |
| SIM-STEWARD-FIRE-REGRESS-001 | sim-steward | fire ecology lib tests | `fire_ecology_live.json` | **done** |
| COD-POWER-NODE-HOVER-001 | coder | already shipped | `power_node_hover_live.json` | **done** (pre-existing) |

**27 queue rows** reconciled via `mark_pick_done_from_witness.py`.

## Still open (code work)

| Task | Owner | Priority | Next action |
|:---|:---|:---|:---|
| **PERF-INSTR-VFX-002** | coder_b | P1 | Phase 2A–2D dirty gates landed; run `cargo run … -- --test vfx` for acceptance witness |
| VEG G4 art-ship | operator / designer-mcp | blocked | `landscape_expanded_g4_signoff.yaml` — not engine code |

## Acceptance (PERF lane)

```text
cargo run -p proc_A_dine01 --release -- --test vfx
cargo test -p proc_A_dine01 --lib stage5
validate-report cargo
witness: debug_runs/triage_perf_vfx_fix_2026-06-11_live.json
  steady_wall_p50_ms < 33
  slice_p50_ms < 5
```

Current witness: Phase 2A–2D code landed. Re-run `cargo run -p proc_A_dine01 --release -- --test vfx` (interactive; flushes analytics every ~5s) for acceptance.
