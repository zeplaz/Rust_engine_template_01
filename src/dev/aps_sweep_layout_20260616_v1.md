# APS Sweep — LAYOUT & DENSITY dimension `v1`

| Field | Value |
|:---|:---|
| **Program** | APS-SWEEP-LAYOUT-001 (1 of 4 parallel dimension audits — this one owns **spatial layout only**) |
| **Owner** | `@designer` |
| **Date** | 2026-06-16 |
| **Scope** | Spatial composition of every APS surface: grouping/alignment, whitespace rhythm, visual hierarchy, `Panedwindow` ratios + pane minsizes, control placement, scroll regions, behavior at `MIN_WINDOW_SIZE` (960×600) vs maximized. **Not** text/copy, tab-design IA, or visual style (parallel audits own those). |
| **Grounding** | Live read of `app.py`, `aps_theme.py`, `catalog.py`, `assembly_panel.py`, `materials_panel.py`, `variants_panel.py`, `atlas_panel.py`, `material_library_widget.py`, `footprint_canvas.py`, `slot_preview_panel.py`, `assembly_preview_panel.py`, `atlas_preview_panel.py`, `metadata_flow_panel.py`, `aps_collapsible.py`, `aps_paned.py`, `scrollable.py`, `status_log_panel.py`, `job_strip.py`, `pipeline_status_bar.py`, `landscape_*_panel.py`. Builds on `design_aps_uiux_style_quality_20260616_v1.md` (does not repeat its IA/style/token-role work). |

**Window contract (from `aps_theme.py`):** `DEFAULT_WINDOW_SIZE = (1280, 800)` launch · `COMFORTABLE_MAX = (1440, 900)` · `MIN_WINDOW_SIZE = (960, 600)` regression floor. All min-fit math below is against **960 wide × 600 tall**.

---

## 0. Method — how "cramped/overflow/starve" was measured

I summed declared pane `minsize` + sash widths (`SASH_WIDTH = 7`, 2 sashes = 14px) + the vertical scrollbar the `ScrollableFrame` always packs (~17px) + tab/panel padding (`padding=8` each side + notebook `padding=(4,2)`), and compared to the **usable width** at MIN (≈ 960 − 17 scrollbar − ~24 tab/panel padding ≈ **~919px**) and usable height at MIN (≈ 600 − chrome stack). Chrome height is measured from `app.py`'s pack order. Density = controls-per-vertical-band before the work area begins.

---

## 1. App shell (`app.py`) — the chrome stack

The shell packs this fixed vertical stack **above** the notebook, top to bottom (each is `fill=tk.X`, none collapsible except the status log):

| # | Band | Source | Approx height | Notes |
|:--|:---|:---|:--|:---|
| 1 | Lane bar (`Lane: [Buildings][Landscape] + chip`) | `_build_lane_bar`, `padding=(8,6,8,0)` | ~32px | |
| 2 | Lane underline | `tk.Frame height=3` | 3px | |
| 3 | Flow bar (`Flow: [3 verbs] | caveat label`) | `_build_flow_bars`, `padding=8` | ~38px | |
| 4 | Authority strip (`Ship truth: …` + 4px border) | `_build_authority_strip`, `padding=(8,0,8,2)` | ~24px | wraps to 2 lines below ~720px → ~40px |
| 5 | Pipeline bar (`Pipeline: [5 pills] + hint`) | `PipelineStatusBar`, `padding=(0,4)` | ~32px | |
| 6 | Job strip | `JobStrip` | 0px idle / ~28px running | correctly hidden when idle ✓ |
| 7 | Status log (collapsed header) | `_pack_status_log`, `CollapsibleSection` | ~28px | |
| 8 | **Notebook (work area)** | `_notebook_container`, `padx=8 pady=4` | remainder | |

**Total fixed chrome ≈ 157px (collapsed log, idle job) → ~189px (authority wraps + job running).**

- At **800px launch height**: chrome eats **~20–24%** before any tab content. Tolerable.
- At **MIN 600px height**: chrome eats **~26–31%**, leaving the notebook **~415–445px**. Every tab that itself stacks pre-work chrome (Assembly, Atlas, Catalog) then has almost no room for its actual editing surface → vertical starvation. **This is the structural root cause of the MIN-window pain.**

### Findings — shell

