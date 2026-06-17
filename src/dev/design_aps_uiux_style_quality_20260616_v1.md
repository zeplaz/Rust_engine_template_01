# APS UI/UX Style + Quality Plan — scaling to a vegetation/landscape lane `v1`

| Field | Value |
|:---|:---|
| **Program** | APS-UIUX-STYLE-QUALITY-001 |
| **Owner** | `@designer` (look/feel · IA · quality · a11y) — pairs with planner-mcp capability/schema plan |
| **Date** | 2026-06-16 |
| **Scope** | `tools/mcp/art_pipeline_suite/*` (Tkinter/ttk) — style system, IA for a new veg lane, a11y hardening, UX Definition-of-Done, roadmap |
| **Not in scope** | pipeline schemas, grammar/topology data model, MCP function signatures (planner-mcp owns) |
| **Grounding** | Prior reviews `design_aps_ux_review_20260615_v1.md` + `design_aps_artist_ship_review_20260615_v1.md`; veg reconciliation `veg_queue_reconciliation_20260616_v1.md`; live read of `aps_theme.py`, `app.py`, `pipeline_status_bar.py`, `aps_tooltips.py`, `aps_inline_feedback.py`, landscape presets, guard tests |

---

## 0. State-of-the-tool delta since the 2026-06-15 reviews (don't re-litigate)

The two prior reviews were dominated by a **file-wipe regression** (six UI + three backend modules zeroed) that made the app non-launching and scored it 2/10. **That is resolved on disk today** — verified by file sizes and guard tests:

| Prior P0/P1 finding | Status now | Evidence |
|:---|:---|:---|
| Six zero-byte UI files → app won't launch | **CLOSED** | all `.py` non-zero (`assembly_panel.py` 966 ln, `variants_panel.py` 429 ln, `grammar_inspector.py` 133 ln, etc.); `test_aps_imports.py` guards it |
| No regression guard for empty modules | **CLOSED** | `test_aps_imports.py` asserts symbols importable + `len(TOOLTIPS) >= 40` |
| Segoe/Consolas 8px on primary labels | **CLOSED** | `FONT_SMALL = ("Segoe UI", 9)` + `FONT_MONO_SMALL = ("Consolas", 9)` tokens; `test_aps_font_floor.py` greps & fails on `8` literals (footprint glyph allowlisted) |
| Pipeline "complete" = has-data not valid | **CLOSED** | `pipeline_status_bar._set_assembly` → `○ pending / ◐ saved (P0 not run) / ✗ P0 failed / ✓ valid`; `test_aps_pipeline_validity.py` |
| Modal popups blocking flow | **CLOSED** | `aps_inline_feedback.set_inline_status`; `test_aps_ux_nonblock_001.py` asserts `messagebox_routine_remaining == 0` |
| `material_library_widget` `NameError` in studio filter | **needs spot re-verify** | not re-read this pass; carry as P2-verify |

**Carried-forward open items** (still valid, fold into roadmap): Assembly density at MIN window, no keyboard path through core loop, color-as-primary on footprint heatmap / material swatch / atlas inline, flow-bar silent no-op when prerequisites missing, atlas tab over-long with CI controls inline.

So this plan is **not** a recovery plan — it is a **style-codification + scale plan**. The single new strategic question is IA for the veg lane (§2).

---

## 1. Style system — documented design tokens

Codifies what `aps_theme.py` already half-defines into a referenceable spec. Where a token is missing, it is flagged **NEW** (a @coder-mcp add) vs **EXISTS** (already in `aps_theme.py`).

### 1.1 Typography ramp

We have a 9px floor on content. Define the full ramp so future panels stop inventing literals.

