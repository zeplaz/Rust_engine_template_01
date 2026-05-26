# Designer workboard `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.6.1` |
| **Date** | 2026-05-25 |
| **Orchestrator registry** | [`tools/orchestrator/queues/designer_signoff_registry.json`](../../tools/orchestrator/queues/designer_signoff_registry.json) |
| **Todo board** | [`stage_designer_todos_v1.md`](stage_designer_todos_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/designer_active_queue.json`](../../tools/orchestrator/queues/designer_active_queue.json) |
| **Sign-off ledger** | [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) |

---

## @designer sign-offs — all **SIGNED** (orchestrator)

| Queue ID | Sign-off | Witness |
|:---|:---:|:---|
| **UI4-DESIGN-001** | **SIGNED** 2026-05-24 | [`world_preview_d04_slide_sheet_spec_v1.md`](../prompts/guides/ui/world_preview_d04_slide_sheet_spec_v1.md) |
| **S7P-DESIGN-001** | **SIGNED** 2026-05-24 | [`stage7_play_scenario_v1.md`](stage7_play_scenario_v1.md) |
| **DESIGN-UI-P2-SIGNOFF-001** | **SIGNED** 2026-05-24 | [`ui_phase2_designer_signoff_v1.md`](../prompts/guides/ui/ui_phase2_designer_signoff_v1.md) |
| **DESIGN-VFX-CAPTURE-001** | **SIGNED** 2026-05-25 **PASS** | [`vfx_capture_status_20260525.md`](../assets/vfx/reference/review_captures/vfx_capture_status_20260525.md) |
| **WATER-DESIGN-002** | **SIGNED** 2026-05-24 | [`water_ocean_fixture_request_v1.md`](water_ocean_fixture_request_v1.md) |
| **DESIGN-MINIMAP-M2-001** | **SIGNED** 2026-05-24 **M2 COMPLETE** | [`minimap_d_m2_signoff_v1.md`](minimap_d_m2_signoff_v1.md) |
| **MINIMAP-DESIGN-M3-001** | **SIGNED** 2026-05-25 | [`minimap_m3_operational_overlay_spec_v1.md`](../prompts/guides/ui/minimap_m3_operational_overlay_spec_v1.md) · [`minimap_d_m3_signoff_v1.md`](minimap_d_m3_signoff_v1.md) |
| **DESIGN-D-WP-REVIEW-001** | **SIGNED** 2026-05-25 **PASS** | [`world_preview_d_wp_review_record_v1.md`](world_preview_d_wp_review_record_v1.md) |
| **DESIGN-D-VFX-POST-001** | **SIGNED** 2026-05-25 **PASS** | [`vfx_design_review_record_v1.md`](vfx_design_review_record_v1.md) |
| **UX-E02-BQ128-001** | **SIGNED** 2026-05-25 | [`bq128_editor_path_design_note_v1.md`](bq128_editor_path_design_note_v1.md) · plan [`bq128_editor_path_plan_v1.md`](bq128_editor_path_plan_v1.md) |
| **S7B-DESIGN-001** | **SIGNED** 2026-05-25 | [`stage7_behavioral_decision_worksheet_v1.md`](../prompts/guides/stage7_behavioral_decision_worksheet_v1.md) · [`stage7_behavioral_d_signoff_v1.md`](stage7_behavioral_d_signoff_v1.md) |
| **FIRE7-DESIGN-001** | **SIGNED** 2026-05-25 | [`fire_lod_player_read_v1.md`](fire_lod_player_read_v1.md) |
| **S7P-DESIGN-002** | **SIGNED** 2026-05-25 | [`s7p_grid_overload_ux_note_v1.md`](s7p_grid_overload_ux_note_v1.md) |

---

## Active

| Queue ID | Owner | Status |
|:---|:---|:---|
| *(none — designer gates clear)* | | |

**Status:** **idle** — all queue sign-offs **SIGNED**. No designer `@assign` until a new gate opens.

### Optional / non-blocking (designer)

| ID | Notes |
|:---|:---|
| **VX-P0-04** | VFX **ACCEPTED** PNG round — harness already green; refresh captures under `assets/vfx/reference/review_captures/` if promoting mock compare |
| **UX-E03** | ~~Transmission stub design note~~ **DONE** — [`ux_e03_transmission_shell_note_v1.md`](ux_e03_transmission_shell_note_v1.md) |

### Routed to `@coder` (not designer)

| ID | Notes |
|:---|:---|
| **UI-P3-M2-TRAY-OPT** | Optional — overlay tray → `MinimapOverlayMask` (M2 deferred) |

### Do not assign (complete)

| ID | Reason |
|:---|:---|
| **S7P-DESIGN-001** | Done — [`stage7_play_scenario_v1.md`](stage7_play_scenario_v1.md) |
| **WATER-DESIGN-001** / **WATER-DESIGN-002** | Done — ocean fixture + [`water_vfx_review_record_v1.md`](water_vfx_review_record_v1.md) |
| **UI4-DESIGN-001** | Done — D-04 slide sheet |
| **VFX-POST-REVIEW-DESIGN** / **DESIGN-D-VFX-POST-001** | Done — alias **D-VFX** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.6.1 | 2026-05-25 | Idle state — optional VX-P0-04 / UX-E03; do-not-assign list |
| v1.6.0 | 2026-05-25 | **S7B-DESIGN-001** worksheet SIGNED |
| v1.5.0 | 2026-05-25 | Six @designer todos + `designer_signoff_registry.json` |
| v1.4.0 | 2026-05-25 | Five-todo closure |
| v1.2.0 | 2026-05-24 | Initial batch |