| ID | Surface | Issue | Sev | Recommended delta |
|:--|:---|:---|:--|:---|
| SH-1 | app shell | 5 always-on full-width chrome bands (lane, flow, authority, pipeline, +log header) stack to ~157px before the work area; at MIN that is ~26–31% of height. No band is collapsible or merged. | **P0** | **Merge bands 1+3 into one top bar** (lane segment on the left, flow verbs on the right of the same row) and **fold the authority strip into the pipeline bar row** as a left-aligned caption. Target ≤ 2 chrome rows (~64px) above the notebook. Saves ~90px → +20% work area at MIN. |
| SH-2 | app shell | Lane underline is a separate 3px `tk.Frame` packed in a different parent (`wrap`) from the lane buttons (`bar`) — the "active lane" tint underlines the whole bar width, not the active segment, so it reads as a divider rule, not a selection indicator. | P2 | Move the underline under the active segment only (or drop it; the filled `▣` glyph + chip already carry lane state). Frees the 3px + ambiguity. |
| SH-3 | app shell | Authority strip wraps to a 2nd line below ~720px width (`wraplength` via `wrap_for_widget(minimum=480)`), silently adding ~16px and shifting everything below at narrow widths. | P2 | After SH-1 merge, give it a fixed single-line height with ellipsis + tooltip for full text, or keep it as a one-line caption inside the pipeline row. |
| SH-4 | app shell | `_notebook_container` uses `padx=8 pady=4` but the chrome bands above use a mix of `padding=8`, `padding=(8,6,8,0)`, `padding=(8,0,8,2)`, `padding=(0,4)` — the left edge of pipeline pills (`padding=(0,4)`, no left pad) does **not** align with the lane/flow/authority left edges (8px). Vertical left-edge is ragged. | P1 | Single left gutter token (`PAD_MD=8`) for **all** chrome bands so labels start on one vertical line. Pipeline bar must inherit the 8px left pad. |
| SH-5 | status log | Status log is a `CollapsibleSection` packed with `fill=tk.BOTH, expand=True` inside a frame packed `expand=False`. When expanded it competes with the notebook for vertical space but is **below** it in pack order, so expanding the log at MIN can push the notebook's bottom controls off-screen with no scroll recovery on the shell itself. | P1 | Cap expanded log height (it already requests `height=5`); pack the log frame `side=tk.BOTTOM` explicitly and never let it `expand` past a fixed max so the notebook keeps priority. |

---

## 2. Assembly tab (`assembly_panel.py`) — **worst offender**

This is the heaviest surface and the worst layout offender. Before the 3-pane editing workspace even appears, the panel stacks **seven** full-width pre-work blocks (all `fill=tk.X`):

```
intro label (wrap 900)                              ~32px
MetadataFlowPanel (default-EXPANDED first run)      ~200px  ← Text height=10
GrammarBuildSetPanel                                ~variable
"Material authority" LabelFrame (2 labels)          ~56px
"Generate" LabelFrame:                              ~tall
   gram_row (checkbox + Archetype + District)       ~30px
   row (StylePack + Tier)                           ~30px
   row2 (Footprint + Floors + Seed + Generate btn)  ~30px
   next_step label                                  ~24px
   "Iterate grammar (advanced)" collapsible         ~28px hdr
   "Massing pressure (advanced)" collapsible        ~28px hdr
file_row (Load/Save/Validate/P0/path/Preview)       ~32px
─────────────────────────────────────────────────────────
THEN: 3-pane horizontal workspace (the actual editor)
```

**Pre-workspace chrome inside the tab ≈ 320–520px** (worst case with metadata-flow expanded). Stacked on top of the **shell** chrome (~157px), the actual 3-pane editor at MIN-height (600px) gets **near-zero or negative** vertical space — the user must scroll the whole tab (via `ScrollableFrame`) just to see the footprint grid. The footprint grid, the single most important object on this tab, is **below the fold** on first open at the default window.

### The 3-pane workspace itself

`horizontal_paned` with `(footprint, materials, inspector)` minsizes:
```python
_fp_min, _mat_min, _insp_min = (215, 195, 215) if _min_w <= 1024 else (240, 220, 260)
# at MIN (960): 215 + 195 + 215 = 625 + 2 sashes(14) = 639
```
Initial sash fractions `(0.30, 0.28)` give inspector ~42%. The comment claims it fits MIN; it does (639 < ~919) — **but** see FP-1: a child of the footprint pane overflows its own pane regardless.

### Findings — Assembly

