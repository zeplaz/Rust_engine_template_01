# APS UI/UX Preview Spec `v1` — OVR-DES-P55-PREVIEW-SPEC-001

| Field | Value |
|:---|:---|
| **ID** | **OVR-DES-P55-PREVIEW-SPEC-001** |
| **Program** | PLAN-APS-UIUX-OVERHAUL-001 |
| **Phase** | P5.5 (preview & presentation) |
| **Date** | 2026-06-17 |
| **Owner** | `@designer` |
| **Authority** | [`aps_design_system_v1.md`](aps_design_system_v1.md) §3.4–§3.5 |
| **Inputs** | [`plan_aps_uiux_overhaul_20260616_v1.md`](plan_aps_uiux_overhaul_20260616_v1.md) §P5.5 · layout sweep AS-4 |
| **Implements** | `OVR-P55-PREVIEW-001` |
| **Verdict** | **PASS** — preview contract for `@coder-mcp` |

```text
OVR-DES-P55-PREVIEW-SPEC-001 Q✓
Unblocks: OVR-P55-PREVIEW-001
```

---

## 0. Problem statement

Preview surfaces behave inconsistently today — different empty/loading/error paths, black thumbnails when `trimesh` absent, silent browser fallback. **One contract** across all preview widgets.

**Hard rule:** never black, never blank, never crash. Every state is **labelled**.

---

## 1. Four-state contract (every preview surface)

| State | Glyph | Label pattern | Background |
|:---|:---:|:---|:---|
| **loading** | `⟳` | `Rendering…` | `COLOR_PANEL_BG` |
| **empty** | `○` | `Nothing selected` / tab-specific empty | `COLOR_INPUT_BG` |
| **error / placeholder** | `◐` | `{reason} — {hint}` | `COLOR_WARN_BG` |
| **result** | `✓` | *(image/grid/canvas)* + fidelity chip | `COLOR_INPUT_BG` |

Implement via shared helper: `preview_surface_state(state, detail) -> (glyph, label, fg, bg)` alongside `status_atom()`.

**Never:** raw black canvas, `None` image, unlabelled gray square.

---

## 2. Fidelity labels (always visible on result)

| Label | Meaning | Surfaces |
|:---|:---|:---|
| **Quick preview** | In-Tk thumbnail; approximate lighting | Slot thumbs, module 3D, material swatch |
| **Interactive 3D** | Browser / three.js; full orbit | Assembly preview |
| **Ship render** | Bake output / atlas tile | Atlas grid cells, variant bake thumb |
| **Layout view** | 2D schematic, not art | Footprint grid, atlas sheet grid |

Chip placement: top-left of preview frame, `FONT_CAPTION`, `COLOR_MUTED` on `COLOR_PANEL_BG` strip.

---

## 3. In-Tk vs browser story

| Channel | When | UX |
|:---|:---|:---|
| **In-Tk** | Default quick-look | Updates on select; async job; loading state |
| **Browser** | Explicit button only | `Preview assembly` / `Open in browser` — never silent fallback |

**Assembly preview button flow:**
1. Click `Preview assembly` → button shows `⟳ Opening preview…`
2. On success → toast `✓ Interactive 3D opened in browser` + optional in-Tk thumb if PNG returned
3. On missing deps → `◐ Preview unavailable — install preview dependencies` (tooltip names dep; not in body)
4. On blank thumb → `◐ Thumbnail unavailable — use Open in browser` (never leave black)

---

## 4. Surface inventory

### 4.1 Slot piece previews (`slot_preview_panel.py`)

| Cell | Content | Empty | Fidelity |
|:---|:---|:---|:---|
| Module | Isolated module mesh | `○ No piece selected` | Quick preview |
| Material | Material on wall+sphere | `○ No material` | Quick preview |
| Combined | Module + material | `○ Select a piece` | Quick preview |
| Context | Placement highlighted | `○ No placement` | Quick preview |

