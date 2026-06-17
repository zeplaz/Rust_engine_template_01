# APS Visual Style & Design-System Sweep — target token spec `v1`

| Field | Value |
|:---|:---|
| **Program** | APS-STYLE-SWEEP-001 (dimension: VISUAL STYLE only) |
| **Owner** | `@designer` — look/feel, design tokens, status system, component vocabulary |
| **Date** | 2026-06-16 |
| **Scope** | `tools/mcp/art_pipeline_suite/*` visual surface: `aps_theme.py` tokens, color usage, typography ramp, status presentation, `tk`/`ttk` styling coherence |
| **Out of scope (other dimension owners)** | text/copy, layout/IA, tab-design. I touch IA only where the token system forces it. |
| **Grounding** | Live read of `aps_theme.py`, `pipeline_status_bar.py`, `pipeline_pills.py`, `aps_inline_feedback.py`, `app.py`, `atlas_panel.py`, `material_library_widget.py`, `footprint_canvas.py`, `catalog.py`, `assembly_panel.py`, `landscape_grammar_panel.py`, `metadata_flow_panel.py`, `aps_tooltips.py`, `test_aps_font_floor.py`. Extends (does not repeat) `design_aps_uiux_style_quality_20260616_v1.md` §1. |

This is the **token-and-component spec** the prior doc gestured at. It (a) defines the full intended scale, (b) names every coherence violation found in the live code, (c) gives an honest read of the Tk/ttk ceiling and the highest-leverage moves.

---

## A. The core problem in one sentence

`aps_theme.py` defines a respectable token set, **but the panels largely bypass it** — ~40 raw hex literals, ~35 inline font tuples, three competing status vocabularies, and near-parity (412 `tk.*` vs 362 `ttk.*`) raw-vs-themed widget usage. The tool *has* a design system; it just isn't *enforced*, so each panel re-improvises and the whole reads as a patchwork rather than one tool.

---

## B. Target design-token spec

### B.1 Typography ramp (full intended scale)

The "9px floor" is real but the current ramp is **degenerate** — `FONT_UI`, `FONT_UI_SM`, `FONT_HINT`, `FONT_SMALL` are all *identical* (`Segoe UI 9`). Four token names, one value. That gives panel authors no reason to pick the semantic token over a literal, which is exactly why literals proliferated. A real ramp needs distinct steps that carry meaning.

| Token | Value | Role | Status vs code |
|:---|:---|:---|:---|
| `FONT_TITLE` | Segoe UI 13 bold | per-tab H1 / window-level heading (one per surface) | **NEW** |
| `FONT_SECTION` | Segoe UI 10 bold | `LabelFrame` headers, section titles (accent-colored) | EXISTS @ 9 bold — **bump to 10** so sections read as a tier above body |
| `FONT_UI_BOLD` | Segoe UI 9 bold | inline emphasis, tree headings, "Pipeline:" / "Flow:" labels | EXISTS |
| `FONT_UI` | Segoe UI 9 | default body / control text | EXISTS |
| `FONT_HINT` | Segoe UI 9 | hints, authority strip, captions (always paired with `COLOR_MUTED`) | EXISTS — collapse `FONT_UI_SM` into this |
| `FONT_SMALL` | Segoe UI 9 | floor for any content label | EXISTS — alias of `FONT_UI`; keep as the documented floor name |
| `FONT_MONO` | Consolas 10 | paths, JSON, IDs | EXISTS |
| `FONT_MONO_SMALL` | Consolas 9 | dense mono (logs, cell strips) — mono floor | EXISTS |
| `FONT_CAPTION` | Segoe UI 8 — **non-content decorative ONLY** | watermark-tier; never an artist-readable label; explicitly allowlisted | NEW (optional) |

**Ramp:** `8 caption-only → 9 body/hint/small → 9 bold emphasis → 10 mono / 10 bold section → 13 title`.
**Rule:** zero font tuples in panels — import a token. The single concrete change that makes this stick: give `FONT_SECTION` a *different size* (10) from body (9) so authors who want a header reach for the token instead of typing `("Segoe UI", 9, "bold")` inline (which 7 panels do today).

### B.2 Color roles + canonical token names

Status colors are well-chosen; the failure is **panels redefining them by hand**. Below is the canonical set; everything in the "live drift" column is a hardcoded hex that should be the named token instead.

