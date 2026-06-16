# GRAMMAR-ITER-001 — Iterate grammar panel wireframe `v1`

| Field | Value |
|:---|:---|
| **Program** | `GRAMMAR-ITER-001-UI` |
| **Owner** | `@designer` |
| **Spec** | [`grammar_iter_001_spec_v1.md`](../../docs/archive/2026-06-src-dev/plans/grammar_iter_001_spec_v1.md) |
| **Brief** | [`grammar_iter_wireframe_brief_v1.md`](grammar_iter_wireframe_brief_v1.md) |
| **Labels** | [`grammar_labels_v1.json`](../../assets/configs/buildings/grammars/grammar_labels_v1.json) (reviewed v1.1) |
| **Implement** | @coder-mcp Phase 1 read-only · Phase 2 Apply wired |
| **Date** | 2026-06-03 |

---

## Placement — Assembly tab

Insert **below** `Generate` LabelFrame, **above** footprint workspace paned split. Do not duplicate slot editor or material browser.

```text
Assembly tab (scroll order)
├── intro line + metadata flow
├── Material authority panel
├── Generate                    ← existing (T0 full regen)
├── Iterate grammar             ← NEW (T1–T7 partial)
├── file row (path / load / save)
└── workspace paned
    ├── footprint + slot editor
    └── inspector (grammar + slot previews)
```

**Rationale:** Generate = “new building from seed.” Iterate = “keep what I like, change one layer.” Separating frames prevents accidental full reroll.

---

## Panel wireframe — default (Massing mode)

```text
┌─ Iterate grammar ───────────────────────────────────────────────────────────┐
│ Mode: [Massing & footprint ▼]     Seed: [43    ▲▼]   [Apply iteration]    │
│ Status: Ready · last parent: industrial_west_8x9_s43_f75a                    │
│                                                                              │
│ Massing strategy                                                             │
│   ( ) Long Hall    (•) Double Hall    ( ) L-Shape Yard    ( ) Yard Complex   │
│                                                                              │
│ Footprint   Width [10 ▲▼]  ×  Depth [6 ▲▼]     Floors [2 ▲▼]               │
│ Footprint mode (read-only):  Rectangle                                         │
│                                                                              │
│ Pin while iterating                                                          │
│   [✓] Pin district style    [✓] Pin age band    [ ] Pin roof slot            │
│                                                                              │
│ Last change:  +12 added · −4 removed · ~6 changed · layers: massing, facade  │
│ Footprint legend:  ■ Added   ■ Removed   ■ Changed   ■ Unchanged              │
│                                                                              │
│ [ ] Auto-preview on iterate          [Preview assembly]                      │
│ [Save branch…]   [Revert to parent]   (disabled until lineage present)       │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Typography:** Segoe UI 9 for labels; Consolas 9 only for `assembly_id` / path tokens — not primary control labels.

---

## Mode-specific fields

### Mode dropdown (always visible)

| mode key | Dropdown label (from `grammar_labels_v1.json`) |
|:---|:---|
| `massing` | Massing & footprint |
| `roof` | Roof only |
| `facade` | Facade ring |
| `detail` | Props & detail |
| `age` | Age & weathering |
| `material_strategy` | Materials only |
| `placement` | Single placement |
| `full` | *(hidden — use Generate frame for T0)* |

**Rule:** Do not offer `full` in Iterate dropdown — avoids duplicate Generate.

---

### Roof mode

```text
│ Roof module slot:  [Industrial Roof ▼]                                        │
│   options: Default Roof · Industrial Roof · Flat Roof                          │
│ Pin: [✓] Pin massing footprint                                               │
```

### Facade mode

```text
│ Wall slot:     [Standard Wall Module ▼]                                      │
│ Door slot:     [Wide Industrial Door ▼]                                      │
│ Window slot:   [Industrial Window ▼]                                         │
│ Facade pack:   Industrial Facade Pack (read-only rule_id: facade_v1)         │
```

Highlight **facade ring** cells on footprint after Apply (yellow changed, green added).

### Detail mode

```text
│ Detail props:  [Roof & Yard Clutter ▼]                                       │
│ Density:       [====●=====] 0.35                                             │
```

### Age mode

```text
│ Age band:  ( ) New    (•) Weathered    ( ) Abandoned                         │
```

### Material strategy mode

```text
│ District profiles (industrial_west) — read-only summary table                │
│   wall_1u → steel_panel_01    roof_industrial → roof_metal_01  …             │
│ [Open Materials tab]  — primary assign path stays on Materials + Assembly    │
│ Override: per-slot material_profile still via selected cell editor           │
```

No duplicate material picker in Iterate panel — link only.

### Placement mode

```text
│ Editing selected cell: node_12 (2,4) — use slot editor below                  │
│ Iterate Apply disabled here — cell edits use Save on snapshot                │
```

Show one-line hint; **defer** controls to existing footprint slot frame.

---

## Visual feedback (required)

### 1. Footprint grid diff (after Apply)

| State | Fill | Text label (a11y) |
|:---|:---|:---|
| Added | `#c8e6c9` green | Added |
| Removed | `#ffcdd2` red hatched | Removed |
| Changed | `#fff9c4` yellow | Changed |
| Unchanged | existing role colors | Unchanged |

