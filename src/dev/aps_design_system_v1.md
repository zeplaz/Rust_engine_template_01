# APS Design System `v1` — canonical lock (OVR-P0-DESIGN-LOCK)

| Field | Value |
|:---|:---|
| **Program** | PLAN-APS-UIUX-OVERHAUL-001 |
| **Gate ID** | **OVR-P0-DESIGN-LOCK** |
| **Date** | 2026-06-17 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** — gates P1–P6 implementation |
| **Inputs** | [`aps_sweep_text_20260616_v1.md`](aps_sweep_text_20260616_v1.md) · [`aps_sweep_layout_20260616_v1.md`](aps_sweep_layout_20260616_v1.md) · [`aps_sweep_tabdesign_20260616_v1.md`](aps_sweep_tabdesign_20260616_v1.md) · [`aps_sweep_style_20260616_v1.md`](aps_sweep_style_20260616_v1.md) · [`aps_sweep_workflow_tooltips_vibe_20260616_v1.md`](aps_sweep_workflow_tooltips_vibe_20260616_v1.md) |
| **Supersedes** | Partial token/IA docs — this file is **single authority** for overhaul phases |
| **No Rust** | Spec only — `@coder-mcp` implements P1+ |

```text
OVR-P0-DESIGN-LOCK Q✓
Unblocks: OVR-P1-TOKENS-001 · OVR-DES-P2-COPY-PACK-001 · OVR-DES-P3-LAYOUT-DELTA-001 · OVR-DES-P45-SPINE-SPEC-001 · …
```

---

## 0. North star (acceptance lens for every phase)

A new artist opens APS and feels **led, calm, and trusted** — not dropped into an engineer's control panel.

| # | Principle | Tk-realistic meaning |
|:---:|:---|:---|
| 1 | **The tool leads** | One **pipeline spine** (clickable stepper) answers "where am I / what's next" |
| 2 | **Real first run** | Plain "how this works" + empty states — not auto-expanded schema dumps |
| 3 | **Every action acknowledged** | Feedback at the action origin — not only in a collapsed log |
| 4 | **One status language** | `✓ ✗ ◐ ○ ⟳` + word-first everywhere |
| 5 | **Calm senior-artist voice** | Zero jargon / gate IDs in visible strings |

**Ceiling:** ~80% visual consistency; no rounded corners, shadows, or motion (Tk limit). "Slick" = disciplined restraint.

---

## 1. Terminology glossary (canonical — one word per concept)

Replace **every** variant in UI strings, tooltips, and inline messages.

| Artist concept | **Use this word** | Never in visible UI | Notes |
|:---|:---|:---|:---|
| Saved building definition | **Assembly** | `snapshot`, `assembly_snapshot` (except path tooltip) | Button: "Generate Assembly", "Save Assembly" |
| Catalog kit item | **Module** | — | Catalog tab only |
| Placed instance on grid | **Piece** | `slot`, `placement`, `node` | "Selected piece", "Save tags to this piece" |
| Footprint square | **Cell** | — | Grid only |
| Surface look | **Material** | `profile`, `material_profile`, `pilot` | "sample material" not `pilot` |
| Color texture map | **Color** | `albedo` | PBR names OK in maps row only: Color / Normal / Roughness |
| Auto-placement rules | **Building style** | `grammar`, `ARCH-DNA`, `β`, `massing`, `DNA` | Checkbox: "Generate from a building style" |
| Building state set | **Variant** | `variant_set_v1`, `variant_key` | Tooltip may name schema |
| Packed tile sheet | **Atlas** | `tile_map`, `atlas_meta` | "tile job" for batch file |
| Strict ship validation | **Ship check** | `P0`, `P0 gate`, bare `QC` | Schema-only: **Check schema** |
| Source of truth line | **What ships:** | `ship truth`, `authority`, `AUTHORITY` | Lead-in pattern |
| Landscape layout | **Layout graph** | `topology_graph`, bare `topology` | Summary: "Layout shapes" |
| Landscape file | **Landscape preset** | `landscape_grammar`, `land_dna` | Settings: "disturbance settings" |
| Veg time axis | **Growth stage** | `succession` (bare) | States tab |
| Post-fire window | **Regrowth** | `regrowth_macro` | States tab |

### Material — one concept, three roles (IA lock)

| Tab | Role | Rule |
|:---|:---|:---|
| **Materials** | Library / studio | Create, preview, edit materials |
| **Assembly** | Assignment | Bind material to each **piece** |
| **Variants** | Reference by id | **Profile dropdown** — no free-text `wall_material` |

---

## 2. Voice & tone (7 rules)

