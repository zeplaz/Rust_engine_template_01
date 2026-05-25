# World Preview D-01 — shell model sign-off `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` → `@coder` **UI-WP-LAYOUT-001** |
| **Status** | **SIGNED** (2026-05-24) — **UI-WP-LAYOUT-001 done** |
| **Parent gate** | [`world_map_preview_layout_decision_v1.md`](world_map_preview_layout_decision_v1.md) §5 (D-01…D-12) |
| **Track rollup** | [`world_preview_d_wp_track_signoff_v1.md`](world_preview_d_wp_track_signoff_v1.md) (**D-WP**) |
| **Worksheet** | [`world_preview_layout_decision_worksheet_v1.md`](world_preview_layout_decision_worksheet_v1.md) |
| **Mock** | [`assets/ui/world_preview/layout_mock_v1.png`](../../../assets/ui/world_preview/layout_mock_v1.png) |
| **Coder queue** | [`ui_world_preview_coder_queue_v1.md`](ui_world_preview_coder_queue_v1.md) |
| **D-02 (optional)** | [`world_preview_d02_map_dominance_signoff_v1.md`](world_preview_d02_map_dominance_signoff_v1.md) — map ≥65%; coder **not** required for D-01 exit |

---

## Executive summary

**D-01 A** — **one egui workspace** hosts World Map Preview + World Generator parameters. The legacy floating **“World Generator”** window is **not** shown when unified mode is on. Parameters open as a **left slide sheet** (toggle **Parameters ▸** in toolbar); full sheet body polish is **D-04** / **UI-WP-LAYOUT-002**.

**Not** [`ui_phase2_designer_signoff_v1.md`](ui_phase2_designer_signoff_v1.md) (simulation HUD P1–P4).

---

## §5 — Signed decision (D-01 only)

| ID | Question | **Choice** | Final spec |
|:---|:---|:---:|:---|
| **D-01** | Shell model | **A** | **Single workspace** — one window/frame; generator as **slide sheet**, not second float |

**Rejected for v1:**

| Option | Why not |
|:---|:---|
| **B** Dual window | Duplicate labels, click steal, unstable load ([`world_gen_chrome_contract.rs`](../../../src/gui/world_gen_chrome_contract.rs)) |
| **C** Docked pair tab | Extra chrome; D-01 A matches §9 operator journey |

**Overrides:** none.

---

## Implementation contract (coder)

| Item | Spec | Code anchor |
|:---|:---|:---|
| Single window | One egui float: **“Operational Archive — World Index”** | [`window.rs`](../../../src/gui/editor/world_preview/window.rs) |
| Unified flag | `WORLD_PREVIEW_UNIFIED_WORKSPACE = true` | [`world_preview/mod.rs`](../../../src/gui/editor/world_preview/mod.rs) |
| Open sync | F8 / new-world latch opens preview + gen visibility | `open_world_gen_workspace` |
| No second float | `world_gen_ui` returns early when unified | [`world_gen_ui.rs`](../../../src/gui/editor/world_gen_ui.rs) |
| Slide sheet entry | Toolbar **Parameters ▸ / ◂** toggles `generator_sheet_open` | `window.rs` |
| Slide sheet body | `draw_world_gen_panel(..., unified_workspace: true)` in left panel | `window.rs` — **stub OK** for LAYOUT-001 |
| Map preserved | Central panel + camera fit unchanged | `window.rs` central panel |
| Scope limit | ≤3 files; **no** `render_raster.rs` / motion §6 | [`ui_world_preview_coder_queue_v1.md`](ui_world_preview_coder_queue_v1.md) |

**Deferred (not D-01):** D-04 dimmed map behind sheet · D-09 asymmetry offsets · §6 motion · WP-L1 paper textures · WP-L4 map look.

---

## Verification

| Check | Pass if |
|:---|:---|
| F8 WorldGen | One workspace visible (not two floats) |
| Parameters toggle | Sheet opens/closes; legacy generator window absent |
| Map camera | Center/zoom stable on resize |
| Tests | `cargo test -p proc_A_dine01 --lib world_preview` · `stage5` green |

**Witness (2026-05-24):** unit tests `unified_workspace_open_syncs_flags` · `unified_workspace_chrome_may_render_uses_preview_only` · **UI-WP-LAYOUT-001** marked **done** in [`continuation_queue.json`](../../../tools/orchestrator/queues/continuation_queue.json).

---

## §11 Designer sign-off checklist (D-01)

| # | Item | Done |
|:---|:---|:---:|
| 1 | **D-01** choice **A** recorded on worksheet | ☑ |
| 2 | Mock shows single-workspace intent ([`layout_mock_v1.png`](../../../assets/ui/world_preview/layout_mock_v1.png)) | ☑ |
| 3 | §9 journey: no dual-window hunt | ☑ |
| 4 | Parent §11 SIGNED ([`world_map_preview_layout_decision_v1.md`](world_map_preview_layout_decision_v1.md)) | ☑ |
| 5 | Coder slice **UI-WP-LAYOUT-001** scope agreed (shell only) | ☑ |
| 6 | Implementation verified in code + tests | ☑ |

**Verdict:** ☑ **SIGNED**

| Role | Date | Verdict | Notes |
|:---|:---|:---|:---|
| Designer | 2026-05-24 | **SIGNED** | D-01 A via full layout worksheet (recommended defaults) |
| Coder | 2026-05-24 | **Done** | `WORLD_PREVIEW_UNIFIED_WORKSPACE` + slide sheet hook |

---

## Unblocks / still open

| Slice | Status |
|:---|:---|
| **UI-WP-LAYOUT-001** (D-01 shell) | **done** |
| **UI-WP-LAYOUT-002** (D-04 sheet body + dim) | queued |
| **UI-WP-LAYOUT-D02-OPT** (D-02 map ≥65%) | **optional** — [`world_preview_d02_map_dominance_signoff_v1.md`](world_preview_d02_map_dominance_signoff_v1.md) |
| **UI-WP-MOTION-001** (§6 motion) | deferred |
| **WP-L4** (map look from capturez) | deferred |

---

## Coder handoff (historical)

```
Lane: UI-WP-LAYOUT-001 — D-01 single workspace shell
Read: world_preview_d01_shell_signoff_v1.md + layout_mock_v1.png
Touch: world_preview/mod.rs, window.rs, world_gen_ui.rs (≤3)
Do NOT: second float, motion §6, raster graph
Verify: cargo test -p proc_A_dine01 --lib world_preview stage5
```

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Focused D-01 sign-off; parent gate SIGNED; UI-WP-LAYOUT-001 done |
