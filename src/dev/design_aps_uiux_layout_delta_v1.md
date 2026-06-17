# APS UI/UX Layout Delta `v1` — OVR-DES-P3-LAYOUT-DELTA-001

| Field | Value |
|:---|:---|
| **ID** | **OVR-DES-P3-LAYOUT-DELTA-001** |
| **Program** | PLAN-APS-UIUX-OVERHAUL-001 |
| **Phase** | P3 (layout & density) |
| **Date** | 2026-06-17 |
| **Owner** | `@designer` |
| **Authority** | [`aps_design_system_v1.md`](aps_design_system_v1.md) §3.3 · §5 |
| **Inputs** | [`aps_sweep_layout_20260616_v1.md`](aps_sweep_layout_20260616_v1.md) |
| **Implements** | `OVR-P3-LAYOUT-001` · `test_aps_min_window_layout.py` |
| **Verdict** | **PASS** — signed layout delta for `@coder-mcp` |

```text
OVR-DES-P3-LAYOUT-DELTA-001 Q✓
Unblocks: OVR-P3-LAYOUT-001
```

---

## 0. Window contract

| Mode | Size | Acceptance |
|:---|:---|:---|
| Default | 1280×800 | Footprint grid visible **without scroll** |
| Min | 960×600 | No tab-level horizontal scroll; no primary work object off-screen |
| Chrome budget | ≤2 rows (~64px) above notebook | Lane+flow merged; authority in pipeline row |

Tokens: use `GAP_*`, `INSET_*`, `PANE_MIN_*` from [`aps_design_system_v1.md`](aps_design_system_v1.md) §3.3 — no raw padding literals.

---

## 1. Shell deltas (P0)

### SH-1 — Merge chrome bands

**Before:** 5 stacked bands (~157px) — lane · underline · flow · authority · pipeline.

**After:**
```text
┌─ Row 1 ─────────────────────────────────────────────────────────────┐
│ [ ▣ Buildings ] [ Landscape ]     Flow: [verb] [verb] [verb]        │
├─ Row 2 ─────────────────────────────────────────────────────────────┤
│ What ships: …          Pipeline: [pill][pill][pill][pill]  hint    │
└─────────────────────────────────────────────────────────────────────┘
│ NOTEBOOK (work area)                                                 │
```

**Saves ~90px vertical at MIN.**

### SH-4 — Left gutter alignment

All chrome bands use `GAP_MD` (8px) left inset. Pipeline bar currently misaligned — fix.

### SH-5 — Status log priority

- Pack log `side=BOTTOM`; cap expanded height.
- Notebook always keeps vertical priority — log never pushes primary controls off-screen.

### SC-1 — Tab scroll policy

- Default `ScrollableFrame(enable_horizontal=False)` for form tabs.
- Horizontal scroll only inside 2D widgets (footprint canvas, atlas preview grid).

---

## 2. Assembly tab deltas (P0 — worst offender)

### AS-1 — Setup strip collapsible

**Before:** ~320–520px pre-work stack above 3-pane editor.

**After:** One collapsible **Setup** strip (default **collapsed** after first Assembly):
- Generate rows · building style · material source · Load/Save · Check schema · Run ship check · Preview
- 3-pane workspace starts near top of tab

### AS-2 — Metadata flow collapsed default

`metadata_flow_panel._initial_expanded` → **False** everywhere. One-line collapsed hint only.

### AS-3 — Footprint canvas overflow

**Before:** Canvas `width=280` + two legends side-by-side in 215px pane → ~295px overflow.

**After:**
- Canvas `fill=BOTH` — tracks pane width
- Legends **below** canvas (vertical stack)
- "Iteration diff" → collapsible, shown only after iterate op

### AS-4 — Inspector slot previews 2×2

**Before:** 4 thumbnails in 1×4 row (overflows 215px pane).

**After:**
```text
┌────┬────┐
│mod │mat │
├────┼────┤
│comb│ctx │
└────┴────┘
```
Thumb cell size: use `PREVIEW_SIZE` token; min inspector pane `PANE_MIN_DETAIL` (280).

### AS-5 — Generate grid regularization

2-column label/field grid with `GAP_MD` gutter. Iterate + shape bias remain collapsibles inside Setup.

### AS-6 — File ops grouping (P2)

`Load/Save` | path | `Check schema` · `Run ship check` | Preview — separated by `ttk.Separator`.

---

## 3. Wireframe — Assembly after deltas

```text
┌─ Assembly ─────────────────────────────────────────────────────────────┐
│ ▸ Setup (Generate · style · file ops)                    [collapsed]   │
├──────────────────┬──────────────────┬───────────────────────────────────┤
│ Footprint        │ Materials        │ Inspector                         │
│ [placements h5]  │ [search/cat]     │ [2×2 piece previews]              │
│ ┌──────────────┐ │ [profile list]   │ [Selected piece — edit]           │
│ │   CANVAS     │ │                  │ [▸ Tags]                          │
│ └──────────────┘ │                  │                                   │
│ Legend: W D C R Y│                  │                                   │
│ ▸ Iteration diff │                  │                                   │
└──────────────────┴──────────────────┴───────────────────────────────────┘
```

**Responsive:** below ~1100px → 2 panes: `[Footprint | Inspector]`; Materials as inspector sub-tab.

---

## 4. Other tab deltas

| ID | Tab | Delta | Sev |
|:---|:---|:---|:---:|
| MAT-1 | Materials | Flatten `studio_tree` nested paned OR raise `nav` min ≥ 330 | P0 |
| MAT-2 | Materials | Preview-modes min 120; metadata collapsed (AS-2) | P1 |
| VAR-1 | Variants | Convert raw `ttk.Panedwindow` → `aps_paned` + mins 180/280 | P0 |
| ATL-1 | Atlas | Left control rail + right preview pane (`horizontal_paned`); log/lod0 → collapsible Advanced | P0 |
| ATL-2 | Atlas | Drop phantom `width=52` on entries | P1 |
| LG-1 | Grammar | 3→2 pane below ~1100px (tree+inspector notebook, graph full-width) | P0 |
| LG-4 | Landscape | Panel `padding=INSET_PANEL` (8) — match Buildings tabs | P2 |
| CAT-1 | Catalog | 5-button row → bottom toolbar + overflow menu below 420px | P1 |

---

## 5. Pane minsize tokens (anti-starve)

| Token | px | Use |
|:---|:---:|:---|
| `PANE_MIN_LIST` | 220 | list/tree nav |
| `PANE_MIN_DETAIL` | 280 | inspector/detail |
| `PANE_MIN_CANVAS` | 320 | footprint / graph / atlas 2D |

**Guard:** child nested-pane sum ≤ parent `minsize` (catches MAT-1 class).

---

## 6. Verification (headless)

`test_aps_min_window_layout.py` must assert at 960×600:

1. No tab-level horizontal scrollbar mapped
2. No child `winfo_reqwidth` > parent pane width
3. Assembly footprint canvas visible (bbox intersects notebook viewport) at 1280×800

Pixel feel = **NEEDS-DISPLAY** (operator).

---

## 7. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-17 |

**@coder-mcp:** cite finding IDs (SH-1, AS-1, …) in commit messages.
