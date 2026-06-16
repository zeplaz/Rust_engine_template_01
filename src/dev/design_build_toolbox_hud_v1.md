# Build toolbox + Adjust HUD copy `v1` (BUILD-READ-DESIGN-002)

| Field | Value |
|:---|:---|
| **Program** | **BUILD-READ-DESIGN-002** |
| **Date** | 2026-06-13 |
| **Owner** | `@designer` (charter) · `@coder` wire in `build_strip` / `contextual_tip` |
| **Verdict** | **PASS** |
| **Parent** | [`plan_operator_build_readability_exec_001_v1.md`](plan_operator_build_readability_exec_001_v1.md) |
| **Extends** | [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) · [`design_build_readability_v1.md`](design_build_readability_v1.md) §4 |
| **Witness** | [`debug_runs/design_build_toolbox_hud_live.json`](../debug_runs/design_build_toolbox_hud_live.json) |

**No Rust.** Locked strings + surface map — not hover-only tooltips.

---

## Mission

Build placement affordances must be **visible in the context strip and rail** — Ctrl=rotate, Shift=scale, two-click place, site overlay legend. Player never discovers modifiers only from F3 debug.

**Acceptance test:** *With Industry tool active and pilot selected, context strip names Ctrl/Shift; rail tooltip names L-footprint pilot; site overlay legend visible when stub on.*

---

## 1. Surface map

| Surface | File (coder) | Always visible? |
|:---|:---|:---:|
| Context strip | `contextual_tip.rs` | Yes — bottom peek |
| Build rail slot tooltip | `build_rail` / category submenu | On hover + selected row |
| Build toolbox button | `build_toolbox.rs` (editor) | When dock open |
| Site overlay legend | placement debug / site overlay | When `BUILD-READ-SITE-v0-002` on |
| Toast | existing validation | On blocked place |

---

## 2. Locked copy — context strip

Merge [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) §3a with readability site context:

| Mode | Template |
|:---|:---|
| **Preview** | `BUILD · {category} · {archetype} · click map to lock · [{cycle}] category` |
| **Adjust (valid)** | `BUILD · {archetype} · locked {x},{z} · Ctrl rotate · Shift scale · click to place · Esc cancel` |
| **Adjust (invalid)** | `BUILD · locked {x},{z} · blocked: {reason} · Esc cancel` |
| **Adjust + site stub** | append ` · site overlay on` when overlay active |
| **Idle** | unchanged non-build strings |

**Remove** as primary: `Enter to commit`, `Shift+click queue`.

---

## 3. Locked copy — build rail / submenu

| Surface | Text |
|:---|:---|
| Industry row (pilot) | `Rail Warehouse (pilot)` |
| Rail tooltip | `L footprint · rotate QA · 11 tiles · site stub 10×8` |
| Preview hint (selected) | `Click map to lock placement` |
| Adjust hint (selected) | `Ctrl rotate · Shift scale · click again to place` |
| Invalid (selected) | `Blocked — {short reason}` |

**Shortcut footnote** (secondary line, smaller): `Enter — place (optional)`

---

## 4. Locked copy — site overlay legend

When site stub overlay enabled ([`BUILD-READ-SITE-v0-002`](plan_operator_build_readability_exec_001_v1.md)):

| Element | Copy / style |
|:---|:---|
| Legend title | `Site stub` |
| Building zones | `Green — building footprint` |
| Void zones | `Dashed — yard / rail / park` |
| Labels | `Yard` · `Rail` · `Svc` · `Park` · `Load` per [`design_build_readability_v1.md`](design_build_readability_v1.md) §2c |

Legend position: **context tray Build tab** footer or 1-line strip under context tip — not floating on map center.

---

## 5. Footprint state copy (partial alpha)

From [`design_build_readability_v1.md`](design_build_readability_v1.md) §1b — wire to ghost + strip:

| State | Strip suffix | Fill |
|:---|:---|:---|
| Valid | (none) | Green α≤0.35 |
| Risky | ` · risky overlap` | Amber hatch |
| Invalid | ` · blocked: {reason}` | Red hatch |
| Adjust locked | ` · locked` | Gold ring |

---

## 6. Accessibility

| # | Rule |
|:---:|:---|
| A1 | Mode + modifiers in **strip text** every frame in Adjust |
| A2 | Invalid names `reason` — not color-only |
| A3 | Enter documented as optional — rail footnote |
| A4 | Site legend text labels — not color-only |

---

## 7. Coder handoff

| Slice | Do |
|:---|:---|
| **BUILD-READ-DESIGN-002** | Wire §2–§5 strings; no new FSM |
| **BUILD-READ-SHAPE-002** | Rail row + tooltip for `pilot:logistics_rail_warehouse_v0` |
| **BUILD-READ-SITE-v0-002** | Legend §4 when overlay on |

Tests: none required for copy — witness JSON + manual G-PLAY checklist.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-13 |
| `@coder` | **context strip wired · witness green** | 2026-06-13 |

```text
BUILD-READ-DESIGN-002 complete
Unblocks: BUILD-READ-SHAPE-002 copy · BUILD-READ-SITE-v0-002 legend
```