1. **Sentence case** for labels, buttons, hints, messages. Tab labels stay single Title-case words.
2. **Buttons = imperative + noun.** "Generate Assembly", "Run ship check", "Pack atlas". Two validators must read differently: "Check schema" vs "Run ship check".
3. **Status = glyph + word** (word readable without color). Never glyph-only.
4. **Errors state the fix.** Every `✗` line includes what to do next.
5. **No code paths, type names, env vars, globs, or tool names in body text.** (`trimesh`, `tilemapgen`, `Cursor`, `RUST_ENGINE_*` → tooltip or remove.)
6. **No agent/program/gate IDs.** Strip `(APS-…)`, `(ARCH-…)`, `(DMCP-…)`, `(LG-5)`, `(G0–G5)`, `(P0)`, `@coder` from all visible chrome.
7. **Explainer panels = cause → effect.** One plain sentence per hop. No arrow chains of symbols.

### Authority strip copy (locked)

| Lane | Text |
|:---|:---|
| Buildings | What ships: the **Assembly** you save here (its materials + tags). Catalog data and atlas tiles only feed into it. |
| Landscape | What ships: the **Landscape preset** you select here. Tiles are baked through the keyframe step only. |

### Ban-list (hard — guard `test_aps_no_jargon.py` in P2)

```
assembly_snapshot · land_dna · topology_graph · material_profile · semantic_tags
variant_set_v1 · tile_batch · keyframe_pack · ARCH-MAT · ARCH-DNA · BUILD-SET
APS-PREVIEW · APS-MAT · DMCP · LG-5 · G4 · G5 · P0 gate · ship truth · rust_engine_mcp
VegetationExtractFrame · placement. · @coder · RON (bare button) · -pk rename (bare)
```

Program IDs may remain in **code comments** and **witness JSON** only.

---

## 3. Design tokens (implement in `aps_theme.py` — P1)

### 3.1 Typography ramp

| Token | Value | Role |
|:---|:---|:---|
| `FONT_CAPTION` | Segoe UI 8 | Decorative only — allowlisted |
| `FONT_UI` / `FONT_HINT` / `FONT_SMALL` | Segoe UI 9 | Body, hints, **floor** |
| `FONT_UI_BOLD` | Segoe UI 9 bold | Emphasis, chrome labels |
| `FONT_SECTION` | Segoe UI **10** bold | Section headers (degenerate fix) |
| `FONT_MONO_SMALL` | Consolas 9 | Dense mono floor |
| `FONT_MONO` | Consolas 10 | Paths, IDs |
| `FONT_TITLE` | Segoe UI **13** bold | One H1 per tab |

**Rule:** zero `("Segoe UI", N)` / `("Consolas", N)` literals in panels. Font guard catches **≤8**.

### 3.2 Color roles

| Token | Hex | Role |
|:---|:---|:---|
| `COLOR_PASS` | `#0a6b0a` | pass / valid |
| `COLOR_FAIL` | `#a00000` | fail / blocked |
| `COLOR_WARN` | `#a66b00` | saved-not-checked / partial |
| `COLOR_MUTED` | `#555555` | pending / caption |
| `COLOR_ACCENT` | `#0a4a7a` | structural — **never** pass state |
| `COLOR_PASS_BG` | `#f0faf0` | pass pill / banner |
| `COLOR_WARN_BG` | `#fff8ee` | warn surface |
| `COLOR_FAIL_BG` | `#fff0f0` | fail surface |
| `COLOR_PANEL_BG` | `#f6f7f9` | panel |
| `COLOR_INPUT_BG` | `#ffffff` | lists, canvases |
| `COLOR_SELECT_BG` | `#e8eef5` | selected card |
| `COLOR_SELECT_ACTIVE` | `#cce0ff` | pressed |
| `COLOR_OUTLINE` | `#c8ccd4` | borders, grid lines |
| `COLOR_LANE_BUILDING` | = `COLOR_ACCENT` | lane tint |
| `COLOR_LANE_LANDSCAPE` | `#1f6b54` | lane tint — ≠ PASS green |

**Data palettes** (`footprint` W/D/C/R/Y, diff colors) live in `aps_palette.py` — not UI role tokens.

### 3.3 Spacing scale (4px grid)

| Token | px |
|:---|:---:|
| `GAP_XS` | 2 |
| `GAP_SM` | 4 |
| `GAP_MD` | 8 |
| `GAP_LG` | 12 |
| `GAP_XL` | 16 |
| `INSET_PANE` | 8 |
| `INSET_PANEL` | 8 |
| `PANE_MIN_LIST` | 220 |
| `PANE_MIN_DETAIL` | 280 |
| `PANE_MIN_CANVAS` | 320 |
| `ROW_HEIGHT` | 24 |
| `SASH_WIDTH` | 7 |

### 3.4 Status atom (canonical — one helper)

```text
{glyph} {word}[ — {detail}]
```

| State | Glyph | Word examples | fg | bg (pills) |
|:---|:---:|:---|:---|:---|
| pass | `✓` | valid · ready · PASS | `COLOR_PASS` | `COLOR_PASS_BG` |
| fail | `✗` | FAIL · blocked | `COLOR_FAIL` | `COLOR_FAIL_BG` |
| warn | `◐` | saved (not checked) · partial | `COLOR_WARN` | `COLOR_WARN_BG` |
| pending | `○` | pending · missing | `COLOR_MUTED` | `#ffffff` |
| working | `⟳` | … | `COLOR_ACCENT` | — |