| Token | Value | Role | Status |
|:---|:---|:---|:---|
| `FONT_SECTION` | Segoe UI 9 **bold** | LabelFrame / section headers (also accent-colored) | EXISTS |
| `FONT_UI_BOLD` | Segoe UI 9 bold | emphasis on a content label, tree headings | EXISTS |
| `FONT_UI` | Segoe UI 9 | default body / control text | EXISTS |
| `FONT_HINT` | Segoe UI 9 | inline hints, authority strip, pipeline caveat | EXISTS |
| `FONT_SMALL` | Segoe UI 9 | smallest allowed on **any content** label (floor) | EXISTS |
| `FONT_MONO` | Consolas 10 | paths, JSON, IDs that benefit from monospace | EXISTS |
| `FONT_MONO_SMALL` | Consolas 9 | dense mono (cell strips, log) — floor for mono | EXISTS |
| `FONT_TITLE` **NEW** | Segoe UI 12 bold | the one per-tab H1 ("Assembly — building snapshot authority") | NEW |
| `FONT_CAPTION` **NEW** | Segoe UI 8 — **decorative/non-content only** | watermark-tier tertiary text; **never** a label an artist must read; guarded out of content by `test_aps_font_floor` allowlist | NEW (optional) |

**Rule:** the ramp is 8(caption-only) → 9(body/small) → 9bold(section) → 10(mono) → 12(title). No literal font tuples in panels — import a token. `test_aps_font_floor.py` already enforces the floor; extend its allowlist comment to name `FONT_CAPTION` if introduced.

### 1.2 Color roles

| Token | Hex | Role | Pairing rule | Status |
|:---|:---|:---|:---|:---|
| `COLOR_PASS` | `#0a6b0a` | PASS / valid / ready | **always** with `✓` glyph + word | EXISTS |
| `COLOR_FAIL` | `#a00000` | FAIL / invalid / blocked | **always** with `✗` glyph + word | EXISTS |
| `COLOR_WARN` | `#a66b00` | WARN / saved-not-validated / partial | **always** with `◐`/`!` glyph + word | EXISTS |
| `COLOR_MUTED` | `#555555` | pending / disabled / "input only" captions | with `○` glyph or "pending" word | EXISTS |
| `COLOR_ACCENT` | `#0a4a7a` | section headers, authority strip, primary action emphasis, links | structural, not status | EXISTS |
| `COLOR_PANE_BG` | `#eceff3` | paned-window gutter | — | EXISTS |
| `COLOR_PANEL_BG` | `#f6f7f9` | panel surface | — | EXISTS |
| `COLOR_INPUT_BG` | `#ffffff` | entries, lists, selected tab | — | EXISTS |
| `COLOR_SASH*` | `#6b8299 / #94a3b4 / #4a5568` | visible sash divider | — | EXISTS |
| `COLOR_LANE_BUILDING` **NEW** | accent-blue family (reuse `COLOR_ACCENT`) | the Buildings lane identity tint | lane chip bg | NEW |
| `COLOR_LANE_LANDSCAPE` **NEW** | a distinct, colorblind-safe **green-neutral** (e.g. `#1f6b54` teal-green, not the PASS green) | the Landscape lane identity tint | lane chip bg | NEW |

**Contrast note:** `COLOR_LANE_LANDSCAPE` must NOT collide with `COLOR_PASS` (both greenish) — choose a teal/desaturated green so "I'm in the Landscape lane" never reads as "this passed." This is the one new color-role risk the veg lane introduces; resolve it at token level, not per-widget.

### 1.3 Spacing / density units

| Token | Value | Use | Status |
|:---|:---|:---|:---|
| `PAD_XS` **NEW** | 2 | intra-control gap | NEW (codify; literals exist) |
| `PAD_SM` **NEW** | 4 | control-to-control | NEW |
| `PAD_MD` **NEW** | 8 | section / panel padding (the de-facto standard already) | NEW |
| `PAD_LG` **NEW** | 12 | inter-group separation | NEW |
| `SASH_WIDTH` | 7 | paned divider | EXISTS |
| `ROW_HEIGHT` | 24 (`Treeview rowheight`) | list/tree row | EXISTS (inline) |

Today padding is hardcoded (`padding=8`, `padx=4`, `pady=(2,0)` …) across panels. Codifying as tokens lets a future "compact density mode" (a11y reduced-noise / small-screen) flip one place. `test_aps_ux_polish_density_tokens.py` exists — extend it to assert panels import these rather than literal ints.

### 1.4 Status vocabulary — glyph + word pairing (canonical)

Color-only was removed; lock the canonical glyph+word so no panel re-invents it.