| ID | Surface | Issue | Sev | Recommended delta |
|:--|:---|:---|:--|:---|
| AS-1 | Assembly pre-work stack | ~320–520px of generate/auth/metadata/file chrome stacked above the editor pushes the footprint grid below the fold at default size and off-screen at MIN. Primary work surface is not where the eye/hand expects (top). | **P0** | Move **Generate / Material-authority / file-ops into a left "Setup" rail or a collapsible top strip that defaults COLLAPSED after first snapshot**; promote the 3-pane workspace to fill from near the top. The grid must be visible without scrolling at 1280×800. |
| AS-2 | MetadataFlowPanel | Defaults **expanded** on first view of each context (`_initial_expanded` sets seen+expanded=True on first run), adding ~200px to an already-tall tab. It appears on Catalog, Assembly, Materials, Atlas, and all 3 Landscape tabs — so the first-run experience of every tab is "wall of explanatory text pushing the work down." | **P0** | Default **collapsed** everywhere; keep the one-line `_collapsed_hint` (already exists) as the resting state. Let the user opt in. (One-line change in `_initial_expanded`.) |
| AS-3 | footprint pane | `FootprintCanvas` packs a fixed `width=280` canvas **plus** two `ttk.LabelFrame` legends (`Cell tokens`, `Iteration diff`) side-by-side (`side=tk.LEFT`) inside the footprint pane whose minsize is **215**. 280 (canvas) + ~120 + ~90 (two legends) ≈ **490px of horizontal content in a 215px pane** → forced horizontal scroll / clipping of the legends inside the pane at every window size. | **P0** | Stack the legends **below** the canvas (vertical), or collapse "Iteration diff" into a collapsible shown only after an iterate op. Make the canvas width track the pane (`fill=BOTH`) instead of fixed 280. See FP-1 detail in §8. |
| AS-4 | inspector pane | The inspector pane stacks SlotPreview (4 thumbnails in a row) + AssemblyPreview (nested 2-pane) + "Selected slot — edit" (grid with collapsible tags) + Grammar inspector collapsible + validation label — five heavyweight regions in one scrolling pane with `minsize=215`. SlotPreview's 4 thumb cells (`PREVIEW_SIZE`-wide each) overflow a 215px pane horizontally just like AS-3. | P1 | Put SlotPreview thumbnails in a 2×2 grid (not 1×4 row) so they fit a narrow pane; or give the inspector pane a higher minsize (260) and reduce footprint/materials mins. |
| AS-5 | Generate LabelFrame | Two "(advanced)" collapsibles (Iterate, Massing pressure) live **inside** the Generate frame, so collapsing them still leaves the 3 generate rows + next-step + 2 headers. Related generate controls (footprint dims, floors, seed, style, tier, archetype, district) are split across 3 rows with inconsistent inter-label padding (`padx=(12,0)`, `(8,0)`, `4`). | P1 | Regularize to a 2-column label/field grid (one `PAD_MD` column gutter) so fields align vertically; group the 3 rows under one header. |
| AS-6 | file_row | The destructive/heavy ops (P0 gate, Validate) sit inline with Load/Save and the path label and Preview, left-to-right with `padx=2` — no separation between benign (Load) and gate (P0) actions, and the path label is jammed between buttons (`padx=8`) breaking the button group. | P2 | Group file ops (Load/Save) | path | validate group (Validate/P0) | preview — with `ttk.Separator` between groups; right-align Preview. |

---

## 3. Catalog tab (`catalog.py`)

2-pane `horizontal_paned`: left `Modules` list (weight 1, minsize 220), right detail (weight 3, minsize 360). Above the paned: MetadataFlow + a filter `bar` (Batch combo, Category combo, Refresh).

| ID | Surface | Issue | Sev | Recommended delta |
|:--|:---|:---|:--|:---|
| CAT-1 | right pane | The right pane stacks summary + sidecar-truth + validation labels + an **inner `ttk.Notebook`** (AssetSpec / Index) + an actions row of **5 buttons** (`Validate GLB`, `Save metadata`, `Reindex library`, `Preview in browser`, `3D preview`). A nested notebook inside a paned inside the tab notebook = 3 levels of tabbed/paned nesting; the 5-button actions row wraps awkwardly when the right pane is dragged narrow. | P1 | Move the 5 actions into a single bottom toolbar of the right pane with overflow into an "⋯ More" menu below ~420px pane width; keep Validate GLB + Save as primary, demote browser/trimesh previews. |
| CAT-2 | left list | Custom canvas list of thumbnail rows (`_list_inner`) — each row is a thumb + 2-line label. Fine, but the list pane minsize (220) + right minsize (360) = 580 + sash = ~594, comfortable at MIN. No overflow. | — | OK. |
| CAT-3 | hardcoded wraplengths | `wraplength=680` literals on summary/sidecar/validation labels even though `track_wraplength` is also wired (minimum 320) — the 680 is a dead initial value that fights the dynamic update on first paint. | P2 | Drop the literal; rely on `track_wraplength`. |

