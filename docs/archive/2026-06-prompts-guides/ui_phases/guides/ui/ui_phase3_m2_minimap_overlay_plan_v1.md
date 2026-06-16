# UI Phase 3 M2 — minimap overlay plan + designer legend `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **UI-P3-M2-PLAN** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@planner` · **M2 coder:** `@coder` · **legend:** `@designer` |
| **Status** | **SIGNED — M2 COMPLETE** (incl. **UI-P3-M2-TRAY-OPT**, 2026-05-23) |
| **Sign-off** | [`minimap_d_m2_signoff_v1.md`](../../../docs/archive/2026-06-src-dev/plans/minimap_d_m2_signoff_v1.md) (**D-MINIMAP-M2**) |
| **Compositor plan** | [`ui_phase3_minimap_compositor_plan_v1.md`](ui_phase3_minimap_compositor_plan_v1.md) |
| **Impl plan** | [`ui_phase3_minimap_m2_impl_full_plan_v1.md`](ui_phase3_minimap_m2_impl_full_plan_v1.md) (**PLAN-UI-P3-M2-IMPL-001**) · rollup [`ui_phase3_minimap_m2_impl_plan_v1.md`](ui_phase3_minimap_m2_impl_plan_v1.md) |
| **Design** | [`ux_gpu_minimap_design_v1.md`](../../../docs/archive/2026-06-src-dev/plans/ux_gpu_minimap_design_v1.md) §7 M2 |

---

## Summary

**M2** adds three **strategic heat channels** on the GPU minimap compositor, reading published snapshots only. **Coder work is done** per witness; this plan unblocks **designer legend** (HUD copy + map read) and documents the one optional polish slice.

---

## Overlay legend (designer — **D-MINIMAP-M2-LEGEND**)

Use in context tray / F3 diagnostics / future overlay tray tooltips.

| Toggle key | Player label (suggested) | Visual read | Token hint |
|:---|:---|:---|:---|
| `fire_heat` | Fire activity | Chunk heat from shared overlay field | Warm rust / ember (existing M1) |
| `logistics_heat` | Corridor flow | Transport overlay rows | Dirty amber thread on routes |
| `construction_heat` | Active builds | Corridor construction phases | Drafting magenta / warm orange bands |
| `ecology_heat` | Climate stress | Ecology macro band | Muted chlorophyll wash |

**Default in sim:** fire + logistics **on** ([`simulation_minimap_overlay_defaults`](../../../src/gui/minimap_shell.rs)); construction + ecology per scenario seed.

**Designer deliverable (optional):** one PNG mock `assets/ui/minimap/overlay_legend_v1.png` — 4 swatches + labels (non-blocking).

---

## Coder slices

| ID | Scope | Status | Witness field |
|:---|:---|:---|:---|
| **UI-P3-M2-001** | Logistics heat | **DONE** | `logistics_rows > 0` |
| **UI-P3-M3-001** | Construction + ecology (code naming) | **DONE** | `construction_rows`, `ecology_rows`, `ui_p3_m3_green` |
| **UI-P3-M2-TRAY-OPT** | Overlay tray → `MinimapOverlayMask` live | **DONE** | `ui_p3_m2_tray_opt_green` |

### Copy-paste — UI-P3-M2-TRAY-OPT (optional)

```
Lane: UI-P3-M2-TRAY-OPT
Read: docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_m2_minimap_overlay_plan_v1.md
First: wire Hud overlay tray checkboxes to MapViewInstances.minimap.overlays
Max files: 3 — in_game_hud.rs or overlay tray + minimap_shell sync
Verify: cargo test -p proc_A_dine01 --lib minimap_compositor stage5
Witness: toggling tray bit flips compositor uniform in same session
```

---

## Acceptance (M2 — met 2026-05-24)

| # | Criterion | Met |
|:---:|:---|:---:|
| M1 | `composite_ok`, `GpuCompute`, `dual_minimap_present: false` | ☑ |
| M2-01 | `logistics_rows: 2` | ☑ |
| M2-02 | `construction_rows: 18` | ☑ |
| M2-03 | `ecology_rows: 100` | ☑ |
| M2-04 | No duplicate extract | ☑ |
| M2-06 | Tray live bridge | ☑ |

---

## M3 forward (out of M2 plan)

Per design §7 M3: fog-of-war, EW, unit markers — **separate** planner slice when Stage 7 behavioral gates open.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | PLAN UI-P3-M2-PLAN — legend + TRAY-OPT queue |