| Token | Hex | Role | Live drift found (replace these) |
|:---|:---|:---|:---|
| `COLOR_PASS` | `#0a6b0a` | pass / valid / ready | `#006400` (variants 146, grammar_iterate 190), `#0a6b1a`/`#1a6b1a` (atlas 240), `#0a6b0a` re-typed (material 495) |
| `COLOR_FAIL` | `#a00000` | fail / invalid / blocked | `#8b0000` (app 334/342, grammar_iterate 192), `#8b1a1a` (atlas 75/240) — **three different reds** |
| `COLOR_WARN` | `#a66b00` | warn / saved-not-validated / partial | `#a66b00` re-typed (material 495) |
| `COLOR_MUTED` | `#555555` | pending / disabled / caption fg | `#555`, `#666`, `#888`, `#444`, `#444444`, `#333`, `#333333` — **seven different greys** across 20+ sites |
| `COLOR_ACCENT` | `#0a4a7a` | section headers, authority, links, structural emphasis | `#0a4a7a` re-typed ~6 sites; misused as a **PASS** color (atlas 67 — see C.4) |
| `COLOR_PANE_BG` | `#eceff3` | paned gutter | — |
| `COLOR_PANEL_BG` | `#f6f7f9` | panel surface | — |
| `COLOR_INPUT_BG` | `#ffffff` | entries, lists, selected tab | `#f8f8f8`, `#f4f4f4`, `#f0f0f0`, `#ececec`, `#e8e8e8` — **five near-white surface greys** for canvases/thumbs |
| `COLOR_SELECT_BG` | `#e8eef5` | selected card/row background | **NEW** — `#e8eef5` invented inline (material 508/530, grammar 140) |
| `COLOR_SELECT_ACTIVE` | `#cce0ff` | active/pressed selection | **NEW** — `#cce0ff` inline (material 509) |
| `COLOR_OUTLINE` | `#c8ccd4` | thin separators, card/canvas borders | **NEW** — `#c8ccd4`, `#c8c8c8`, `#888888` inline |
| `COLOR_SASH / _LIGHT / _DARK` | `#6b8299 / #94a3b4 / #4a5568` | visible sash | — |
| `COLOR_LANE_BUILDING` | = `COLOR_ACCENT` | Buildings lane tint | EXISTS (alias) |
| `COLOR_LANE_LANDSCAPE` | `#1f6b54` | Landscape lane tint (teal-green, ≠ PASS) | EXISTS — but used raw as a fg in atlas 92 |

**Status-tint backgrounds** (currently floating in `pipeline_pills.py` only): promote to tokens so any inline-status surface can use them.

| Token | Hex | Role | Status |
|:---|:---|:---|:---|
| `COLOR_PASS_BG` | `#f0faf0` | pass pill / inline-pass background | NEW (lives inline in pills) |
| `COLOR_WARN_BG` | `#fff8ee` | warn / saved background | NEW (inline in pills) |
| `COLOR_FAIL_BG` | `#fff0f0` | fail background | NEW (inline in pills) |

**Domain/data palettes are legitimately separate** and should be *named, not removed*: `TOKEN_COLORS` (footprint W/D/C/R/Y) and `DIFF_COLORS` (added/removed/changed) in `footprint_canvas.py` are categorical data encodings, not UI chrome — keep them, but move them to a `aps_palette.py` (or a `DATA_*` block in the theme) so they read as "data palette," distinct from "UI role tokens." Today they sit anonymously at module top and invite copy-paste of their hexes elsewhere.

### B.3 Spacing / density units

| Token | Value | Use | Status |
|:---|:---|:---|:---|
| `PAD_XS` | 2 | intra-control gap, pill inner pad | NEW (literals everywhere) |
| `PAD_SM` | 4 | control-to-control | EXISTS |
| `PAD_MD` | 8 | section / panel padding (de-facto standard) | EXISTS |
| `PAD_LG` | 12 | inter-group separation | EXISTS |
| `ROW_HEIGHT` | 24 | tree/list row | NEW (inline in `init_aps_ttk`) |
| `THUMB_SIZE` | (per-panel) | card thumb edge | leave panel-local |

`PAD_XS` and `ROW_HEIGHT` are the only gaps; `PAD_SM/MD/LG` exist but are widely *ignored* in favor of literal `padx=4, pady=(2,0)` etc. The density-token guard `test_aps_ux_polish_density_tokens.py` is currently a **1-line stub** (verified — it does nothing). It must actually assert panels import these.

### B.4 The status system (canonical — this is the biggest coherence win)

There are **three competing status vocabularies in the live tree**. They must collapse to one.

