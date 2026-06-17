# DES-APS-CHROME-MOCKUP-001 — APS chrome mockup → Tk spec `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-CHROME-MOCKUP-001** |
| **Blocks** | **APS-E1-CHROME-001** |
| **Parent** | [`design_aps_domain_ia_sign_v1.md`](design_aps_domain_ia_sign_v1.md) · [`design_aps_uiux_style_quality_20260616_v1.md`](design_aps_uiux_style_quality_20260616_v1.md) |
| **Date** | 2026-06-16 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |

---

## Full-window mockup (1280×800 target)

```text
┌─ Art Pipeline Suite ─────────────────────────────────────────────────────────────┐
│ Lane  (●) Buildings   ( ) Landscape     │ Buildings lane │  ← segmented + chip   │
├────────────────────────────────────────────────────────────────────────────────────┤
│ Flow  [Send to Assembly] [Bake variants] [Pack atlas]                              │
├────────────────────────────────────────────────────────────────────────────────────┤
│ ▌ Ship truth: assembly_snapshot (materials + tags). Sidecar and atlas are inputs. │ ← left border = lane tint
├────────────────────────────────────────────────────────────────────────────────────┤
│ Pipeline  [✓ Catalog valid] [◐ Assembly saved (QC not run)] [○ Materials pending]…│ ← pills, not plain text
│           Keyframe bake is behind Atlas — …                                        │
├────────────────────────────────────────────────────────────────────────────────────┤
│ ╭─ Catalog ─╮ Assembly │ Materials │ Variants │ Atlas ╯                           │ ← selected tab underline = lane tint
│ │  (tab body)                                                                      │
├────────────────────────────────────────────────────────────────────────────────────┤
│ Jobs: (idle)                                                    [Cancel]           │
│ ▸ Status log                                                                       │
└────────────────────────────────────────────────────────────────────────────────────┘
```

**Landscape lane:** same shell; teal chip `Landscape lane`; 4 tabs `Presets · Grammar · States · Atlas`; flow = Generate grammar · Bake states · Pack LG-5 atlas.

---

## 1. Segmented lane control

### Widget

| Piece | Tk implementation |
|:---|:---|
| Container | `ttk.Frame` `padding=(PAD_MD, PAD_SM, PAD_MD, 0)` |
| Label | `ttk.Label` text `Lane:` `font=FONT_UI_BOLD` |
| Segments | Two `ttk.Radiobutton` in group — **not** menu |
| Active segment | Filled background + lane underline (see style) |
| Lane chip | `ttk.Label` right of segments — always shows **word** |

### Copy

| Segment | Label | Selected prefix |
|:---|:---|:---|
| Buildings | `Buildings` | `▣ Buildings` (filled square = active) |
| Landscape | `Landscape` | `▣ Landscape` when active |

Chip text: `Buildings lane` / `Landscape lane` — `foreground=COLOR_LANE_*`.

### ttk style (`init_aps_ttk`)

```python
# NEW styles — @coder-mcp
style.configure("Aps.Lane.TRadiobutton", padding=(12, 4), font=FONT_UI)
style.map("Aps.Lane.TRadiobutton",
    background=[("selected", COLOR_INPUT_BG), ("!selected", COLOR_PANEL_BG)],
    foreground=[("selected", COLOR_LANE_BUILDING), ...],  # per value via widget binding
)
```

**Implementation note:** Radiobutton `foreground` on select is lane-specific — set in `_apply_lane` on both segment widgets + chip. Inactive segment: `COLOR_MUTED` text, `COLOR_PANEL_BG` fill. Active: `COLOR_INPUT_BG` fill + **3px bottom border** simulated via `tk.Frame` underline child (`height=3`, `bg=COLOR_LANE_*`).

### Keyboard

`Ctrl+1` / `Ctrl+2` per [`design_aps_domain_a11y_v1.md`](design_aps_domain_a11y_v1.md).

---

## 2. Authority strip

| Token | Value |
|:---|:---|
| Widget | `ttk.Frame` + `ttk.Label` `textvariable=_authority_var` |
| Font | `FONT_HINT` |
| Left border | `tk.Frame` width **4px** `bg=COLOR_LANE_BUILDING` or `COLOR_LANE_LANDSCAPE` |
| Padding | `PAD_MD` horizontal |