| State | Glyph | Word | Color token | Example string |
|:---|:--:|:---|:---|:---|
| pass / valid | `✓` | `valid` / `ready` / `PASS` | `COLOR_PASS` | `✓ Assembly valid` · `PASS: 24 cells indexed` |
| fail / invalid | `✗` | `P0 failed` / `FAIL` | `COLOR_FAIL` | `✗ Assembly P0 failed` · `FAIL: 3 placements missing material` |
| warn / partial | `◐` | `saved (P0 not run)` / `partial` | `COLOR_WARN` | `◐ Assembly saved (P0 not run)` |
| pending / idle | `○` | `pending` | `COLOR_MUTED` | `○ Variants pending` |
| in-progress | `⟳` | `…` | `COLOR_ACCENT` | `⟳ Pack atlas…` (job-strip + disabled button) |

**Rule:** every status surface emits `{glyph} {label} {word}` and sets fg via `aps_inline_feedback.validation_foreground(ok)`. The glyph/word carry the meaning; color is reinforcement only.

### 1.5 Canonical interaction patterns (the four the brief asks for)

Stated as explicit-state recipes (form B), so any new lane reuses them verbatim.

```text
PRIMARY ACTION (e.g. "Generate snapshot", "Pack atlas")
  ⊙ ─render▶ (○ idle: ttk.Button default)
     ─hover▶ (◐ hover: ttk default highlight)
     ─press▶ (⟳ active: button DISABLED + text "⟳ {label}…", JobStrip shows running)   ← app._start_job already does this
     ─done▶  (○ idle restored + inline status line set via set_inline_status(ok=True/False))
     ═[prereq missing]▶ (⊘ blocked: DO NOT silently no-op — set inline banner near the action, ok=False)   ← FIX (flow bar still logs into collapsed log)

INLINE STATUS (replaces modal popups)
  label + StringVar, updated by set_inline_status(label, var, text, ok=...)
  ok=True→✓ green · ok=False→✗ red · ok=None→◐ warn · default→○ muted    ← never messagebox for routine feedback

VALIDATION RESULT (P0 gate, atlas QC)
  plain-language sentence, PASS:/FAIL: prefix (not color-only), itemized issues, "fix before ship" CTA
  persists on SuiteState (assembly_p0_passed) so pipeline bar reads it    ← already wired for Assembly; replicate for Landscape

DESTRUCTIVE CONFIRM (e.g. "Regenerate all pilots", reindex, overwrite registry)
  the ONLY allowed messagebox.askyesno — allowlisted in test_aps_ux_nonblock_001
  must name the artifact + consequence ("Overwrite material_profiles_v1.json? N profiles affected.")
  visually separated from benign neighbors (Separator + right-align, never adjacent to "Reload preview")
```

---

## 2. Scaling IA — adding a vegetation/landscape lane without bloat (THE central question)

### 2.1 The problem precisely

The 5 tabs (Catalog → Assembly → Materials → Variants → Atlas) encode a **building** workflow: a footprint grid with module placements that get material profiles, baked to variants, packed to an atlas. The authoring shape is *placement-centric* (grid cells + GLB modules + material assignment).

The veg/landscape lane is a **different authoring shape** (confirmed by the preset data and the reconciliation doc):
- Inputs are **landscape grammar presets** (`land_dna`, `pressure_field` lambdas, `topology_graph` of Network/Corridor/Ring/Patch/Cluster/Fringe), not footprint+modules.
- The "atlas" is the **LG-5 tile atlas** (topology tiles, burn/scar/recovery states), with its own G0–G5 ship gates distinct from building atlas QC.
- There is **no module-placement / material-profile-per-cell step** — succession state and disturbance (fire/harvest/construction) replace it.

Forcing this into the building tabs (e.g. "just add veg modes to Assembly") would overload the already-heaviest tab and confuse the authority story (building snapshot vs landscape grammar are different ship-truth objects). Adding 5 more veg tabs (10 total) is the bloat we must avoid.

### 2.2 Options evaluated