| Surface | Live glyphs | Live words | Live color | Problem |
|:---|:---|:---|:---|:---|
| Pipeline pills (`pipeline_pills.py`) | `○ ◐ ✓ ✗` | pending / saved (QC not run) / valid / FAIL | token bg + fg | **the canonical one** |
| Material cards (`material_library_widget.py`) | `● ◐ ○` | Ready / Partial / Missing | `#0a6b0a/#a66b00/#888` inline | uses `●` (solid) for pass, not `✓`; re-typed hexes; different word set |
| Atlas register (`atlas_panel.py`) | **none** | PASS / FAIL | `#0a4a7a` (blue!) / `#8b1a1a` | **no glyph**, and PASS rendered in accent-**blue**, not green — reads as "info," not "passed" |
| Grammar iterate / variants | none / mixed | inline sentences | `#006400 / #8b0000 / #444` | ad-hoc, re-typed reds/greens |

**Canonical status atom — every status surface emits exactly this:**

```text
{glyph} {word}[ — {detail}]      fg = role color    [bg = role tint, for pill/banner surfaces]

  pass/valid/ready     ✓   COLOR_PASS   (+COLOR_PASS_BG on pills)
  fail/invalid/blocked ✗   COLOR_FAIL   (+COLOR_FAIL_BG)
  warn/partial/saved   ◐   COLOR_WARN   (+COLOR_WARN_BG)
  pending/idle/missing ○   COLOR_MUTED
  in-progress          ⟳   COLOR_ACCENT
```

Rule: **glyph + word carry meaning; color reinforces, never alone.** The four glyphs `✓ ✗ ◐ ○` (+ `⟳`) are colorblind-safe by *shape* (filled-check / cross / half / hollow). Retire the material-card `●` (it collides visually with `○`/`◐` and adds a 4th shape for no reason) — map "Ready→✓, Partial→◐, Missing→○". Atlas register MUST prefix `✓`/`✗` and use `COLOR_PASS`/`COLOR_FAIL`. This should live in **one helper** — extend `aps_inline_feedback.py` with a `status_atom(state) -> (glyph, fg, bg)` so no panel re-derives the mapping.

### B.5 Component vocabulary (the patterns the brief asks for)

Defined as concrete recipes so panels stop improvising. Each names the *one correct* way.

```text
SECTION / CARD
  ttk.Labelframe (TLabelframe style: groove border, COLOR_PANEL_BG, FONT_SECTION accent label)
  — NOT a bare tk.Frame with a hand-bolded tk.Label header (assembly/atlas do both today)

PRIMARY ACTION (Generate, Pack, Bake)
  ttk.Button, default style. Running → disabled + text "⟳ {label}…" + JobStrip active.
  Blocked-prereq → inline banner (✗ + COLOR_FAIL) adjacent to the button, never a silent log line.

SECONDARY ACTION (Browse…, Refresh, Open folder)
  ttk.Button, "Aps.Toolbar.TButton" compact style. Visually lighter than primary.

DESTRUCTIVE ACTION (Regenerate all, Overwrite registry)
  ttk.Button + the ONLY allowed askyesno; Separator + right-aligned, never adjacent to benign.

INLINE STATUS (replaces popups)
  ttk.Label + StringVar, set via aps_inline_feedback. Emits the B.4 status atom.

VALIDATION RESULT (P0 gate, atlas QC)
  status atom + plain-language sentence + itemized issues. Persists on SuiteState so the pipeline bar reads it.

PIPELINE PILL  → already canonical (pipeline_pills.apply_pill). Make every other status surface look like this.

SELECTABLE CARD / ROW
  COLOR_SELECT_BG when selected, COLOR_INPUT_BG idle, 2px COLOR_ACCENT border selected / 1px COLOR_OUTLINE idle.
  (material cards invent #e8eef5/#cce0ff/#f4f4f4 inline — tokenize.)

DATA CANVAS (footprint, atlas grid, topology graph)
  background = COLOR_INPUT_BG; grid lines = COLOR_OUTLINE; data fills from the DATA_* palette;
  categorical cells ALWAYS carry a glyph/letter (footprint already does — replicate for topology nodes).
```

---

## C. Consistency findings (where the code violates the system today)

**C.1 — Font literals everywhere (~35 sites).** `("Segoe UI", 9, ...)` and `("Consolas", 9/10)` typed inline in `assembly_panel`, `atlas_panel`, `atlas_preview_panel`, `catalog`, `material_library_widget`, `footprint_canvas`, `grammar_inspector`, `grammar_dna_panel`, `metadata_flow_panel`, `landscape_*`, `variants_panel`, `pipeline_status_bar` (line 22), `job_strip`, `aps_tooltips`. Root cause: the ramp is degenerate (B.1) so the token gives no benefit over the literal.

