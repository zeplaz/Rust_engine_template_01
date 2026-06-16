# Designer todos — sign-off board `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.5.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@designer` |
| **Orchestrator registry** | [`tools/orchestrator/queues/designer_signoff_registry.json`](../../tools/orchestrator/queues/designer_signoff_registry.json) |
| **Machine queue** | [`tools/orchestrator/queues/designer_active_queue.json`](../../tools/orchestrator/queues/designer_active_queue.json) |
| **Workboard** | [`stage_designer_workboard_v1.md`](stage_designer_workboard_v1.md) |

**Rule:** Each row below is **DONE** with **SIGNED** on witness doc — orchestrator must not re-queue unless witness is revoked.

---

## @designer UI4-DESIGN-001

| Field | Value |
|:---|:---|
| **Status** | **DONE — SIGNED** |
| **Signed** | 2026-05-24 |
| **Witness** | [`world_preview_d04_slide_sheet_spec_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/world_preview_d04_slide_sheet_spec_v1.md) |
| **Artifact** | [`slide_sheet_spec_v1.png`](../assets/ui/world_preview/slide_sheet_spec_v1.png) |
| **Unblocks** | **UI-WP-LAYOUT-002** (**DONE**) |

---

## @designer S7P-DESIGN-001

| Field | Value |
|:---|:---|
| **Status** | **DONE — SIGNED** |
| **Signed** | 2026-05-24 |
| **Witness** | [`stage7_play_scenario_v1.md`](stage7_play_scenario_v1.md) |
| **Unblocks** | **S7P-STEWARD-001** (**DONE** — `stage7_play_live.json`) |

---

## @designer UI-OH-P4-ART-001

| Field | Value |
|:---|:---|
| **Status** | **DONE — SIGNED — PASS** |
| **Signed** | 2026-05-25 |
| **Prereq** | **PLAN-UI-P4-ATLAS-001** · Phase 4.1 code |
| **Brief** | [`ui_phase4_icon_atlas_brief_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase4_icon_atlas_brief_v1.md) |
| **Witness record** | [`ui_oh_p4_art_signoff_record_v1.md`](ui_oh_p4_art_signoff_record_v1.md) |
| **Atlas** | `assets/textures/ui/icon_atlas_phase4_v1.png` |
| **Re-bake** | `python tools/orchestrator/scripts/bake_icon_atlas_phase4.py` |
| **Unblocks** | **P4-VEH-01** · optional **P4-F03** |

### Copy-paste — @designer

```
Queue: UI-OH-P4-ART-001
Read: docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase4_icon_atlas_brief_v1.md §4–§7
Bake: python tools/orchestrator/scripts/bake_icon_atlas_phase4.py
Test: cargo test -p proc_A_dine01 --lib icon_atlas
Sign: docs/archive/2026-06-src-dev/plans/ui_oh_p4_art_signoff_record_v1.md §11 SIGNED — PASS
```

---

## @designer UI-OH-D2-SIGN-001

| Field | Value |
|:---|:---|
| **Status** | **DONE — SIGNED — PASS** |
| **Signed** | 2026-05-25 |
| **Prereq coder** | **UI-OH-2A-001** + **UI-P2A-CODER-B** — `ui_oh_2a_001.green` · `ui_p2a_coder_b.green` · `mock_zone_parity` |
| **Witness record** | [`ui_oh_d2_signoff_record_v1.md`](ui_oh_d2_signoff_record_v1.md) |
| **Checklist** | [`ui_phase2_designer_signoff_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase2_designer_signoff_v1.md) v2.2.0 |
| **Witness JSON** | `debug_runs/ui_shell_migration_live.json` — lib refresh 2026-05-25 (`1779748960`) |
| **Pass scope** | P1–P3 mock parity · P4 **2C-B** dual column · §1.6 interactions |
| **Deferred** | Phase 5 pause · World Preview D-WP |

### Copy-paste — @designer

```
Queue: UI-OH-D2-SIGN-001
Prereq: UI-OH-2A-001 + mock_zone_parity — cargo test -p proc_A_dine01 --lib ui_oh_2a_001_live_witness_refresh
Read: docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase0_panel_mocks_v1.md
      docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase2_designer_signoff_v1.md
      docs/archive/2026-06-src-dev/plans/ui_oh_d2_signoff_record_v1.md
Witness: debug_runs/ui_shell_migration_live.json (ui_oh_2a_001.green, ui_p2a_coder_b.green)
Sign: ui_oh_d2_signoff_record_v1.md §11 SIGNED — PASS
Do NOT: reopen 2C layout; conflate D-WP
```

---

## @designer DESIGN-UI-P2-SIGNOFF-001

| Field | Value |
|:---|:---|
| **Canonical queue ID** | `DESIGN-UI-P2-SIGNOFF` (alias **DESIGN-UI-P2-SIGNOFF-001**) |
| **Status** | **DONE — SIGNED** |
| **Signed** | 2026-05-24 |
| **Witness** | [`ui_phase2_designer_signoff_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase2_designer_signoff_v1.md) **v2.2.0** |
| **Unblocks** | — (historical gate; no rework) |