| Option | What it is | Pros | Cons | Verdict |
|:---|:---|:---|:---|:---|
| **A. Sixth tab "Landscape"** | one new tab alongside the 5 | minimal chrome change | a single tab can't hold preset-edit + topology-graph + atlas-QC + states; becomes a mega-tab worse than Assembly; muddies the building-centric tab row | ✗ reject |
| **B. Tab group / nested notebook** | tabs grouped under headers | groups read | nested notebooks are a Tk a11y/keyboard nightmare; doubles "which tab am I on" ambiguity | ✗ reject |
| **C. Context-driven workspace (auto-morph)** | tabs change meaning based on selected asset type | fewest surfaces | violates spatial-consistency principle #4 — the artist never trusts what a tab means; silent re-meaning is the cardinal UX sin | ✗ reject |
| **D. Top-level LANE switch (Buildings ⇄ Landscape) above the tab row** | a persistent mode segment that swaps the **whole tab set** + recolors chrome | each lane gets its own purpose-built tabs; building tabs stay clean; artist always knows the domain (persistent lane chip + tinted chrome); pipeline bar reflects the active lane's steps; scales to a 3rd lane later without touching existing lanes | one more top-level control; requires the flow-bar/pipeline-bar/authority-strip to be lane-aware | ✅ **RECOMMEND** |

### 2.3 Recommendation — **Option D: a top-level lane switch**

Add a persistent **Lane segmented control** at the very top of the window (above the Flow bar), with two segments: **Buildings** and **Landscape**. Selecting a lane swaps the entire `ttk.Notebook` page set and re-tints the chrome with that lane's identity color (`COLOR_LANE_BUILDING` / `COLOR_LANE_LANDSCAPE`). The flow bar, pipeline bar, and authority strip all become lane-scoped.

- **Buildings lane** keeps today's 5 tabs unchanged: `Catalog · Assembly · Materials · Variants · Atlas`.
- **Landscape lane** gets a purpose-built 4-tab set: `Presets · Grammar · States · Atlas`.
  - **Presets** — browse/clone the 10 landscape presets, edit `land_dna` + `pressure_field` lambdas (slider rows, plain-language labels), validate against `landscape_grammar_v0.schema.json`. (Analogue of Catalog.)
  - **Grammar** — the topology-graph workspace: tree of Network/Corridor/Ring/Patch/Cluster/Fringe with operator stacks + glyph-planning preview; generate/iterate; the ship-truth grammar object. (Analogue of Assembly — but graph, not footprint grid.)
  - **States** — succession + disturbance state matrix (burn / scar / recovery / harvest), the veg analogue of Variants' layer model.
  - **Atlas** — LG-5 tile atlas QC with the **G0–G5 scope-explicit** ship gate (schema-green ≠ bake-green ≠ art-ship-green; the reconciliation doc is explicit this distinction must be surfaced, not hidden behind one word "green").