Legend row **always visible** after first Apply — not color-only.

### 2. Grammar inspector (extend existing panel)

```text
┌─ Grammar inspector ─────────────────────────────────────────┐
│ Lineage: parent industrial_west_8x9_s43_f75a · iter #2 · massing │
│ … archetype / district / massing / roof / facade …          │
│ rule table:                                                 │
│   massing   double_hall   Double Hall — two-bay depth…  PIN  │
│   facade    facade_v1     Industrial Facade Pack              │
└─────────────────────────────────────────────────────────────┘
```

- **PIN** badge on rows present in `grammar_overrides`
- Lineage line above meta grid (Segoe UI 9)

### 3. Slot preview

On Apply with selected cell: auto-refresh module / material / combined / context thumbs (toggle **Auto-preview on iterate**, default **on**).

---

## Actions

| Control | Behavior | Phase |
|:---|:---|:---:|
| **Apply iteration** | Calls `grammar-iterate` API; reload snapshot in memory | 2 |
| **Preview assembly** | Existing Bevy/browser path — unchanged | now |
| **Save branch** | Dialog: suffix `_iter{n}` · writes new JSON · sets `grammar_lineage` | 3 |
| **Revert to parent** | Load `parent_assembly_id` snapshot from disk | 3 |
| **Auto-preview** | Checkbox; triggers preview after successful Apply | 2 |

Phase 1: panel visible, Apply disabled, tooltip from `ui.phase2_disabled` in labels JSON.

---

## Accessibility checklist

| Item | Design |
|:---|:---|
| Mode + Apply without hover-only | All controls on-panel; tooltips optional supplement |
| Diff not color-only | Numeric + text: “+12 added · −4 removed · ~6 changed” |
| Pin checkboxes | Text labels beside each pin |
| Min width 960px | Mode row wraps to two lines; strategy radios wrap 2×2 |
| Keyboard | Tab order: Mode → Seed → strategy → W/D/Floors → pins → Apply → Preview |
| Phase 1 disabled state | Apply greyed + visible reason string (not tooltip-only) |

---

## Coder-mcp implementation notes

1. New `GrammarIteratePanel` in `assembly_panel.py` or sibling module; pack between Generate and file row.
2. Load dropdown labels from `grammar_labels_v1.json` (`iteration_modes`, massing keys, etc.).
3. Wire footprint canvas `set_diff_highlight(added, removed, changed)` — Phase 1 can mock from last two in-memory snapshots.
4. Inspector: read `grammar_lineage`, `grammar_overrides` when Phase 1 lands.
5. Do **not** merge Iterate into Generate checkbox row — keeps T0/T1 mental model clear.

---

## Top 3 UX risks

1. **Generate vs Apply confusion** — artists may still hit Generate after tuning sliders. Mitigation: separate frames + status line “Iterate keeps parent lineage; Generate starts fresh seed path.”
2. **Material strategy mode scope creep** — duplicate Materials tab. Mitigation: read-only summary + link; assign stays on Materials + cell editor.
3. **Diff legend overload on small footprints** — 4×2 grids look noisy. Mitigation: legend collapses to text-only summary when footprint &lt; 12 cells.

---

## Sign-off

**Verdict:** PASS WITH NOTES — wireframe + labels v1.1 approved; Phase 1 disabled state copy required on ship.

**Unblocks:** `GRAMMAR-ITER-001-APS1` (@coder-mcp Phase 1 read-only UI)
