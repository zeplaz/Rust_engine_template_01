# DES-APS-ASSEMBLY-DENSITY-001 — Assembly MIN-window layout spec `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-ASSEMBLY-DENSITY-001** |
| **Parent** | [`design_aps_uiux_style_quality_20260616_v1.md`](design_aps_uiux_style_quality_20260616_v1.md) §3 |
| **Date** | 2026-06-16 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |

---

## Breakpoint

| Width | Layout |
|:---|:---|
| **≥ 1100px** | 3-pane: Footprint \| Materials \| Inspector (current) |
| **960–1099px** | **2-pane:** Footprint+Materials left (stacked) \| Inspector right |
| **< 960px** | Blocked by `minsize(960,600)` — must not horizontal-scroll |

## 2-pane detail @ 960×600

```text
┌─ Footprint (top 45%) ─────────┐┌─ Inspector (full height) ───┐
│ grid + placement list         ││ Slot previews (1 row scroll)   │
├─ Materials (bottom 55%) ───────┤│ Selected slot edit             │
│ tree collapsed to 8 rows      ││ Tags in sub-notebook tab 2     │
└───────────────────────────────┘└────────────────────────────────┘
```

## Collapse defaults @ MIN

| Section | Default |
|:---|:---|
| Grammar inspector | **collapsed** |
| Variant tags | **collapsed** |
| Metadata → engine | collapsed (unchanged) |
| Slot previews | **visible** (priority) |

## Landscape Grammar tab

Apply **same breakpoint rule** pre-emptively: graph left \| inspector right → stack below 1100px.

## Acceptance

Manual: 960×600, complete assign-material loop without horizontal scrollbar.

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-16 |
