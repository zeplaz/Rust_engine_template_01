# APS Grammar Tier Wireframes `v1` — APS-GRAM-TIER-003

| Field | Value |
|:---|:---|
| **ID** | **APS-GRAM-TIER-003** |
| **Program** | PLAN-APS-GRAMMAR-EVOLUTION-001 |
| **Date** | 2026-06-17 |
| **Owner** | `@designer` |
| **Authority** | [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) — **designer owns APS exposure per tier** |
| **Plan ref** | [`plan_aps_grammar_evolution_v1.md`](plan_aps_grammar_evolution_v1.md) § exposure map |
| **Amends** | [`design_aps_assembly_density_v1.md`](design_aps_assembly_density_v1.md) |
| **Implements** | `AssemblyPanel.apply_grammar_tier()` visual contract |
| **Verdict** | **PASS** |

```text
APS-GRAM-TIER-003 Q✓
Wireframes for G0 / G1 / G2 — coder gates per tier table
```

---

## 0. Tier strip (all tiers)

Persistent chip left of Generate row:

| Tier | Chip copy | Tint |
|:---|:---|:---|
| G0 | `G0 — pilot kit` | `COLOR_MUTED` |
| G1 | `G1 — family seed` | `COLOR_ACCENT` |
| G2 | `G2 — axis coverage` | `COLOR_PASS` |
| G3+ | `G3 — layer depth` / `G4 — production` | per plan |

Never show raw `IndustrialWarehouse` in chip.

---

## 1. G0 — minimal (today)

**Expanded panels at launch: ≤2** (Generate + footprint grid). Kit hint **visible**.

```text
┌─ Assembly ─────────────────────────────────────────────────────────────┐
│ G0 — pilot kit                                                        │
│ ▸ Setup (collapsed)                                                   │
├──────────────────┬──────────────────┬───────────────────────────────────┤
│ FOOTPRINT        │ MATERIALS        │ INSPECTOR                         │
│ [grid]           │ [list]           │ [2×2 previews]                    │
│                  │                  │ Selected piece — edit             │
│                  │                  │ ▸ Grammar inspector (collapsed)   │
├──────────────────┴──────────────────┴───────────────────────────────────┤
│ ○ Kit hint: Only one building type in the set — more archetypes unlock │
│   richer generation at G1.                                            │
└───────────────────────────────────────────────────────────────────────┘
```

| Surface | G0 |
|:---|:---|
| Archetype combo | 1 value + hint |
| District combo | 1–2 values |
| Shape bias / DNA | **hidden** |
| Iterate grammar | **hidden** |
| Build-set brief | advanced collapsed |
| Manual fallback | collapsed |
| Inspector | collapsed default |

---

## 2. G1 — family seed (after GRAM-CONTENT-004)

Kit hint **off** or downgraded to one-line caption. Archetype combo **≥3** grouped `Industrial`.

```text
┌─ Assembly ─────────────────────────────────────────────────────────────┐
│ G1 — family seed                                                      │
│ Building type [Industrial Warehouse ▼]  District [Industrial West ▼]  │
│ ▸ Setup (collapsed)                                                     │
├──────────────────┬──────────────────┬───────────────────────────────────┤
│ FOOTPRINT        │ MATERIALS        │ INSPECTOR                         │
│ ...              │ ...              │ ▸ Grammar inspector (collapsed)   │
└───────────────────────────────────────────────────────────────────────┘
```

| Surface | G1 |
|:---|:---|
| Archetype | ≥3 labels (Warehouse · Factory Cluster · Rail Edge) |
| District | ≥2 where grammar provides |
| Shape bias | **collapsed** advanced only |
| Iterate | **collapsed** advanced |
| Build-set | collapsed |
| Kit hint | **hidden** |

Dropdown grouping:
```text
Industrial
  Industrial Warehouse
  Factory Cluster
  Rail Edge Warehouse
```

---

## 3. G2 — shape coverage (target)

DNA preset picker **visible**; iterate still collapsed; build-set **promoted** strip.

```text
┌─ Assembly ─────────────────────────────────────────────────────────────┐
│ G2 — axis coverage                                                   │
│ Building type [...]  District [...]  Shape preset [Logistics rail ▼]  │
│ Set health: ◐ 2/4 massing strategies covered · Run sweep                │
│ ▸ Setup                                                               │
├──────────────────┬──────────────────┬───────────────────────────────────┤
│ FOOTPRINT        │ MATERIALS        │ INSPECTOR                         │
│ click cell ↔     │                  │ Grammar inspector (expandable)      │
│ inspector row    │                  │ Step / Rule / Detail / Tags       │
└───────────────────────────────────────────────────────────────────────┘
```

| Surface | G2 |
|:---|:---|
| Shape bias panel | **visible** — preset picker |
| β sliders | collapsed until G3 |
| Iterate | collapsed |
| Build-set / sweep | **promoted** strip |
| Inspector ↔ grid | **P3** — row click highlights cells |
| Preview | P2 + rule highlight on select |

---

## 4. Anti-patterns (forbid all tiers)

- Five grammar panels expanded at full width before G1
- Engineer ids in primary combos
- Duplicate pipeline walkthrough in Generate **and** status bar
- Footprint canvas on Landscape Grammar tab

---

## 5. MIN window (@ 960×600)

Inherits [`design_aps_assembly_density_v1.md`](design_aps_assembly_density_v1.md):

- G0–G2: 2-pane collapse below 1100px
- Tier chip + Generate row never wrap below footprint visibility threshold
- Kit hint **never** expands to multi-line block at MIN — truncate + tooltip

---

## 6. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-17 |

**NEEDS-DISPLAY:** compare G0 vs G1 launch after `apply_grammar_tier` lands.