**Retire** material `●` — map Ready→`✓`, Partial→`◐`, Missing→`○`.

Implement: `status_atom(state) -> (glyph, word, fg, bg)` in `aps_inline_feedback.py`.

### 3.5 Component recipes

| Pattern | Spec |
|:---|:---|
| **Section** | `ttk.LabelFrame` + `FONT_SECTION` accent label |
| **Primary action** | `ttk.Button`; running → disabled + `⟳ {label}…` |
| **Blocked action** | Prereq banner adjacent (`✗` + reason) — not log-only |
| **Pipeline pill** | `tk.Frame` RIDGE + status atom + tint bg |
| **Destructive** | Only `askyesno`; separated from benign neighbors |
| **Selected card** | `COLOR_SELECT_BG` + 2px accent border |

---

## 4. IA contract (finalized)

### 4.1 Lane switch (LIVE — Option D)

```text
LANE (persistent):  [ Buildings ]  [ Landscape ]
```

- Dual notebook — **never** relabel one 5-tab row.
- Lane chip word + tint — not color alone.
- `Ctrl+1` / `Ctrl+2`; no cross-lane selection bleed.

### 4.2 Tab sets (locked order)

**Buildings (5):** `Catalog → Materials → Assembly → Variants → Atlas`

**Landscape (4):** `Presets → Grammar → States → Atlas`

| Rule | Detail |
|:---|:---|
| Materials before Assembly | Profiles exist before assignment |
| Stamp | **Folded into Atlas** terminal register state — **no 5th pill, no Stamp tab** |
| Pipeline keys | **=== tab keys**, same order, per lane |
| Catalog | **Modules only** — remove dead landscape branch |
| Variants material | Profile-id dropdown from Materials catalog |

### 4.3 Tab ownership map

| Tab | Owns | Authority |
|:---|:---|:---|
| **Catalog** | Module browse, module info edit, GLB check | module library (input) |
| **Materials** | Material library / studio | profiles (input) |
| **Assembly** | Footprint, pieces, tags, **material assign**, ship check | **Assembly** (ships) |
| **Variants** | Variant layers, bake prep | variant set (derived) |
| **Atlas** | Pack, QC, **register** | atlas (output) |
| **Presets** | Preset browse, validate | preset (input) |
| **Grammar** | Layout graph editor | layout graph (ships) |
| **States** | Growth + fire matrix | catalog rows (derived) |
| **Atlas (landscape)** | LG-5 pack + register + G-scope QC | atlas (output) |

### 4.4 Pipeline spine (P4.5 — direction locked here)

1. **Pills are clickable** — select tab; do not auto-switch on completion.
2. **Current tab** marked `▣` on spine (distinct from status glyph).
3. **Flow verbs** = "advance" for enabled next step only; disabled shows reason inline.
4. **Flow verbs narrate** sub-steps in status log when collapsing work.

Buildings pipeline keys: `catalog · materials · assembly · variants · atlas`  
Landscape pipeline keys: `presets · grammar · states · atlas` (register = atlas terminal `✓ Stamp registered`)

---

## 5. Layout contract (P3 implements)

| Window | Size | Rule |
|:---|:---|:---|
| Default | 1280×800 | Footprint grid visible without scroll |
| Min | 960×600 | No forced horizontal scroll; 2-pane collapse below 1100px |
| Chrome budget | ≤2 rows (~64px) above notebook | Merge lane+flow; fold authority into pipeline row |

| Priority | Rule |
|:---|:---|
| P0 | Metadata explainer **collapsed** default |
| P0 | Assembly: setup strip **collapsed** after first Assembly; grid top-visible |
| P0 | Left gutter `GAP_MD` aligned on all chrome |
| P1 | Status log max height capped; notebook keeps vertical priority |

---

## 6. Phase citation map

| Phase | Implements from this doc |
|:---|:---|
| **P1** | §3 tokens + guards |
| **P2** | §1 glossary + §2 voice + ban-list |
| **P3** | §5 layout |
| **P4** | §4 IA (tab order, Catalog strip, Variants dropdown, pipeline keys) |
| **P4.5** | §4.4 spine |
| **P5** | §3.4–3.5 status + components |
| **P5.6** | §0 #2 first-run + metadata rewrite |
| **P6** | Full DoD checklist below |

### Extended DoD (program exit)

- [ ] Token guards fail on literals
- [ ] No ban-list strings in visible UI
- [ ] Buildings tab order Catalog→Materials→Assembly→Variants→Atlas
- [ ] Landscape Stamp folded into Atlas — 4 pipeline pills
- [ ] Clickable spine; flow verbs gated
- [ ] MIN 960×600 usable; footprint visible @ 1280×800
- [ ] One status atom everywhere
- [ ] `pytest -k aps` green + operator eyeball

---

## 7. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-17 |

**@coder-mcp:** cite this file in every OVR-P* commit message. **@designer-mcp:** G0 audit (`DMCP-OVR-G0-AUDIT-001`) against §2 ban-list.