---

## @designer DESIGN-D-VFX-POST-001

| Field | Value |
|:---|:---|
| **Status** | **DONE — SIGNED — PASS** |
| **Depends on** | **P2-FIRE-SPARK-011** (@coder **A**) — **DONE** |
| **Also needs** | **P2-VFX-VISUAL-001** green witness |
| **Signed** | 2026-05-25 |
| **Brief (full prompts)** | [`vfx_post_implementation_review_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/vfx_post_implementation_review_v1.md) |
| **Witness record** | [`vfx_design_review_record_v1.md`](vfx_design_review_record_v1.md) (**D-VFX**) |
| **Track closure** | [`fire_spark_track_closure_plan_v1.md`](fire_spark_track_closure_plan_v1.md) (**PLAN-FIRE-VFX-CLOSURE-001**) |
| **Water channel** | [`water_vfx_review_record_v1.md`](water_vfx_review_record_v1.md) (**WATER-DESIGN-001**) |
| **Witness JSON** | `debug_runs/stage5_full_app_live.json` — `tactical_vfx_witness.all_green: true`; **§12** reconfirmed 2026-05-25 after **STEWARD-SPARK-VFX-001** + Coder **A** **P2-FIRE-SPARK-011** |
| **Steward** | [`steward_spark_vfx_gate_v1.md`](steward_spark_vfx_gate_v1.md) **GO (qualified)** |
| **Captures (sibling)** | **DESIGN-VFX-CAPTURE-001** — [`vfx_capture_status_20260525.md`](../assets/vfx/reference/review_captures/vfx_capture_status_20260525.md) |
| **Non-blocking** | `fire_instance_buffer_rows: 0` (overlay bootstrap) — **F-T02** |

### Copy-paste — @designer

```
Queue: DESIGN-D-VFX-POST-001
Prereq: @coder A P2-FIRE-SPARK-011 DONE (fire_spark_011_green @ zoom 0.85)
Read: docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/vfx_post_implementation_review_v1.md
      docs/archive/2026-06-src-dev/plans/vfx_design_review_record_v1.md
      docs/archive/2026-06-src-dev/plans/water_vfx_review_record_v1.md
Witness: debug_runs/stage5_full_app_live.json → tactical_vfx_witness.all_green
Mocks: assets/vfx/reference/elemental_sparks/fire_spark_target_v1.png
       assets/vfx/reference/water/water_surface_target_v1.png
Captures: assets/vfx/reference/review_captures/*_tactical_20260524.png (VFX-CAPTURE-001)
Sign: vfx_design_review_record_v1.md § Overall — SIGNED PASS
Do NOT: re-run unless major shader re-baseline
```

---

## @designer DESIGN-VFX-CAPTURE-001

| Field | Value |
|:---|:---|
| **Canonical queue ID** | `VFX-CAPTURE-001` (alias **DESIGN-VFX-CAPTURE-001**) |
| **Status** | **DONE — SIGNED** (sub-deliverable of **DESIGN-D-VFX-POST-001**) |
| **Signed** | 2026-05-25 |
| **Witness** | [`vfx_capture_status_20260525.md`](../assets/vfx/reference/review_captures/vfx_capture_status_20260525.md) |
| **Captures** | `fire_tactical_20260524.png` · `water_river_tactical_20260524.png` · `water_lake_tactical_20260524.png` |

---

## @designer WATER-DESIGN-002

| Field | Value |
|:---|:---|
| **Status** | **DONE — SIGNED** |
| **Signed** | 2026-05-24 |
| **Witness** | [`water_ocean_fixture_request_v1.md`](water_ocean_fixture_request_v1.md) |
| **Fixture test** | `water_w1_ocean_001_dem_deep_band_fills_ocean_tiles` |
| **Unblocks** | **WATER-W1-OCEAN-001** (**DONE**) |

---

## @designer DESIGN-MINIMAP-M2-001

| Field | Value |
|:---|:---|
| **Canonical track** | **D-MINIMAP-M2** (alias **DESIGN-MINIMAP-M2-001**) |
| **Status** | **DONE — SIGNED — M2 COMPLETE** |
| **Signed** | 2026-05-24 |
| **Witness** | [`minimap_d_m2_signoff_v1.md`](minimap_d_m2_signoff_v1.md) |
| **Live JSON** | `debug_runs/minimap_compositor_live.json` — logistics / construction / ecology rows |
| **Deferred** | **UI-P3-M2-TRAY-OPT** (optional coder — tray → `MinimapOverlayMask`) |

---

## @designer DESIGN-D-WP-REVIEW-001

| Field | Value |
|:---|:---|
| **Status** | **DONE — SIGNED — PASS** |
| **Signed** | 2026-05-25 |
| **Brief (full prompts)** | [`world_preview_d_wp_post_impl_review_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/world_preview_d_wp_post_impl_review_v1.md) |
| **Witness record** | [`world_preview_d_wp_review_record_v1.md`](world_preview_d_wp_review_record_v1.md) |
| **Track rollup** | [`world_preview_d_wp_track_signoff_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/world_preview_d_wp_track_signoff_v1.md) v1.1 |
| **Witness JSON** | `debug_runs/wave_p_live.json` — **COD-B-WP-WITNESS-001** lib refresh 2026-05-25 (`written_at_epoch_secs` **1779725532**); review **§12** reconfirmed |
| **Prereq coder** | **COD-B-WP-WITNESS-001** **DONE** — `cod_b_wp_witness_001_green` · `ui_wp_layout_002_green` · `ui_wp_layout_d07_green` · `wave_p_green` |
| **Pass scope** | D-01 unified workspace · D-04 sheet + dim · D-07 corner inset |
| **Deferred** | D-05, D-08…D-12, WP-L4, optional D-02 |

### Copy-paste — @designer

```
Queue: DESIGN-D-WP-REVIEW-001
Read: docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/world_preview_d_wp_post_impl_review_v1.md
      docs/archive/2026-06-src-dev/plans/world_preview_d_wp_review_record_v1.md
Prereq: COD-B-WP-WITNESS-001 — cargo test -p proc_A_dine01 --lib ui_wp_layout_002_writes_wave_p_live_json
Witness: debug_runs/wave_p_live.json (ui_wp_layout_002_green, ui_wp_layout_d07_green)
Manual: F8 → Parameters sheet → corner minimap on map (optional sim re-write)
Sign: world_preview_d_wp_review_record_v1.md §11 SIGNED — PASS
Do NOT: reopen worksheet D-* choices; render_raster / GenerateWorldEvent
```

---

## @designer UX-E02-BQ128-001

| Field | Value |
|:---|:---|
| **Status** | **DONE — SIGNED** |
| **Signed** | 2026-05-25 |
| **Deliverable** | [`bq128_editor_path_design_note_v1.md`](bq128_editor_path_design_note_v1.md) |
| **Planner plan** | [`bq128_editor_path_plan_v1.md`](bq128_editor_path_plan_v1.md) (**PLAN-UX-BQ128-001**) |
| **Depends on** | **None** (parallel OK) |
| **Backlog** | **BQ-128** · Phase E **UX-E02** |
| **Unblocks** | **BQ-128-APPLY-001** (coder preset picker + apply-to-ghost) |

### Copy-paste — @designer

```
Queue: UX-E02-BQ128-001
Read: experience_layer_ux_hud_designer_brief_v1.md §4
      wave_s_open.md · construction_invariants.md
Deliverable: docs/archive/2026-06-src-dev/plans/bq128_editor_path_design_note_v1.md
Scope: Editor-only path for blueprints/presets.ron — sim Pending blueprints panel;
       bundle hydrate/import/export/capture. No sim authority writes.
Witness: debug_runs/wave_s_blueprint_roundtrip.json
Do NOT: WorldGen shell, map-editor book, instant-build from preset
```

---

## @designer S7B-DESIGN-001

| Field | Value |
|:---|:---|
| **Status** | **DONE — SIGNED** |
| **Signed** | 2026-05-25 |
| **Worksheet** | [`stage7_behavioral_decision_worksheet_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/stage7_behavioral_decision_worksheet_v1.md) |
| **Sign-off** | [`stage7_behavioral_d_signoff_v1.md`](stage7_behavioral_d_signoff_v1.md) |
| **Brief** | [`stage7_behavioral_world_designer_brief_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/stage7_behavioral_world_designer_brief_v1.md) |
| **Unblocks** | **S7B-PLAN-001** (`@planner`) → **S7B-M1-001** |

### Copy-paste — @planner (next)

```
Queue: S7B-PLAN-001
Read: docs/archive/2026-06-src-dev/plans/stage7_behavioral_full_plan_v1.md
      docs/archive/2026-06-prompts-guides/runbooks/guides/stage7_behavioral_decision_worksheet_v1.md (SIGNED picks)
      docs/archive/2026-06-src-dev/trees/stages/stage7_behavioral_planner_handoff_v1.md
Deliver: phase plan + stage7_behavioral_live.json field schema
Prereq: S7B-DESIGN-001 SIGNED — no Rust in design gate
VM-09: v2 for M2+ comm authority; M1 contracts OK under PROJ-2 policy
```

---

## @designer FIRE7-DESIGN-001

| Field | Value |
|:---|:---|
| **Status** | **DONE — SIGNED** |
| **Signed** | 2026-05-25 |
| **Witness** | [`fire_lod_player_read_v1.md`](fire_lod_player_read_v1.md) |
| **Plan** | [`stages/fire_sim_phase7_plan_v1.md`](stages/fire_sim_phase7_plan_v1.md) · [`fire_sim_phase7_architecture_v1.md`](fire_sim_phase7_architecture_v1.md) |
| **Unblocks** | **F7-C** policy caps (after **F7-A-001**); optional **FIRE7-DESIGN-002** |

---

## @designer S7P-DESIGN-002

| Field | Value |
|:---|:---|
| **Status** | **DONE — SIGNED** |
| **Signed** | 2026-05-25 |
| **Witness** | [`s7p_grid_overload_ux_note_v1.md`](s7p_grid_overload_ux_note_v1.md) |
| **Unblocks** | **S7P-GRID-UX-UI-001** (Coder B **B3**) · [`grid_overload_ux.rs`](../economy/activation/grid_overload_ux.rs) |

### Copy-paste — @coder B3

```
Lane: S7P-GRID-UX-UI-001
Read: docs/archive/2026-06-src-dev/plans/s7p_grid_overload_ux_note_v1.md
      src/economy/activation/grid_overload_ux.rs
Do: align GRID_OVERLOAD_TOAST_MESSAGE + PWR prefix; optional alerts tray row
Exit: s7p_grid_ux_toast_ui_wired · s7p_grid_ux_001_green
```

---

## Active (remaining)

| ID | Owner | Status |
|:---|:---|:---|
| *(none — designer gates clear)* | | |
| **UI-P3-M2-TRAY-OPT** | `@coder` | **DONE** — `ui_p3_m2_tray_opt_green` (optional slice closed) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.8.0 | 2026-05-25 | **FIRE7-DESIGN-001** LOD table · **S7P-DESIGN-002** grid overload UX note |
| v1.7.0 | 2026-05-25 | **UI-OH-P4-ART-001** traced Phase 4 icon atlas **SIGNED** |
| v1.6.0 | 2026-05-25 | **UI-OH-D2-SIGN-001** Phase 2 sign-off after **2A** mock parity |
| v1.5.0 | 2026-05-25 | **S7B-DESIGN-001** behavioral worksheet **SIGNED** |
| v1.4.0 | 2026-05-25 | **UX-E02-BQ128-001** BQ-128 editor path **SIGNED** |
| v1.3.0 | 2026-05-25 | **DESIGN-D-VFX-POST-001** orchestrator ID + todo section |
| v1.2.0 | 2026-05-25 | **DESIGN-D-WP-REVIEW-001** post-impl **PASS** |
| v1.1.0 | 2026-05-25 | Six @designer todos + `designer_signoff_registry.json` |
| v1.0.0 | 2026-05-25 | Initial five-todo closure |