---

## 4. Materials tab (`materials_panel.py` + `material_library_widget.py` studio_tree)

Outer = `vertical_paned`: library (weight 3, minsize 280) over preview-modes (weight 1, minsize 180). The library uses `studio_tree` layout, which is itself a `horizontal_paned`:
```
outer vertical: [ library  (min 280 tall) ]
                [ preview-modes (min 180 tall) ]
library studio_tree (horizontal): nav (min 240) | preview-strip (min 320)   = 560 + sash
   nav is ITSELF a horizontal_paned: Categories tree (min 140) | Profiles list (min 180) = 320
```

| ID | Surface | Issue | Sev | Recommended delta |
|:--|:---|:---|:--|:---|
| MAT-1 | nested paned starve | The `nav` pane has minsize **240** but contains a nested horizontal paned needing **140 + 180 = 320 + sash(7) = 327** before it can lay out without clipping. A pane that is 240 wide cannot hold a 327-wide child → the Categories tree or Profiles list is clipped / forces internal horizontal scroll until the user widens nav past its sibling. **Three levels of horizontal paned nesting** (tab → studio_tree → nav_row). | **P0** | Raise `nav` minsize to ≥ 330, or (better) **flatten**: make Categories / Profiles / Preview three siblings of ONE horizontal paned instead of a paned-in-a-paned. Removes a sash and the starve. |
| MAT-2 | vertical split ratio | Preview-modes pane (weight 1, min 180) under a weight-3 library. At MIN height (600) minus shell chrome (157) minus the tab's intro+banner+metadata (~250 first-run) the vertical paned gets ~190px total — less than library's 280 min + preview's 180 min = 460. The vertical paned **cannot satisfy both mins** → one pane is crushed to its min and the other clips. | P1 | Reduce preview-modes min to 120; collapse metadata-flow (AS-2) to reclaim ~200px; consider preview-modes as a collapsible bottom drawer rather than an always-on pane. |
| MAT-3 | toolbar density | `MaterialLibraryWidget` toolbar packs 5 buttons left + 1 right (`Use in Assembly`) + a filter row (Search entry + Category combo) + a hint label — 8 controls across 2 rows before the grid. At narrow pane widths the 5-button left group overflows under the right button. | P1 | Move generate/folder/registry into an "⋯" overflow; keep Add + Search + Category visible. |

---

## 5. Variants tab (`variants_panel.py`)

2-pane `ttk.Panedwindow` (raw, **not** `aps_paned` — no minsizes, no visible-sash style): left Variants list (weight 1), right editor (weight 2). Above: intro + banner + a `top` row of **6 buttons** (Load / Load example / New from assembly / Save JSON / Save RON / Validate) + path label + status.

| ID | Surface | Issue | Sev | Recommended delta |
|:--|:---|:---|:--|:---|
| VAR-1 | raw paned | Uses `ttk.Panedwindow(orient=HORIZONTAL)` directly, bypassing `aps_paned.add_pane` → **no minsize floor** (panes can be dragged to 0px, hiding the list or editor entirely) and **no visible sash** (`Aps.*.Sash` style not applied) — inconsistent with every other tab's draggable, visible divider. | **P0** | Convert to `horizontal_paned` + `add_pane(..., minsize=180/280)`. Pure consistency + anti-starve fix; one-import change. |
| VAR-2 | 6-button top row | Six file-op buttons in one undelimited `padx=2` row; `Save JSON` and `Save RON` are two buttons where a format toggle would do; no separation between load-group and save-group. | P1 | Collapse Save JSON/RON into one `Save ▾` split-button; separator between Load-group and Save/Validate-group. |
| VAR-3 | layer_row grid | The `Layers` LabelFrame uses a 4-column grid mixing combos (width 10–12), a `Scale` (`sticky=EW`), and full-width entries (`columnspan=3`) — column widths are driven by the widest cell so the `Lighting`/`Power` labels in row 0 don't align with `Damage state`/`Fill` labels in lower rows. Visually ragged grid. | P2 | Fix column 0 width (label gutter) and column 1 min via `columnconfigure`; keep entries in their own full-width rows below the combo grid, not interleaved. |
| VAR-4 | agent patch | `patch_text` Text `height=8` packed `expand=True` competes with the layer grid in the same right pane; at MIN the layer grid (7 rows) + agent strip + 8-line text overflow the pane → tab scroll. | P2 | Put the agent patch strip in a collapsible (default collapsed) — it's an advanced/occasional flow. |

