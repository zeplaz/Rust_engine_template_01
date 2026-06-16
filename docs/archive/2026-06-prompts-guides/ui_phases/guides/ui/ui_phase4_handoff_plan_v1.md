# UI Phase 4 — handoff plan (UI4 → UI-WP-LAYOUT-002) `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **UI-P4-PLAN** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@planner` |
| **Status** | **GATE CLEARED** — **UI4-DESIGN-001** **DONE** · **UI-WP-LAYOUT-002** **DONE** |
| **Track plan** | [`stages/ui_phase4_execution_plan_v1.md`](../../../docs/archive/2026-06-src-dev/trees/stages/ui_phase4_execution_plan_v1.md) |
| **Product** | [`world_preview_product_decision_v1.md`](world_preview_product_decision_v1.md) |

---

## Why this handoff exists

**UI4-DESIGN-001** is **SIGNED** (2026-05-24). **UI-WP-LAYOUT-002** landed in `window.rs` / `mod.rs` — **no longer blocked**. **Coder B** parallel lanes use **disjoint files** only (not `world_preview/window.rs`).

---

## Sequence (strict)

```text
1. @designer  UI4-DESIGN-001
      Deliver: slide_sheet_spec_v1.png + worksheet § D-04
      Sign: worksheet row D-04 = SIGNED
           │
           ▼
2. @coder    UI-WP-LAYOUT-002  (Coder A — render/chrome)
      Files: window.rs, world_gen_ui.rs (≤3)
      Exit: F8 → one workspace; sheet opens; map dimmed
           │
           ▼
3. @coder    UI-WP-LAYOUT-003 / MOTION / L4  (queued after 002)
```

**Parallel allowed for Coder B after step 1 starts (not before):**

| Lane | Disjoint from WP? | Examples |
|:---|:---:|:---|
| FX-WATER | ☑ | `gpu_water_particles.rs` |
| IND / S7-PLAY | ☑ | `economy/activation/` |
| UI-SHELL-REFRESH | ☑ | `simulation_shell_phase2` |
| UI-WP-LAYOUT-002 | — | **Coder A only** |

---

## UI4-DESIGN-001 — designer checklist

| # | Deliverable | Path / rule |
|:---:|:---|:---|
| D1 | Slide sheet height | **40–55%** of workspace height when expanded |
| D2 | Map dim | **40–50%** opacity or vellum scrim when sheet open |
| D3 | Entry control | Tab or button label: **"Generator"** / **"Survey sheet"** |
| D4 | Close behavior | Sheet dismiss restores full map brightness |
| D5 | Sign-off artifact | `assets/ui/world_preview/slide_sheet_spec_v1.png` |

**DONE** (2026-05-24) — D4–D5 complete; **UI-WP-LAYOUT-002** landed.

---

## UI-WP-LAYOUT-002 — coder A checklist

| # | Task | Verify |
|:---:|:---|:---|
| C1 | Sheet panel hosts existing generator fields | Fields visible when expanded |
| C2 | Map panel remains interactive under dim | Pan/zoom stable |
| C3 | No second floating World Generator window | grep / manual F8 |
| C4 | `cargo test -p proc_A_dine01 --lib stage5` | green |

### Copy-paste — UI-WP-LAYOUT-002

```
Lane: UI-P4 — UI-WP-LAYOUT-002
Read: docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase4_handoff_plan_v1.md
Prereq: UI4-DESIGN-001 SIGNED + slide_sheet_spec_v1.png
First: implement dimmed map + slide sheet panel in window.rs
Do NOT: render_raster.rs; GenerateWorldEvent; simulation_shell_phase2
Verify: F8 WorldGen manual + cargo test -p proc_A_dine01 --lib stage5
```

---

## UI-WP-LAYOUT-002 — coder B (parallel guard)

```
Lane: (parallel) — NOT UI-WP-LAYOUT-002
Rule: Do not touch world_preview/window.rs or world_gen_ui.rs this session
Pick: WATER-* / IND-* / UI-SHELL-REFRESH per stage_coder_workboard_v1.md
```

---

## Witness

| Artifact | When |
|:---|:---|
| Manual F8 screenshot | After LAYOUT-002 |
| `stage5` tests | Every PR |
| `wave_p_live.json` | Operator refresh (Wave P product exit) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | PLAN UI-P4-PLAN — UI4/LAYOUT-002 coupling |