**Layout:** 2×2 grid per [`design_aps_uiux_layout_delta_v1.md`](design_aps_uiux_layout_delta_v1.md) AS-4.

**Degradation:** if `trimesh` absent → all four cells show `◐ Quick preview needs optional 3D library — layout view still works` on `COLOR_WARN_BG`.

### 4.2 Assembly 3D (`assembly_preview_panel.py`)

| State | Copy |
|:---|:---|
| empty | `○ No Assembly loaded — generate or load one first` |
| loading | `⟳ Opening interactive 3D…` |
| result (browser) | Button `Open in browser` enabled; fidelity chip **Interactive 3D** |
| error | `✗ Preview failed — check the log` |

### 4.3 Material preview (`material_library_widget.py`, `material_preview_modes.py`)

| State | Copy |
|:---|:---|
| empty | `○ Select a material` |
| loading | `⟳ Generating preview…` |
| no color map | `◐ No color map yet — click Generate selected` |
| result | Swatch + **Quick preview** chip |

Preview modes strip: label `Preview` (not program ID). Modes: `Sphere` · `Wall` · `Floor` — sentence case.

### 4.4 Atlas preview (`atlas_preview_panel.py`)

| State | Copy |
|:---|:---|
| empty | `○ No packed tile sheet yet — run Pack atlas` |
| loading | `⟳ Loading atlas…` |
| result | Grid + `Atlas: {n} tiles · grid {c}×{r}` + **Ship render** chip |
| register hint | `Next: register this atlas for the map` |

### 4.5 Footprint canvas (`footprint_canvas.py`)

Not a raster preview — **Layout view**. Selection feedback: `COLOR_SELECT_BG` + 2px accent border on selected cell. Legend below canvas (AS-3).

### 4.6 Catalog module thumb (`catalog.py`)

| State | Copy |
|:---|:---|
| empty list | `○ No modules match this filter` |
| no GLB | `◐ No 3D file — validate or pick another module` |
| result | Thumb + module id |

`Quick 3D preview` button — not `trimesh` in label.

---

## 5. Update-on-select behaviour

| Trigger | Previews that update | Pattern |
|:---|:---|:---|
| Select piece in footprint | Slot 2×2, combined, context | Cancel stale job; show `⟳` immediately |
| Select material in library | Material swatch, modes strip | Debounce 150ms |
| Select variant row | Bake status line | Sync only |
| Select atlas tile/cell | Atlas grid highlight | Sync only |

**No UI block** — all render work on worker thread via existing `_start_job` pattern.

---

## 6. Selection feedback (managed surfaces)

Lists/grids share selected-card recipe from design system §3.5:

- `COLOR_SELECT_BG` fill
- 2px `COLOR_ACCENT` left border
- Status atom on row if validation state known

Applies: Catalog module list, Materials profile cards, Atlas cell grid, Preset list.

---

## 7. Sizing tokens

| Token | px | Use |
|:---|:---:|:---|
| `PREVIEW_THUMB_SM` | 96 | 2×2 slot cells @ min inspector |
| `PREVIEW_THUMB_MD` | 128 | material swatch default |
| `PREVIEW_THUMB_LG` | 192 | catalog list thumb |
| `PREVIEW_MIN_H` | 120 | preview pane floor |

At MIN window, thumbs scale down to `PREVIEW_THUMB_SM` — never clip without scroll inside the preview frame itself.

---

## 8. Verification

**Headless (`test_aps_runtime_callbacks.py`):**
- Each surface returns labelled placeholder for missing asset input
- No callback returns `None` image without label
- Empty + error paths render non-black background color

**NEEDS-DISPLAY (operator):**
- Select piece → 2×2 updates without jank
- Assembly browser preview opens with toast
- Atlas grid readable at 1280×800

---

## 9. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-17 |

**@coder-mcp:** one helper, one contract — do not per-panel invent states.
