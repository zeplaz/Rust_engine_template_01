# UI Phase 4 — world preview shell + optional art `v1`

| Field | Value |
|:---|:---|
| **Track ID** | `UI-P4` |
| **Version** | `1.0.0` |
| **Status** | **CODER CLOSED** (2026-05-25) — optional WP-L3/L4 |
| **Exit milestone** | **UI Phase 4 operational** — D-04 slide sheet + optional atlas PNG |
| **Master UI** | [`../ui_overhaul_plan.md`](../ui_overhaul_plan.md) |
| **D-01** | **DONE** — [`../../prompts/guides/ui/ui_world_preview_coder_queue_v1.md`](../../prompts/guides/ui/ui_world_preview_coder_queue_v1.md) |
| **Handoff** | [`../../prompts/guides/ui/ui_phase4_handoff_plan_v1.md`](../../prompts/guides/ui/ui_phase4_handoff_plan_v1.md) (**UI-P4-PLAN**) |
| **Product** | [`../../prompts/guides/ui/world_preview_product_decision_v1.md`](../../prompts/guides/ui/world_preview_product_decision_v1.md) (**PLAN-WP-DECISION-001**) |

---

## Scope

| In scope | Out of scope |
|:---|:---|
| World Map Preview chrome (D-04, D-09, motion) | Simulation HUD P1–P4 (closed) |
| Phase 4 icon atlas PNG (optional) | Minimap M3 overlays → [`ui_phase3_minimap_compositor_plan_v1.md`](../../prompts/guides/ui/ui_phase3_minimap_compositor_plan_v1.md) M3 queue |
| Wave P composite **read-only** in preview | Wave P gameplay mutation |

**Boundary:** [`../../prompts/guides/ui_boundary_guide_v1.md`](../../prompts/guides/ui_boundary_guide_v1.md) — preview = egui tooling; sim HUD = Bevy.

---

## @designer instructions

### UI4-DESIGN-001 — D-04 slide sheet layout (required for LAYOUT-002)

**Read:** [`../../prompts/guides/ui/world_map_preview_layout_decision_v1.md`](../../prompts/guides/ui/world_map_preview_layout_decision_v1.md) § D-04

| Deliverable | Path |
|:---|:---|
| **Spec on disk** | [`world_preview_d04_slide_sheet_spec_v1.md`](../../prompts/guides/ui/world_preview_d04_slide_sheet_spec_v1.md) — **UI4-DESIGN-001** |
| Annotated mock | `assets/ui/world_preview/slide_sheet_spec_v1.png` (or extend `layout_mock_v1.png`) |
| Dimmed map + sheet height % | In spec § Layout + worksheet § D-04 |

**Status:** **DONE** (2026-05-24) — **SIGNED**; unblocked **UI-WP-LAYOUT-002**.

### UI4-DESIGN-002 — Phase 4 icon atlas art (optional, non-blocking)

**Read:** [`../../prompts/guides/ui/ui_phase4_icon_atlas_brief_v1.md`](../../prompts/guides/ui/ui_phase4_icon_atlas_brief_v1.md)

| Task | Output |
|:---|:---|
| Replace placeholder PNG | `assets/textures/ui/icon_atlas_phase4_v1.png` |
| Match manifest | `configs/ui/icon_atlas_phase4.icon_atlas.ron` grid |

**Blocks:** nothing — code path already green with placeholder.

### UI4-DESIGN-003 — Map look (WP-L4)

**Read:** [`../../prompts/guides/ui/world_preview_visual_references_v1.md`](../../prompts/guides/ui/world_preview_visual_references_v1.md)

Sign color key + ref captures 01–06 before **UI-WP-L4-001** coder slice.

---

## @coder instructions

### Slice map

| ID | Goal | Status | Docs |
|:---|:---|:---:|:---|
| **UI-WP-LAYOUT-001** | D-01 unified workspace | ✅ DONE | world preview coder queue |
| **UI-WP-LAYOUT-002** | D-04 slide sheet body | ✅ **DONE** | `window.rs` dim + sheet; lib `ui_wp_layout_002_*` green |
| **UI-WP-LAYOUT-003** | Paper frames + D-09 offsets | queued | needs WP-L1 assets |
| **UI-WP-MOTION-001** | Motion table §6 | queued | after LAYOUT-002 |
| **UI-WP-L4-001** | Raster look from signed refs | queued | after UI4-DESIGN-003 |
| **PLAN-UI-P4-ATLAS-001** | Icon atlas + petroleum rollup | **DONE** (planner) | [`ui_phase4_icon_atlas_plan_v1.md`](../../prompts/guides/ui/ui_phase4_icon_atlas_plan_v1.md) |
| **P4-P5-01** | Petroleum tab `P5Br` | **DONE** | `sync_petroleum_panel_tab_system` |
| **P4-ART-01** | Traced atlas PNG if lands | optional | designer drop → coder verify |
| **P4-VEH-01** | Vehicle row consumers | queued | logistics / convoy UI |

**Files (typical):** `src/gui/editor/world_preview/window.rs`, `world_gen_ui.rs`, `world_preview/mod.rs` — **≤3 per PR**.

**Do NOT:** mutate sim `RepresentationResult`; add minimap extract; break `stage5` FULL_APP.

### Copy-paste — UI-WP-LAYOUT-002

```
Track: UI-P4 — UI-WP-LAYOUT-002
Read: src/dev/stages/ui_phase4_execution_plan_v1.md
      prompts/guides/ui/world_map_preview_layout_decision_v1.md § D-04
Prereq: UI4-DESIGN-001 SIGNED
First: implement slide sheet panel + dimmed map in window.rs
Do NOT: render_raster graph / GenerateWorldEvent
Verify: F8 WorldGen → sheet opens; cargo test -p proc_A_dine01 --lib stage5
```

### Copy-paste — P4-ART-01 (optional, post plan)

```
Track: UI-P4 — P4-ART-01
Read: prompts/guides/ui/ui_phase4_icon_atlas_plan_v1.md § P4.2b
      ui_phase4_icon_atlas_brief_v1.md §4–§7
First: drop designer PNG; verify atlas loads in sim build rail
Do NOT: change IconId enum without manifest update
Verify: ui_shell_migration_live.json icon_atlas_loaded: true
```

### Acceptance — UI Phase 4 exit

| # | Criterion |
|:---:|:---|
| U1 | D-04 slide sheet functional (LAYOUT-002) |
| U2 | `cargo test -p proc_A_dine01 --lib stage5` green |
| U3 | Wave P: `wave_p_live.json` refreshed in sim (operator) |
| U4 | Optional: traced atlas in repo OR explicit defer documented |
| U5 | No preview ECS mutation (Wave P audit clean) |

---

## @sim-steward preflight (before LAYOUT-002)

**ID:** `UI-P4-PREFLIGHT`

- Confirm preview RT ≠ minimap RT (existing unit test)
- Confirm `product_egui_shell_active` false during WorldGen where required
- Shift B GO → hand to `@coder` UI-WP-LAYOUT-002

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | D-01 done; forward slices defined |