Strings: from `domain_router.AUTHORITY_BY_LANE` (already on disk).

---

## 3. Pipeline pills

Pills replace flat `ttk.Label` pipeline steps. See [`design_aps_pipeline_pills_v1.md`](design_aps_pipeline_pills_v1.md) for copy.

### Pill chrome

| Property | Buildings | Landscape |
|:---|:---|:---|
| Shape | `tk.Frame` relief `RIDGE` borderwidth **1** `padx=6 pady=2` | same |
| Background idle | `#ffffff` | `#ffffff` |
| Background pass | `#f0faf0` | `#f0faf0` |
| Background warn | `#fff8ee` | `#fff8ee` |
| Background fail | `#fff0f0` | `#fff0f0` |
| Text | `FONT_UI` 9pt — `{glyph} {Step} {state_word}` | same |
| Spacing | `padx=4` between pills | same |
| Tooltip | `pipeline_{step_key}` | same |

**Class name:** `PipelinePill` in `pipeline_status_bar.py` or `aps_chrome.py`.

### Layout

```text
Pipeline:  [pill] [pill] [pill] [pill] [pill?]     {lane hint — FONT_HINT, not a pill}
```

Landscape optional **Stamp** pill after Atlas (register/map stamp) — see pipeline pills doc.

---

## 4. Tab chrome (Notebook)

### Dual notebook (signed)

- `_notebook_buildings` — 5 tabs, never relabel to landscape names.
- `_notebook_landscape` — 4 tabs; **Materials tab absent**.

### Selected tab styling

| State | Style |
|:---|:---|
| Selected tab | `background=COLOR_INPUT_BG` + **bottom accent** 3px `COLOR_LANE_*` |
| Unselected | `background=COLOR_PANEL_BG` |
| Padding | `(12, 6)` — existing |
| Font | `FONT_UI` |

```python
style.configure("Aps.TNotebook", ...)
style.configure("Aps.TNotebook.Tab", padding=(12, 6))
# Map selected tab: add underline frame under notebook tab strip OR use clam tab bordercolor
style.map("Aps.TNotebook.Tab",
    bordercolor=[("selected", COLOR_LANE_BUILDING)],  # swap per lane in _apply_lane
)
```

**Lane switch:** call `style.map` with `COLOR_LANE_BUILDING` vs `COLOR_LANE_LANDSCAPE` when `_apply_lane` runs.

### Tab labels (fixed)

| Lane | Tabs |
|:---|:---|
| Buildings | Catalog · Assembly · Materials · Variants · Atlas |
| Landscape | Presets · Grammar · States · Atlas |

---

## 5. Flow bar

Unchanged layout; lane-scoped frame swap (`_flow_buildings` / `_flow_landscape`). Buttons use `ttk.Button` default; primary verb (first) may use `style="Aps.Primary.TButton"` with `foreground=COLOR_ACCENT` border — optional P2.

---

## 6. Token map (implementer checklist)

| Surface | Tokens |
|:---|:---|
| Lane chip fg | `COLOR_LANE_BUILDING` / `COLOR_LANE_LANDSCAPE` |
| Authority border | same |
| Tab underline | same |
| Pill pass fg/bg | `COLOR_PASS` / `#f0faf0` |
| Pill warn | `COLOR_WARN` / `#fff8ee` |
| Pill fail | `COLOR_FAIL` / `#fff0f0` |
| Pill pending | `COLOR_MUTED` / `#ffffff` |
| Section headers | `FONT_SECTION` + `COLOR_ACCENT` |
| Tab H1 inside panel | `FONT_TITLE` (when added) |

---

## 7. Acceptance (matches mockup)

| ☐ | Criterion |
|:---:|:---|
| ☐ | Segmented lane visible above Flow; chip shows lane **word** |
| ☐ | Authority strip left border tints per lane |
| ☐ | Pipeline steps render as **pills** with bg + glyph + word |
| ☐ | Selected notebook tab shows lane-colored underline |
| ☐ | Landscape = 4 tabs only; Buildings = 5 tabs |
| ☐ | No footprint grid on Landscape Grammar tab |

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-16 |

```text
DES-APS-CHROME-MOCKUP-001 Q✓ — unblocks APS-E1-CHROME-001
```
