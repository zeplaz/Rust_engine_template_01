# APS-UX-DENSITY-001 — Assembly density sign-off `v1.1`

| Field | Value |
|:---|:---|
| **Program** | **APS-UX-DENSITY-001** |
| **Parent** | [`design_tools_ux_polish_charter_v1.md`](design_tools_ux_polish_charter_v1.md) Phase 4 |
| **Date** | 2026-06-03 |
| **Owner** | `@designer` (sign-off) · `@coder-mcp` (implementation) |
| **Verdict** | **PASS** |
| **Witness** | [`debug_runs/aps_ux_density_001_live.json`](../../debug_runs/aps_ux_density_001_live.json) |

---

## Viewport policy (read this first)

| Tier | Size | What we mean |
|:---|:---|:---|
| **Design target** | **1280×800** on **1080p** display | Production daily driver — must **work and look nice** here |
| **Default launch** | **1280×800** | APS opens here (`aps_theme.DEFAULT_WINDOW_SIZE`) |
| **Comfortable max** | **1440×900** | More space on same monitor — supported |
| **Minimum floor** | **960×600** | `minsize` — **regression guard only** |

960×600 is **not** the product canvas. Aesthetic sign-off at **1280×800**; also spot-check **1440×900** when undocked.

Canonical rules: [`aps_ux_professional_polish_rules_v1.md`](aps_ux_professional_polish_rules_v1.md) §0.

---

## Mission

Progressive disclosure on Assembly (and Variants advanced strips) so **primary work** — generate, footprint, slot edit, previews — stays visible without wading through tag grids and grammar trees on every visit.

**Primary review viewport:** **1280×800**. Also supported: **1440×900**. **Floor check:** 960×600 still completable.

---

## Implemented (coder-mcp)

| Section | Default | Location |
|:---|:---:|:---|
| **Semantic & variant tags** | Collapsed | `assembly_panel.py` → `CollapsibleSection` |
| **Grammar inspector** | Collapsed | `assembly_panel.py` → nested in collapsible |
| **Iterate grammar** | Collapsed | Generate block → `Iterate grammar (advanced)` |
| **Agent patch strip** | Collapsed | `variants_panel.py` → `Agent patch strip (advanced)` |
| **Footprint pane min width** | 280px @ narrow workspace | `assembly_panel.py` `_workspace_minsizes` |

**Collapsed headers (designer):**

| Section | Collapsed label pattern |
|:---|:---|
| Tags | `Semantic & variant tags (N selected)` when slot has active tags |
| Grammar | `Grammar inspector — {human archetype}` from snapshot |

Component: [`aps_collapsible.py`](../../tools/mcp/art_pipeline_suite/aps_collapsible.py) — `set_title()` for dynamic badges.

---

## Designer acceptance

| # | Check | Viewport | Result |
|:---:|:---|:---|:---:|
| D1 | Tags collapsed on fresh Assembly tab | 1280×800 | **PASS** |
| D2 | Grammar collapsed on fresh tab | 1280×800 | **PASS** |
| D3 | Iterate grammar collapsed | 1280×800 | **PASS** |
| D4 | Variants agent strip collapsed | 1280×800 | **PASS** |
| D5 | Primary column visible without scrolling past tag grid | **1280×800** | **PASS** |
| D6 | Collapsed tag header shows selection count | any | **PASS** |
| D7 | Collapsed grammar header shows archetype hint | any | **PASS** |
| D8 | Same primary path usable (not pretty required) | 960×600 floor | **PASS** (witness) |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@coder-mcp` | **witness green** | 2026-06-03 |
| `@designer` | **PASS** | 2026-06-03 |

```text
APS-UX-DENSITY-001 complete
Design target: 1280×800 @ 1080p (comfortable max 1440×900)
Floor: 960×600 regression only
Witness: debug_runs/aps_ux_density_001_live.json
```