**C.2 — Sub-floor font slips past the guard (real bug).** `material_library_widget.py:512` uses `("Segoe UI", 7)` on a card button. The font-floor guard `test_aps_font_floor.py:30` regex `…,\s*8\b` matches **only size 8**, so 7 is invisible to it. The 9px-floor claim in the prior doc is therefore false on at least one content surface. Fix the regex to `,\s*[1-8]\b` (catch ≤8) and keep the `footprint_canvas glyph_size` allowlist.

**C.3 — Color literals bypass tokens (~40 sites, ~12 distinct hexes for 5 roles).** Worst offenders: **seven greys** (`#555 #666 #888 #444 #444444 #333 #333333`) all meaning `COLOR_MUTED`; **three reds** (`#a00000 #8b0000 #8b1a1a`) all meaning fail; **five near-white surfaces** (`#f8f8f8 #f4f4f4 #f0f0f0 #ececec #e8e8e8`) all meaning `COLOR_INPUT_BG`. No guard exists for hardcoded hex — there should be one (allowlisting `aps_theme.py` + `DATA_*` palette).

**C.4 — PASS rendered in accent-blue (semantic color error).** `atlas_panel.py:67` sets the *successful* register message to `color = "#0a4a7a"` (the accent/structural blue), and FAIL to `#8b1a1a`. A passing state that isn't green breaks the one rule the whole status system rests on. Plus the message has no `✓`/`✗` glyph (C.6).

**C.5 — Three status vocabularies (B.4).** Pills (`○◐✓✗`+word+tint), material cards (`●◐○`+Ready/Partial/Missing), atlas register (PASS/FAIL, no glyph, blue). An artist learns the pill language then sees a different one two tabs over.

**C.6 — Glyph dropped on text status surfaces.** Atlas register, variants bake status (`#006400` green sentence, no glyph), grammar-iterate diff all show color+word but **no shape glyph** — so on grayscale/colorblind they collapse to undifferentiated text. The footprint canvas does this *right* (always draws the W/D/C/R/Y letter, even at 7px); that discipline isn't applied to text status.

