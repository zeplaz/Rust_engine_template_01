# World Map Preview — coder queue `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.1.0` |
| **Date** | 2026-05-24 |
| **Authority** | [`world_map_preview_layout_decision_v1.md`](world_map_preview_layout_decision_v1.md) — **SIGNED** |
| **Worksheet** | [`world_preview_layout_decision_worksheet_v1.md`](world_preview_layout_decision_worksheet_v1.md) |
| **D-01 sign-off** | [`world_preview_d01_shell_signoff_v1.md`](world_preview_d01_shell_signoff_v1.md) |
| **D-02 sign-off** | [`world_preview_d02_map_dominance_signoff_v1.md`](world_preview_d02_map_dominance_signoff_v1.md) — **optional** |
| **D-WP track** | [`world_preview_d_wp_track_signoff_v1.md`](world_preview_d_wp_track_signoff_v1.md) |
| **Mock** | [`assets/ui/world_preview/layout_mock_v1.png`](../../../assets/ui/world_preview/layout_mock_v1.png) |
| **Runbook** | [`world_preview_runbook_v1.md`](../world_preview_runbook_v1.md) (raster only until noted) |
| **Playbook** | [`tools/orchestrator/agents/ui_layout_agent.md`](../../../tools/orchestrator/agents/ui_layout_agent.md) |

**Visual direction (map look, not D-01 shell):** [`world_preview_visual_references_v1.md`](world_preview_visual_references_v1.md) · `assets/ui/world_preview/references/capturez/ref_*.png`

---

## @coder — pick up here (UI-WP-LAYOUT-001)

```
Lane: UI-WP-LAYOUT-001 — D-01 single workspace shell
Read: world_map_preview_layout_decision_v1.md (signed §5) + layout_mock_v1.png
Scope: D-01 ONLY — merge preview + generator into one workspace; generator as slide sheet hook (stub OK)
Max files: 3
Touch: window.rs, world_gen_ui.rs, world_preview/mod.rs OR ui_gates.rs (pick one)
Do NOT: motion §6, paper textures, D-09 offsets, raster graph, GenerateWorldEvent
Verify: F8 WorldGen → one workspace visible; map camera stable on resize
Test: cargo test -p proc_A_dine01 --lib stage5
```

### Slice UI-WP-LAYOUT-001 — D-01 shell

| Step | Task | Files (≤3) | Exit |
|:---:|:---|:---|:---|
| **1** | Single egui workspace window replaces dual-window default | `window.rs`, `world_gen_ui.rs` | One window hosts map layout |
| **2** | Generator: hide separate `World Generator` window when D-04 A active; expose slide sheet entry (button/tab) | same | No second floating window on WorldGen enter |
| **3** | Wire `WorldGenUiState.visible` + `WorldPreviewUiState.window_open` to unified open flag | `world_preview/mod.rs` or chrome latch | F8 opens workspace once |

**Optional slice (not blocking):**

| ID | Scope | Sign-off |
|:---|:---|:---|
| **UI-WP-LAYOUT-D02-OPT** | D-02 — clamp panels so map ≥ **65%** area at HD baseline | [`world_preview_d02_map_dominance_signoff_v1.md`](world_preview_d02_map_dominance_signoff_v1.md) |

**Done (D-04):**

| ID | Scope |
|:---|:---|
| **UI-WP-LAYOUT-002** | D-04 slide sheet body + dimmed map — [`world_preview_d04_slide_sheet_spec_v1.md`](world_preview_d04_slide_sheet_spec_v1.md) **SIGNED** |

**Deferred slices:**

| ID | Blocked by | Scope |
|:---|:---|:---|
| UI-WP-LAYOUT-003 | WP-L1 assets | Paper frames + D-09 offsets |
| UI-WP-MOTION-001 | LAYOUT-001 | §6 motion table |
| **WP-L4** | Signed refs + color key | Map look from `capturez` refs 01–06 · [`world_preview_visual_references_v1.md`](world_preview_visual_references_v1.md) |

---

## Definition of done (UI-WP-LAYOUT-001)

- [x] One workspace window on `AppState::WorldGen` (not two floats)
- [x] Map central panel preserved; camera center/zoom stable on resize
- [x] Separate generator window not shown by default
- [x] `cargo test -p proc_A_dine01 --lib stage5` green
- [x] No changes to `render_raster.rs` / invalidation (D-01 scope only)

**Status:** **DONE** (2026-05-24) — `WORLD_PREVIEW_UNIFIED_WORKSPACE` + slide sheet in `window.rs` / `world_gen_ui.rs`; tests `unified_workspace_*` green.

> **Not UI-P2-DESIGN:** Phase 2 sim shell sign-off ([`ui_phase2_designer_signoff_v1.md`](ui_phase2_designer_signoff_v1.md)) is a **separate** closed lane (P1–P4). World Preview **D-01** is **UI-WP** only.

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-24 | **UI-WP-LAYOUT-001 done** — D-01 shell verified in code + tests |
| v1.0.0 | 2026-05-24 | Unblocked after designer SIGNED; D-01 first slice |
