# Steward / operator queue `v1`

| Queue ID | When | Done when | Status |
|:---|:---|:---|:---|
| **UI-WP-LAYOUT-D07** | Now | D-07 corner inset on map; `ui_wp_layout_d07_green` in `wave_p_live.json` | ☑ code |
| **UI-WP-LAYOUT-002** | Optional | Open World Preview + **Parameters ▸** in sim for live `d04_*` refresh | ☑ lib JSON |
| **S7P-STEWARD-001** | Now | `stage7_play_live.json` + `concrete_chain_e2e.production_green: true` in sim | ☑ writer + optional seed |
| **WATER-W1-OCEAN-001** | Done | `water_ocean_tiles > 0` per [`water_ocean_fixture_request_v1.md`](water_ocean_fixture_request_v1.md) | ☑ 1715 tiles (visual 2026-05-25) |
| **VFX-CAPTURE-001** | Optional | PNGs under `assets/vfx/reference/review_captures/` | ☑ interim from design targets |
| **TRIAGE-VM-09-CODER-B** | Done | `view_representation` ViewManager zoom; witness `dual_writer_pose_violation: false` | ☑ |

## TRIAGE-VM-09-CODER-B

- **Change:** `resolve_world_main_camera_scale` in [`view_representation.rs`](../gui/view_representation.rs) (matches `gpu_particles` slice 1).
- **Verify:** `cargo test -p proc_A_dine01 --lib view_runtime vm09_slice2`
- **Witness:** `debug_runs/infrastructure_view_isolation_live.json` → `vm_09.triage_vm09_coder_b_green`

## UI-WP-LAYOUT-002

- **Spec:** [`prompts/guides/ui/world_preview_d04_slide_sheet_spec_v1.md`](../prompts/guides/ui/world_preview_d04_slide_sheet_spec_v1.md)
- **Code:** `window.rs` (40% map dim, Esc close, sheet width 37.5%), `draw_world_gen_panel` header
- **Verify:** `cargo test -p proc_A_dine01 --lib wave_p_live_proof` · open unified workspace → **Parameters ▸**

## S7P-STEWARD-001

- **Scenario:** [`stage7_play_scenario_v1.md`](stage7_play_scenario_v1.md)
- **Witness:** `debug_runs/stage7_play_live.json` (written in Simulation every 120 frames)
- **Optional seed:** `$env:RUST_ENGINE_STAGE7_PLAY_SEED=1` before run → Portland chain on sim enter
- **Verify:** `concrete_chain_e2e.production_green: true` in `industrial_activation_live.json` and `stage7_play_live.json`

## WATER-W1-OCEAN-001

- **Fixture:** unit tests `water_w1_ocean_001_*` + visual `water_ocean_tiles: 1715`
- **Catalog:** lake-shore + perimeter DEM in `water_surface_visual.rs`

## VFX PNGs (optional)

Interim tactical stills (design-target stand-ins until operator sim capture):

| File | Source |
|:---|:---|
| `fire_tactical_20260524.png` | `elemental_sparks/fire_spark_target_v1.png` |
| `water_river_tactical_20260524.png` | `water/water_surface_target_v1.png` |
| `water_lake_tactical_20260524.png` | `water/water_surface_target_v1.png` |

Replace with in-sim captures per [`assets/vfx/reference/review_captures/README.md`](../assets/vfx/reference/review_captures/README.md) when ready.