---

## 6. Atlas tab (`atlas_panel.py`)

A **long single-column stack** of ~14 packed bands, no paned, no top-level grouping:
intro → domain-banner → register-row → MetadataFlow → batch-row → Run-batch-btn → inline-status → tile-row(folder+rename) → Pack-btn → Refresh-btn → qc-row(2 btns) → atlas-qc-label → **AtlasPreview (expand=True)** → lod-row → debug/hint → "Log" label → Log Text(height=6).

| ID | Surface | Issue | Sev | Recommended delta |
|:--|:---|:---|:--|:---|
| ATL-1 | flat single column | ~14 bands, no grouping; the only `expand=True` region (AtlasPreview, the thing the artist wants to SEE) sits in the **middle** of the stack with ~8 control bands above and a 6-line log + lod0 + debug below it — so the preview is squeezed and the eye has no anchor. Primary actions (Run batch / Pack / Validate) are scattered across 4 separate rows. | **P0** | Restructure into a **left control rail (batch / pack / qc inputs) + right preview pane** (`horizontal_paned`), preview gets the expand. Group the 3 primary verbs (Run batch · Pack · Validate) into one action toolbar. Demote lod0-batch + debug + log into a collapsible "Advanced / Log" bottom strip. |
| ATL-2 | entry widths | `batch_entry` and `folder_entry` are `width=52` but also `fill=tk.X, expand=True` — the 52-char width is a phantom min that, combined with the trailing Browse/From-set buttons, can push the row past the panel width at MIN, forcing the tab's horizontal scroll. | P1 | Drop the `width=52`; let `fill=X expand` own the sizing. |
| ATL-3 | two log surfaces | Atlas has its own `log_text` (height 6) **and** the shell has the Status log — duplicated log real estate, the atlas one always-on at the bottom eating ~110px. | P2 | Remove the panel-local log; route to the shell status log (already receives `_on_log`). Reclaims ~110px for the preview. |
| ATL-4 | register row vs domain | "Check landscape register" button + status appears in the **Buildings** atlas too (only meaningful in Landscape lane) — control present where it doesn't apply. | P2 | Hide the register row unless lane == landscape (set_domain already runs). |

---

## 7. Landscape tabs — Presets / Grammar / States

