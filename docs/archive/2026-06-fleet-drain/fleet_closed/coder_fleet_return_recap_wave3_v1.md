# Coder fleet recap — dual queue closure (2026-05-26)

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Closure** | [`coder_dual_queue_todos_v1.md`](coder_dual_queue_todos_v1.md) — 28 IDs |
| **Open wave** | [`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md) — 31 coder rows |
| **Verify** | `cargo test -p proc_A_dine01 --lib coder_a_dual_queue coder_b_queue_bundle` |

---

## What landed (summary)

Two lib bundles refreshed witnesses and asserted exit criteria — **not** full product sign-off for every row.

| Bundle | Tests | Scope |
|:---|:---|:---|
| [`coder_a_dual_queue_closure_v1.rs`](coder_a_dual_queue_closure_v1.rs) | `coder_a_dual_queue_14_closure_bundle` | 14 @coder A rows |
| [`coder_b_queue_bundle_proof.rs`](coder_b_queue_bundle_proof.rs) | `coder_b_queue_bundle_001_lib_refresh` + VM-09 audit | 14 @coder B rows |

---

## @coder A — what was done (14)

| ID | What landed | Proof / notes |
|:---|:---|:---|
| **FIRE7-F7-A-001** | Infra witness block `fire7_f7_a_001` + `f7_a_per_view_extract_bounded` | **Witness bundle only** — product exit is **FIRE7-F7-A-EXIT-001** (wave 3) |
| **P2-FIRE-SPARK-010** | Sparks above smoke in tactical VFX witness | `fire_sparks_above_smoke` in stage5 JSON |
| **P2-FIRE-SPARK-011** | Spark compute @ tactical zoom | `fire_spark_011_green` |
| **P2-WATER-POLISH-001** | River + ocean read fields | `water_w1_river_read_green` |
| **UX-E03-CODER-A** | `TransmissionMediaProviderRegistry` on sim enter | `ux_e03_coder_a` in shell JSON |
| **S7B-M4-PLAY-001** | M4 playtest witness writer | `s7b_m4_play_001` in stage7 JSON |
| **INFRA-GPU-TILE-001** | Instanced dispatch readiness | `instanced_dispatch_ok` / draw path |
| **UI-WP-PIPELINE** | Wave P witness rows | `wave_p_live.json` |
| **UI-WP-L4-001** | Raster look witness | `ui_wp_l4_001_green` |
| **UI-WP-MOTION-001** | Motion table witness | `ui_wp_motion_001_green` |
| **UI-WP-LAYOUT-003** | Layout-003 witness | `ui_wp_layout_003_green` |
| **P4-VEH-01** | Vehicle chips in shell | `p4_veh_01.green` |
| **INFRA-PERF-001** | Qualified via WC-D04 / frame budget | not full OPS-F01 60s |
| **S7B-TUNE-DELAY-001** | `dispatch_delay_ticks` witness | `s7b_tune_delay_001_green` |

**Fixes bundled along the way:** `ProjectionNodeTrait` import, tactical VFX fixture (`logistics` overlay + ocean tiles + `all_green_for_visual_proof`), LOG-E01 projection graph fixture.

---

## @coder B — what was done (14)

| ID | What landed | Proof / notes |
|:---|:---|:---|
| **LOG-E01-WITNESS** | Logistics rollup in stage5 refresh | `logistics_active_rows > 0` via lib fixture |
| **IND-E02-DEFAULT** | Lib refresh path for `ind_e02_green` | default **play** path still open → **IND-E02-DEFAULT-PLAY-001** |
| **P2-VFX-WITNESS-001** | Tactical VFX merged refresh | stage5 tactical block |
| **P2-WATER-WITNESS-002** | Water fields @ tactical zoom | same refresh |
| **INFRA-VM10-001** | Minimap lockstep in infra JSON | `infrastructure_view_isolation_live.json` |
| **INFRA-VM11-001** | Preview vs main audit fields | same |
| **INFRA-VM09-STRAY-001** | Stray `MapCameraDesired` audit green | lib test in bundle |
| **WITNESS-SHELL-P4** | `icon_atlas_loaded` + P2A tail | `refresh_ui_p2a_001_live_witness` |
| **UI-P2A-WITNESS-TAIL** | f03 + p4 auth greens | shell JSON |
| **WAVE-P-WITNESS** | wave_p layout greens | `ui_wp_layout_d02_opt_green` |
| **WAVE-C-WITNESS** | wave_c green | `applied_chunks: 2` in tile report |
| **UI-WP-LAYOUT-D02-OPT** | D-02 dominance witness | wave_p |
| **CONSTRUCTION-MV-001** | **Qualified** — profile/operational gate only | full MV → **CONSTRUCTION-MV-SIM-001** |
| **S7P-GRID-UX-001** | **Qualified** — witness fields only | in-game UI → **S7P-GRID-UX-UI-001** |

**Orchestrator refresh:** `refresh_coder_b_queue_bundle_live_witnesses()` + agent debug index.

---

## What is **not** done (honest gaps)

| Gap | Owner next |
|:---|:---|
| ~~F7-A/B/C product depth~~ | **Done** @coder A wave 3 — `coder_a_wave3_closure_v1` · `fire_streaming_live.json` · `fire7_f7_c_001_green` |
| `--test visual` sign-off (VFX, WP, LOG) | Coder A/B visual rows + operator refresh |
| Grid overload **in sim** toast/ops strip | Coder B **S7P-GRID-UX-UI-001** (design **SIGNED**) |
| Construction multiview in **sim** | Coder B **CONSTRUCTION-MV-SIM-001** |
| `ind_e02_green` without seed env | Coder B **IND-E02-DEFAULT-PLAY-001** |
| Minimap M3 units / replay | Coder B **UI-P3-M3-*** + designer specs |
| OPS-F01 60s perf | Operator |

---

## Witness files touched

`stage5_full_app_live.json` · `infrastructure_view_isolation_live.json` · `industrial_activation_live.json` · `ui_shell_migration_live.json` · `wave_p_live.json` · `wave_c_live.json` · `construction_stage_live.json` · `stage7_behavioral_live.json` · `stage6_virtualization_live.json` (WC-D04) · `agent_debug_index.json`

---

## Handoff to planner / designer

| Lane | Doc |
|:---|:---|
| Planner wave 4 | [`planner_wave4_todos_v1.md`](planner_wave4_todos_v1.md) |
| Designer wave 4 | [`designer_wave4_todos_v1.md`](designer_wave4_todos_v1.md) |
| Coder wave 3 (continue) | [`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md) |