**C.7 — `tk` vs `ttk` patchwork (412 vs 362).** Raw `tk.Label`/`tk.Button`/`tk.Frame` do **not** inherit the `clam` theme set in `init_aps_ttk`, so they render with default-grey OS chrome against the themed `ttk` surfaces — the single biggest contributor to the "some panels look different" feeling. Some raw `tk` use is *necessary* (canvas, image buttons, colored pills, `Text`); much is not (plain labels/frames that could be `ttk`). Pills are `tk.Frame`/`tk.Label` **by necessity** (ttk can't set per-widget bg) — that's fine and should be the documented exception.

**C.8 — Section headers done two ways.** Some sections use `ttk.Labelframe` (themed, accent label); others use a bare frame + a hand-bolded `tk.Label` (`assembly_panel`, `atlas_panel`, `landscape_*`). No single "this is a section" component.

**C.9 — Density literals; guard is a stub.** `padx/pady` literals pervasive; `test_aps_ux_polish_density_tokens.py` is a **1-line empty file** despite the prior doc citing it as an existing guard. The token enforcement it implies does not exist.

**C.10 — No iconography, and that's correct.** There are no raster icons; status is glyph-based Unicode (`✓ ✗ ◐ ○ ⟳ ● ▸`). Given Tk constraints (no vector icon system, fragile image scaling, no icon font support) **this is the right call** — Unicode glyphs are theme-independent, crisp, and scale with font size. The fix is to *standardize the glyph set* (B.4), not add image icons. One optional lift: a small set of PNG glyphs at 16px for the lane chip / flow verbs only, where a recognizable mark adds scan-speed — but this is low priority and adds an asset-pipeline burden.

---

## D. The Tk/ttk ceiling — honest assessment

**What `ttk` + `clam` can realistically deliver (and the code already proves):**
- Custom named styles (`Aps.Toolbar.TButton`, `Aps.Lane.TRadiobutton`), consistent fonts, padding, treeview row height, colored `Labelframe` labels, visible paned sashes, notebook tab tinting on select. The `init_aps_ttk` foundation is genuinely good.
- A coherent flat-ish, light, professional look is achievable — `clam` is the most stylable built-in theme and accepts `background/foreground/bordercolor/lightcolor/darkcolor/relief` maps.

**The hard ceiling (don't fight these):**
- **No per-widget background on `ttk` labels/frames** without a one-off style per color — this is *why* pills/cards/canvases must stay raw `tk`. That's not a defect; it's the platform. Document it as the sanctioned exception.
- **No rounded corners, no shadows, no gradients, no opacity/alpha** in core Tk. Cards are square, borders are 1px relief. "Polished" here means *consistent and uncluttered*, not *Material/Fluent depth*.
- **No hover/transition animation** on `ttk` beyond state maps (`active`, `pressed`). Motion polish is off the table.
- **Font hinting/antialiasing** is OS-controlled; we can't smooth it.
- **High-DPI** scaling is coarse (Tk scaling factor) — pixel paddings won't all scale cleanly.

**Verdict:** the gap between "dated Tk" and "polished" is **~80% consistency, ~20% chrome.** The tool looks dated today mostly because it's *inconsistent* (C.1–C.9), not because Tk is incapable. Enforcing the token system closes most of the perceived-quality gap with zero new platform capability. The remaining 20% (depth/animation) is genuinely capped — accept it.

---

## E. The 3 highest-leverage moves

1. **Enforce the tokens with guards, and make the ramp non-degenerate.** Add a hardcoded-hex guard (allowlist `aps_theme.py`+`DATA_*`) and a font-literal guard; fix the font-floor regex to catch ≤8 (closes the real `7px` bug C.2); fill in the dead density-token guard. Give `FONT_SECTION` a distinct size (10) so authors *want* the token. This single move kills C.1/C.2/C.3/C.9 — the bulk of the patchwork — and prevents regression. **Highest leverage because it's mechanical, testable, and self-reinforcing.**

2. **Unify the status system into one atom.** One `status_atom(state) -> (glyph, fg, bg)` in `aps_inline_feedback`; retire the material-card `●` set; make atlas register and all text-status surfaces emit `✓/✗/◐/○` + word + role color (fixing the blue-PASS error C.4 and the missing-glyph colorblind gap C.6). Every status across every tab then reads identically. **Highest perceived-coherence gain per line changed.**

3. **Define and apply the component vocabulary (B.5), starting with `tk`→`ttk` migration of plain labels/frames + a single "section" component.** Replace bare-frame+bold-label sections with `ttk.Labelframe`; convert non-essential raw `tk.Label`/`tk.Frame` to `ttk` so they inherit the theme; document the necessary `tk` exceptions (pills, canvases, image buttons, `Text`). This is what makes the surfaces stop looking like different tools.

Do these three and the tool crosses from "functional but dated" to "coherent and intentional" without touching the platform ceiling.

---

## F. Engine hooks for @coder-mcp (token/guard adds only)

- `aps_theme.py`: add `FONT_TITLE`, `FONT_CAPTION`; bump `FONT_SECTION` to size 10; add `PAD_XS`, `ROW_HEIGHT`, `COLOR_SELECT_BG`, `COLOR_SELECT_ACTIVE`, `COLOR_OUTLINE`, `COLOR_PASS_BG`, `COLOR_WARN_BG`, `COLOR_FAIL_BG`; move `TOKEN_COLORS`/`DIFF_COLORS` to a named `DATA_*` block or `aps_palette.py`.
- `aps_inline_feedback.py`: add `status_atom(state) -> (glyph, fg, bg)`; route material cards + atlas register + variants/grammar-iterate status through it.
- Guards: fix `test_aps_font_floor.py` regex to `,\s*[1-8]\b`; new `test_aps_no_hardcoded_hex.py`; new `test_aps_no_font_literal.py`; implement the stubbed `test_aps_ux_polish_density_tokens.py`.
- Migration (mechanical): replace the ~40 hex literals and ~35 font tuples with tokens; convert non-essential `tk.Label/Frame` to `ttk`.

## G. Risks / tradeoffs

- **`tk`→`ttk` migration can regress per-widget bg** where a label legitimately needed a color (selected rows) — migrate only *plain* labels/frames; keep colored surfaces as documented `tk` exceptions.
- **Bumping `FONT_SECTION` to 10** slightly increases section header height — verify the 960×600 MIN window still fits (other-dimension layout owner should confirm).
- **Hex guard false-positives** on the data palette — must allowlist `aps_theme.py` and the `DATA_*`/palette module, else it blocks legitimate categorical colors.
- This sweep is **style-only**; the status-atom and section-component changes touch surfaces the layout/tab-design dimensions also edit — sequence after their structural changes land to avoid churn.

```text
APS-STYLE-SWEEP-001 complete (visual-style dimension)
Core: tokens exist but unenforced → patchwork. 3 vocabularies, ~40 hex literals, ~35 font literals, real 7px floor bug.
Ceiling: ~80% consistency / ~20% chrome — Tk can't do depth/motion; it CAN look coherent. Enforce, don't replatform.
Top 3: (1) token guards + non-degenerate ramp  (2) one status atom  (3) component vocabulary + tk→ttk migration
```