These are newer and partly scaffold. **Grammar repeats the exact mistake the prior style doc warned about** (3-pane that won't fit MIN).

| ID | Surface | Issue | Sev | Recommended delta |
|:--|:---|:---|:--|:---|
| LG-1 | Grammar 3-pane | `horizontal_paned` tree(min 200) | graph-canvas(min 240) | inspector(min 220) = **660 + 2 sashes(14) = 674**. Plus the tab's `ScrollableFrame` vertical scrollbar (~17) + panel padding. At MIN usable width (~919) it fits horizontally **but barely**, and the prior doc explicitly flagged "apply the responsive 3→2-pane rule to Grammar pre-emptively" — not done. Below ~1024 the three mins leave each pane near its floor (graph canvas unusable at 240). | **P0** | Apply the responsive rule: below ~1100px collapse to 2 panes (tree+inspector as a sub-notebook, graph full-width below) — the same fix Assembly needs. Lower graph-canvas min only if it stays legible. |
| LG-2 | Presets detail | List (`fill=BOTH expand`) **above** a `Must-read` LabelFrame packed `fill=X` with 5 stacked `q1..q5` labels + validator. The detail block is fixed-height at the bottom; when the list is long the detail can be pushed below the fold at MIN. Detail and list both want vertical space with no split control. | P1 | Make list + detail a `vertical_paned` so the user can rebalance; or cap the list height and let detail be the expand region. |
| LG-3 | States/Grammar metadata | Both lead with MetadataFlowPanel (default-expanded first run, ~200px) before any control — same AS-2 problem, compounding the already-tight Grammar 3-pane. | P1 | Fixed by AS-2 (default collapsed). |
| LG-4 | left-edge rhythm | Grammar/States/Presets each use `padding=4` on the panel but the shell tabs assume `padding=8` elsewhere (Assembly/Materials/Atlas/Variants use 8). Landscape tabs sit 4px further left than building tabs → the work area left edge **shifts** when switching lanes. | P2 | Standardize panel `padding=PAD_MD(8)` across all tabs so the content origin is stable across lane switch. |

---

## 8. Detail: `FootprintCanvas` overflow (the AS-3 mechanism)

```python
# footprint_canvas.py — workspace packs canvas + 2 legends side-by-side:
workspace = ttk.Frame(self); workspace.pack(fill=BOTH, expand=True)
canvas_wrap.pack(side=LEFT, ...);  self.canvas = Canvas(width=280, height=200)  # FIXED 280
legend     = LabelFrame("Cell tokens").pack(side=LEFT, fill=Y, padx=(10,0))     # ~120
diff_legend= LabelFrame("Iteration diff").pack(side=LEFT, fill=Y, padx=(10,0))  # ~90
# total horizontal demand ≈ 280 + 10 + 120 + 10 + 90 = 510px
# host = footprint pane, minsize 215  →  295px of overflow, clipped/scrolled
```
The canvas also **grows** with footprint size (`width * px + pad`, `px=28`) — a 10-wide footprint = 292px just for the grid. The two legends are reference material, not per-interaction; stacking them under the canvas (or behind a collapsible) removes the overflow and lets the canvas claim the pane width.

---

## 9. `scrollable.py` interaction with forced horizontal scroll

`app._add_scrollable_tab` constructs every tab with `ScrollableFrame(enable_horizontal=True)`. But `_on_canvas_configure` only stretches the interior to canvas width **when horizontal is disabled**:
```python
def _on_canvas_configure(self, event):
    if not self._enable_horizontal:        # ← with horizontal ON, interior width is NEVER synced to canvas
        if self._canvas.itemcget(...,"width") != str(event.width):
            self._canvas.itemconfigure(self._interior_id, width=event.width)
```
**Consequence:** with horizontal enabled, the interior frame sizes to its content's natural width, not the canvas width. Any panel whose content is naturally narrower than the canvas leaves dead space on the right; any panel naturally wider (every overflow above: FP legends, 52-char entries, 4-thumb rows) triggers a **horizontal scrollbar at the bottom of the tab** — exactly the "horizontal-scroll forced at min size" failure the brief asks to flag. Horizontal scroll on a form-style tab is a usability smell (users don't expect to scroll a property panel sideways).

| ID | Surface | Issue | Sev | Recommended delta |
|:--|:---|:---|:--|:---|
| SC-1 | every tab | `enable_horizontal=True` globally + interior-width-not-synced ⇒ tabs can show a horizontal scrollbar and never fill width. Horizontal scroll on property/editor tabs is unexpected. | **P0** | Default tabs to **vertical-only** scroll (`enable_horizontal=False`) so interior tracks canvas width and content reflows; reserve horizontal scroll for genuinely 2D surfaces (atlas preview, footprint canvas) handled inside those widgets, not at the tab level. Fixing the overflows (FP-1, ATL-2, AS-4) then removes the need for tab-level horizontal scroll entirely. |

---

## 10. Overall composition assessment

The suite has **good primitives** (`aps_paned` with visible sashes + minsizes, `aps_collapsible`, `ScrollableFrame` debounced) but applies them **inconsistently** and **stacks too much fixed chrome before work areas**. The dominant failure mode is **vertical**: a tall shell chrome stack (§1) compounds with tall per-tab pre-work stacks (Assembly §2, Atlas §6) so that at the **default** 1280×800 the primary work object is often below the fold, and at **MIN 960×600** it is off-screen, recoverable only by scrolling — and a secondary **horizontal** failure where fixed-width children (footprint legends, 52-char entries, 4-thumb rows) overflow narrow panes and trip the globally-enabled horizontal tab scroll (§9).

Ranked by impact:
1. **Chrome-before-work** (shell + Assembly + Atlas) — the structural problem; everything else is downstream.
2. **Horizontal overflow in narrow panes** (footprint legends, materials nested-paned starve, wide entries) tripping forced horizontal scroll.
3. **Inconsistent application of the paned primitives** (Variants raw paned with no minsize; Landscape padding=4 vs 8; pipeline-bar left edge unaligned).
4. **No spacing rhythm** — **399** hardcoded padding/padx/pady/wraplength literals across 26 files (confirmed by grep); the same logical gap is `4` here, `(8,0)` there, `2` elsewhere.