This isolates lanes (multiview-safety principle #5: views isolated unless intentionally linked), keeps each tab focused (no mega-tab), and the artist *always* knows the domain via a persistent, color-coded lane chip — satisfying the brief's hard requirement.

### 2.4 Wireframe sketch

```text
┌───────────────────────────────────────────────────────────────────────────────────┐
│  Art Pipeline Suite                                                       [_][□][X] │
├───────────────────────────────────────────────────────────────────────────────────┤
│  Lane:  [▣ Buildings ]  [  Landscape  ]        ← segmented control, persistent      │  NEW (lane switch)
│         └ active = filled + lane-tint underline; inactive = outline                 │
├───────────────────────────────────────────────────────────────────────────────────┤
│  Flow:  [Generate grammar]  [Bake states]  [Pack LG-5 atlas]    (lane-scoped verbs) │  ← flow verbs change per lane
│         All actions call rust_engine_mcp CLI/MCP — agents use the same APIs.         │
├───────────────────────────────────────────────────────────────────────────────────┤
│  Ship truth: landscape_grammar preset (land_dna + topology_graph). Atlas is output. │  ← authority strip is lane-aware
├───────────────────────────────────────────────────────────────────────────────────┤
│  Pipeline:  ✓ Presets valid · ◐ Grammar saved (gate not run) · ○ States · ○ Atlas   │  ← pipeline steps are lane's steps
│             LG-5 atlas art-ship (G4/G5) is separate from schema/bake green.          │
├───────────────────────────────────────────────────────────────────────────────────┤
│ ╭─ Presets ─╮ Grammar │ States │ Atlas │            (Notebook — Landscape lane set)  │  ← tab SET swapped by lane
│ │                                                                                    │
│ │  [tab content for the active Landscape tab]                                        │
│ │                                                                                    │
│ ╰────────────────────────────────────────────────────────────────────────────────╯ │
├───────────────────────────────────────────────────────────────────────────────────┤
│  Jobs:  (idle)                                                  [Cancel]            │
│  ▸ Status log  (collapsed)                                                          │
└───────────────────────────────────────────────────────────────────────────────────┘

When Lane = Buildings, the same shell shows:  Catalog · Assembly · Materials · Variants · Atlas
with the building authority strip ("Ship truth: assembly_snapshot …") and blue lane tint.
```

**Why this beats a 6th tab:** a 6th tab would put Landscape *inside* the building tab row, so the artist sees building verbs (Send to Assembly, material profiles) while authoring veg, and the pipeline bar would have to mix two unrelated workflows. The lane switch keeps each pipeline-bar mental model honest and lets the LG-5 G0–G5 gate live in a Landscape-Atlas tab without diluting the building atlas QC.

### 2.5 Lane-switch state & isolation rules (engine hooks for @coder-mcp)

- `SuiteState.active_lane: Literal["buildings","landscape"]` (default `"buildings"`).
- Lane switch rebuilds (or shows/hides cached) notebook page set; **does not** carry building selection into landscape (no silent cross-lane state — principle #5). Persist last lane in `aps_ui_prefs.json`.
- `PipelineStatusBar.STEPS` becomes lane-keyed; `AUTHORITY_STRIP` becomes a per-lane constant; flow-bar verbs become per-lane.
- Lane identity color tints: the lane chip, the active-tab underline, and the authority-strip left border — a persistent, non-color-only "you are here" (chip also carries the lane word, never color alone).

---

## 3. Accessibility hardening (close the still-partial items)

| Item | Prior status | Action | Owner |
|:---|:---|:---|:---|
| Status not color/glyph-only | PARTIAL | footprint heatmap: draw role glyph at all cell sizes (or per-role hatch); material swatch: print profile id/initial beside swatch; atlas inline: `PASS:`/`FAIL:` text prefix. Lane identity: chip carries lane **word** + color. | @coder-mcp (impl) · @designer (glyph/hatch spec) |
| Min-font floor | CLOSED | maintain via `test_aps_font_floor`; the new Landscape tabs must import tokens, not literals — add them to the grep scope (already globs `*.py`). | @coder-mcp |
| Keyboard path through core loop | FAIL | add accelerators + Tab-order into canvases; footprint grid + topology-graph tree must be keyboard-navigable (arrow to select cell/node, Enter to act). Define a documented keymap. | @designer (keymap spec) · @coder-mcp (impl) |
| Usable at MIN 960×600 | FAIL | Assembly 3-pane → collapse to 2-pane below ~1100px with inspector in a sub-notebook; apply the SAME responsive rule to the Landscape Grammar tab (graph + inspector) so the new lane doesn't repeat the mistake. | @coder-mcp |
| Scroll affordances | PASS | keep `aps_scroll` wheel areas; new Landscape lists/trees must call `attach_wheel_area` (guarded by the nonblock test pattern). | @coder-mcp |
| Metadata-flow legibility | PASS | extend `metadata_flow_panel` with a Landscape variant: "land_dna + topology_graph → landscape grammar preset → runtime ecology / atlas". Plain language, no jargon-only headers. | @designer (copy) · @coder-mcp (panel) |
| Contrast — lane green vs PASS green | NEW RISK | pick `COLOR_LANE_LANDSCAPE` as teal/desaturated green, verify ≥4.5:1 on `COLOR_PANEL_BG` and visually distinct from `COLOR_PASS` in a grayscale sim. | @designer |
| Reduced-noise / density mode | OPTIONAL | the new spacing tokens (§1.3) enable a future compact mode; not a launch blocker. | deferred |

---

## 4. Interaction-quality bar — Definition-of-Done for APS UX

This is the regression gate every future APS change (building or landscape) must clear. Each row names whether a guard test already enforces it.

```text
▢ APS-UX-DoD ─⬡ checklist ⦃
   1  imports/launch?      app + every panel imports; no zero-byte module        [GUARD: test_aps_imports.py]
   2  font floor?          no ("Segoe UI",8)/("Consolas",8) on content labels     [GUARD: test_aps_font_floor.py]
   3  non-blocking?        no routine messagebox.showinfo/warning/error;
                           only allowlisted askyesno for destructive confirm      [GUARD: test_aps_ux_nonblock_001.py]
   4  non-color status?    every status = glyph + word + color (color reinforces, never sole)   [PARTIAL → extend guard]
   5  validity≠presence?   pipeline/QC "valid" only after the gate passes,
                           not merely "has data"                                  [GUARD: test_aps_pipeline_validity.py]
   6  tooltip not sole?    no action/meaning is comprehensible ONLY via hover;
                           label/inline copy stands alone                         [REVIEW: @designer copy pass]
   7  density tokens?      padding/spacing via tokens, not literals               [GUARD: test_aps_ux_polish_density_tokens.py — extend]
   8  keyboard path?       core loop completable without a mouse                  [NEW GUARD needed]
   9  MIN-window usable?   no horizontal scroll / starved pane at 960×600         [MANUAL until a layout assert exists]
   10 lane-clarity?        active lane obvious (chip+word+tint); no silent
                           cross-lane state bleed                                 [NEW GUARD — lane-state isolation test]
   11 witness honesty?     a green witness cannot be written over a tree that
                           fails import/collection (witness depends on smoke)     [GUARD: test_aps_witness_refresh + import gate]
 ⦄ ─⬡[cargo/pytest green on touched modules]▶ ─⬡[handoff lists engine hooks + remaining risk]▶ ★done
 fail any ⬡ ⟶ ¬done
```

The modal-popup regression we fixed is row 3; the file-wipe was rows 1 + 11. This checklist is what prevents both classes of regression from recurring, and rows 4/8/9/10 are the gaps to close.

---

## 5. Prioritized roadmap (P0/P1/P2 · owner · veg-prereq)

Owner key: **C** = @coder-mcp (implementable) · **D** = @designer (spec/copy). **Veg-prereq** = must land before the Landscape lane can ship.

| # | Pri | Item | Owner | Veg-prereq? |
|:--|:--|:---|:--:|:--:|
| 1 | **P0** | **Lane switch infra** — `active_lane` on SuiteState, segmented control, swap notebook page sets, lane-tinted chrome, per-lane authority strip + pipeline STEPS + flow verbs. | C | **YES** (enables the lane) |
| 2 | **P0** | **Design-token codification** — add `FONT_TITLE`, `PAD_*`, `COLOR_LANE_*`; pick colorblind-safe lane-landscape green (≠ PASS); document the ramp + roles. | D→C | **YES** (veg chrome needs lane color) |
| 3 | **P0** | **Lane-clarity + isolation guard** — test asserting active-lane is rendered (chip word) and no cross-lane state bleed; extend DoD §4 row 10. | C | **YES** |
| 4 | **P1** | **Landscape tab specs** — wireframe + copy + interaction model for Presets / Grammar / States / Atlas (graph-not-grid; G0–G5 scope-explicit atlas QC). | D | **YES** (spec gates impl) |
| 5 | **P1** | **Keyboard path** — keymap spec + impl: footprint grid AND topology-graph tree navigable by keyboard; new DoD guard row 8. | D→C | partial (building now; veg uses same recipe) |
| 6 | **P1** | **MIN-window responsive rule** — Assembly 3-pane→2-pane below ~1100px; apply same rule to Grammar tab pre-emptively. | C | partial |
| 7 | **P1** | **Non-color completion** — footprint glyph at all sizes, material swatch id text, atlas `PASS:/FAIL:` prefix; extend non-color guard (row 4). | D→C | partial |
| 8 | **P1** | **Flow-bar no-op fix** — prerequisite-missing surfaces an inline banner near the action, not a line in the collapsed log; apply to lane-scoped flow verbs. | C | no (but inherited by veg flow) |
| 9 | **P2** | **Re-verify `material_library_widget` studio-filter `NameError`** (carried from prior audit; not re-read this pass). | C | no |
| 10 | **P2** | **Atlas tab declutter** — collapse lod0/CI controls behind "Advanced"; mirror the pattern in the Landscape Atlas tab from day one. | C | partial |
| 11 | **P2** | **Metadata-flow Landscape variant** — plain-language land_dna+topology→grammar→runtime diagram. | D→C | nice-to-have |
| 12 | **P2** | **Density tokens migration** — replace literal padding ints with `PAD_*`; extend density-token guard. | C | no |

**Critical path to the veg lane:** items **1 → 2 → 3** (infra + tokens + clarity guard) then **4** (tab specs) unblock @coder-mcp to build the Landscape tabs against planner-mcp's schema/capability plan. Items 5–8 raise the shared quality bar both lanes inherit.

---

## 6. Required engine hooks (for @coder-mcp)

- `SuiteState.active_lane` + persisted `aps_ui_prefs.json["active_lane"]`.
- Lane-aware `PipelineStatusBar` (STEPS keyed by lane), per-lane `AUTHORITY_STRIP` constant, per-lane flow-bar verb set.
- `app._build_lane_switch()` segmented control above `_build_flow_bar`; `_on_lane_changed` rebuilds/shows the lane's notebook page set; lane-tint applied to chip + active-tab + authority-strip border.
- New token constants in `aps_theme.py` (§1.1–1.3) + `COLOR_LANE_*`.
- Landscape-lane panels (Presets/Grammar/States/Atlas) constructed via the existing `_add_scrollable_tab` recipe so scroll/wheel a11y is inherited.
- `landscape_grammar_v0.schema.json` validate hook for the Presets tab (reuse the existing validate-report path; planner-mcp owns the schema).

## 7. Diagnostics / guards required

- Extend `test_aps_font_floor.py` glob to cover new Landscape panels (already globs `*.py` — no change, just confirm new files land in `art_pipeline_suite/`).
- New guard: lane-state isolation (no cross-lane bleed) + active-lane rendered (DoD row 10).
- New guard: keyboard path through the core loop (DoD row 8).
- Extend `test_aps_pipeline_validity.py` to cover the Landscape pipeline's "valid≠present" (the LG-5 G0–G5 scope split must not flatten to one ✓).
- Keep `test_aps_imports.py` asserting every (including new) panel imports.

## 8. Risks / tradeoffs

- **Lane switch is the biggest single UX change** — it touches global chrome (flow/pipeline/authority). Mitigation: build it lane-aware but keep Buildings behavior byte-identical (default lane, same tabs, same strings); ship Buildings-only first, add the Landscape segment when its tabs exist.
- **Green-collision risk** (lane-landscape tint vs PASS green) — resolved at token level (§1.2); verify in grayscale sim before merge.
- **Scope-word flattening** (the reconciliation doc's core warning): if the Landscape Atlas tab shows a single "green" it will hide the schema/bake/art-ship distinction. The atlas QC copy MUST surface G0–G5 scope explicitly (this is a copy/spec obligation on @designer, item 4).
- **Don't pre-build veg tabs blind** — item 4 (specs) must precede impl; pairs with planner-mcp's schema plan so the Presets/Grammar tabs bind to the real `landscape_grammar_v0` shape, not a guess.

---

## Sign-off

| Role | Output |
|:---|:---|
| `@designer` | style-token spec · IA recommendation (Option D lane switch) + wireframe · a11y hardening list · UX DoD checklist · P0/P1/P2 roadmap with owners + veg-prereq tags |

```text
APS-UIUX-STYLE-QUALITY-001 complete
IA: top-level Buildings⇄Landscape lane switch (Option D) — NOT a 6th tab
Critical path to veg lane: lane infra + tokens + clarity guard → Landscape tab specs
DoD: 11-row regression gate (file-wipe=rows 1+11, modal=row 3 already guarded; gaps = rows 4/8/9/10)
```
