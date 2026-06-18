# APS Assembly workflow realignment `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-ASSEMBLY-WORKFLOW-001** |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Implements** | `AssemblyPanel._build()` reorder |
| **Authority** | [`design_aps_grammar_tier_wireframes_v1.md`](design_aps_grammar_tier_wireframes_v1.md) |

---

## Artist path (Buildings · Assembly tab)

```text
1. Generate strip   — tier chip · type · district · seed · [Generate Assembly]
2. Work area        — Footprint | Material library | Inspector (3-pane)
3. File / ship      — Load · Save · Check schema · Run ship check · Preview
4. Setup (collapsed)— grammar toggle · manual override · iterate · DNA
5. Kit reference    — build-set brief + sweep (advanced, bottom)
```

## Rationale

| Before | After |
|:---|:---|
| Generate LabelFrame stacked 5 grammar panels | Primary row only; advanced in **Setup** |
| Kit grammar reference at top | Bottom — reference, not first action |
| Material authority block above generate | One line inside Material library pane |
| Manual style/footprint beside primary combos | **Setup & manual fallback** (collapsed) |

## Tier behavior (unchanged contract)

See [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md). G2+ adds **Set health** strip with Run sweep.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