**Worst offender surface: the Assembly tab.** It is the heaviest (966 lines), it owns the ship-truth object (the assembly snapshot), and it commits **three** of the worst layout sins at once: (a) ~320–520px of pre-work chrome above the editor so the footprint grid — the single most important interactive object in the whole suite — is below the fold at default size and off-screen at MIN; (b) a footprint pane whose own child (`FootprintCanvas`) overflows it horizontally by ~295px regardless of window size; (c) an inspector pane stacking five heavyweight regions (4-thumb preview row, nested 2-pane preview, slot-edit grid, grammar collapsible, validation) where the thumbnail row alone overflows the 215px pane. It is where a layout overhaul yields the most usability return.

---

## 11. Recommended spacing/density token set (canonical)

A single 4px-based scale. Codify these in `aps_theme.py` and **migrate the 399 literals onto them** so density is tunable in one place (enables a future compact mode for MIN-window / a11y reduced-density). This extends — and makes concrete — the `PAD_*` table the prior style doc proposed.

| Token | px | Role (the ONE meaning each is allowed) | Replaces literals |
|:---|:--:|:---|:---|
| `GAP_XS` | 2 | intra-control hairline (icon↔label, swatch↔text); never between groups | `padx=2`, `pady=1/2` |
| `GAP_SM` | 4 | control-to-control within a row; label-to-its-field | `padx=4`, `pady=4`, `padding=4` |
| `GAP_MD` | 8 | section/panel inset; row-to-row; the default panel `padding` | `padding=8`, `padx=8`, `pady=8` |
| `GAP_LG` | 12 | inter-**group** separation (button-group ↔ button-group; label-block ↔ control-block) | `padx=(12,0)`, the ad-hoc `(8,…)` group gaps |
| `GAP_XL` | 16 | major region break (chrome band ↔ work area; top toolbar ↔ first section) | currently absent — regions abut |
| `INSET_PANE` | 4 | the padding **inside** every paned child frame (uniform, replaces the mix of `padding=2`/`padding=4`) | pane `padding=2/4` |
| `INSET_PANEL` | 8 | the padding of a **tab root panel** (uniform — fixes LG-4 lane-shift) | tab `padding=4` vs `8` |
| `SASH_WIDTH` | 7 | paned divider (already a token) | EXISTS |
| `ROW_H` | 24 | tree/list row height (already inline) | `Treeview rowheight` |

**Pane minsize tokens (anti-starve floors):**

| Token | px | Role |
|:---|:--:|:---|
| `PANE_MIN_LIST` | 200 | a list/tree navigation pane floor |
| `PANE_MIN_DETAIL` | 320 | a detail/inspector/preview pane floor (so it never crushes below readable) |
| `PANE_MIN_CANVAS` | 260 | a 2D canvas pane floor (footprint / graph / atlas) below which the canvas is unusable |

**Rhythm rule:** the only legal vertical gaps are `GAP_SM` (within a section), `GAP_MD` (between sections), `GAP_LG` (between groups), `GAP_XL` (band↔work). Horizontal label-to-field is always `GAP_SM`; group-to-group is always `GAP_LG`. A guard test (extend `test_aps_ux_polish_density_tokens.py`) should fail on any new literal `padx=`/`pady=`/`padding=` integer outside this set.

---

## 12. Recommended Assembly layout (ASCII wireframe)

Demonstrates AS-1/AS-3/AS-4 fixes: setup chrome collapses, footprint grid is top-left and visible immediately, legends stack under the canvas, inspector thumbs go 2×2.

```text
┌─ Assembly ─────────────────────────────────────────────────────────────────┐
│ ▸ Setup (Generate · grammar · material authority · file ops)   [collapsed]  │  ← AS-1: one collapsible strip, GAP_XL below
│   (expanded shows the 2-col generate grid + Load/Save | Validate/P0 | Prev)  │
├──────────────────────────────────────────────────────────────────────────────┤
│  Footprint & placements    │  Material library      │  Inspector             │
│  ┌───────────────────────┐ │  ┌──────────────────┐  │  ┌──────────────────┐  │
│  │ placements list (h5)  │ │  │ search / category │  │  │ Slot previews    │  │
│  ├───────────────────────┤ │  ├──────────────────┤  │  │ ┌────┬────┐      │  │  ← AS-4: 2×2, not 1×4
│  │                       │ │  │ profile cards /  │  │  │ │mod │mat │      │  │
│  │   FOOTPRINT CANVAS    │ │  │ tree             │  │  │ ├────┼────┤      │  │
│  │   (fills pane width)  │ │  │                  │  │  │ │comb│ctx │      │  │
│  │                       │ │  │                  │  │  │ └────┴────┘      │  │
│  ├───────────────────────┤ │  └──────────────────┘  │  ├──────────────────┤  │
│  │ Legend: W D C R Y     │ │                        │  │ Selected slot    │  │  ← AS-3: legend BELOW canvas
│  │ ▸ Iteration diff      │ │                        │  │ (2-col grid)     │  │
│  └───────────────────────┘ │                        │  │ ▸ Sem/var tags   │  │
│   min PANE_MIN_CANVAS(260)    min PANE_MIN_LIST(200)    min PANE_MIN_DETAIL  │
│                                                            (320)             │
└──────────────────────────────────────────────────────────────────────────────┘
Below ~1100px: drop to 2 panes — [Footprint | Inspector], Materials as a sub-tab of Inspector.
```

---

## 13. Severity rollup

| Sev | Count | IDs |
|:--|:--|:---|
| **P0** | 9 | SH-1, AS-1, AS-2, AS-3, MAT-1, VAR-1, ATL-1, LG-1, SC-1 |
| **P1** | 11 | SH-4, SH-5, AS-4, AS-5, CAT-1, MAT-2, MAT-3, VAR-2, ATL-2, LG-2, LG-3 |
| **P2** | 11 | SH-2, SH-3, AS-6, CAT-3, VAR-3, VAR-4, ATL-3, ATL-4, LG-4 |

---

## 14. Required engine hooks (for the master overhaul plan / @coder-mcp)

- `aps_theme.py`: add `GAP_*` / `INSET_*` / `PANE_MIN_*` tokens (§11); migrate the 399 literals; extend `test_aps_ux_polish_density_tokens.py` to fail on out-of-set padding integers.
- `app.py`: merge chrome bands (SH-1) — lane+flow into one top bar, authority into the pipeline row; uniform `INSET_PANEL` left gutter (SH-4); cap status-log expansion (SH-5).
- `metadata_flow_panel.py`: flip `_initial_expanded` default to **collapsed** (AS-2) — fixes the first-run vertical bloat on all 7 tabs at once.
- `assembly_panel.py`: extract Setup chrome into a default-collapsed strip (AS-1); 2×2 slot-preview (AS-4); 2-col generate grid (AS-5); separated file ops (AS-6).
- `footprint_canvas.py`: stack legends below a pane-tracking canvas (AS-3/FP-1).
- `material_library_widget.py`: flatten studio_tree to one horizontal paned or raise `nav` minsize ≥ 330 (MAT-1).
- `variants_panel.py`: convert raw `ttk.Panedwindow` to `aps_paned.horizontal_paned` + `add_pane` minsizes (VAR-1).
- `atlas_panel.py`: control-rail + preview-pane split; demote log/lod0/debug to a collapsible (ATL-1); drop phantom entry widths (ATL-2); route to shell log (ATL-3).
- `landscape_grammar_panel.py`: responsive 3→2-pane below ~1100px (LG-1).
- `scrollable.py` / `app._add_scrollable_tab`: default `enable_horizontal=False`; sync interior width in all cases (SC-1).

## 15. Diagnostics required

- **MIN-window layout assert** (the prior doc's DoD row 9 is "MANUAL until a layout assert exists"): a headless test that builds each tab at 960×600 and asserts no child `winfo_reqwidth` exceeds its pane and no tab-level horizontal scrollbar is mapped. This is the witness that catches FP-1 / MAT-1 / ATL-2 regressions.
- **Pane-min-vs-child assert**: for every `add_pane(minsize=N)`, assert the child's summed nested-pane mins ≤ N (catches MAT-1 class).
- **Density-token guard** extension (§11) to forbid raw padding literals.

---

## Sign-off

```text
APS-SWEEP-LAYOUT-001 complete (LAYOUT/DENSITY dimension only)
Worst offender: Assembly tab (chrome-before-work + footprint overflow + inspector overflow)
Root cause: tall shell chrome stack (~157px) × tall per-tab pre-work stacks → work object below fold at default, off-screen at MIN
Token recommendation: 4px scale GAP_XS..GAP_XL + INSET_PANE/PANEL + PANE_MIN_* ; migrate 399 literals
P0 ×9 · P1 ×11 · P2 ×11
```
