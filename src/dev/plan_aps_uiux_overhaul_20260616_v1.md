# PLAN-APS-UIUX-OVERHAUL-001 — Art Pipeline Suite UI/UX full overhaul `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-APS-UIUX-OVERHAUL-001** |
| **Owner (sequencing)** | `@orchestrator-mcp` · implementation `@coder-mcp` · design authority `@designer` |
| **Date** | 2026-06-16 |
| **Status** | **DRAIN — P3 next** |
| **Drain playbook** | [`plan_aps_uiux_overhaul_drain_finish_v1.md`](plan_aps_uiux_overhaul_drain_finish_v1.md) ★ |
| **Machine queue** | [`tools/orchestrator/queues/aps_uiux_overhaul_queue.json`](../tools/orchestrator/queues/aps_uiux_overhaul_queue.json) (24 rows) |
| **Agent board** | [`aps_uiux_overhaul_agent_todos_v1.md`](aps_uiux_overhaul_agent_todos_v1.md) |
| **Dispatch** | [`aps_uiux_overhaul_dispatch_orders_v1.md`](../tools/orchestrator/queues/aps_uiux_overhaul_dispatch_orders_v1.md) |
| **Scope** | Text · Layout · Tab design / IA · Visual style — a full overhaul of `tools/mcp/art_pipeline_suite/` |
| **Non-goal** | Grammar/generation *content* quality ("soft geometry", missing types) — separate lane. No replatform off Tkinter. |

## Source sweeps (detailed findings — the authority for each dimension)

- Text & copy: [`aps_sweep_text_20260616_v1.md`](aps_sweep_text_20260616_v1.md)
- Layout & density: [`aps_sweep_layout_20260616_v1.md`](aps_sweep_layout_20260616_v1.md)
- Tab design & IA: [`aps_sweep_tabdesign_20260616_v1.md`](aps_sweep_tabdesign_20260616_v1.md)
- Visual style & design system: [`aps_sweep_style_20260616_v1.md`](aps_sweep_style_20260616_v1.md)
- Workflow, tooltips & vibe: [`aps_sweep_workflow_tooltips_vibe_20260616_v1.md`](aps_sweep_workflow_tooltips_vibe_20260616_v1.md)
- Prior context (build on, don't repeat): [`design_aps_uiux_style_quality_20260616_v1.md`](design_aps_uiux_style_quality_20260616_v1.md), [`design_aps_ux_review_20260615_v1.md`](design_aps_ux_review_20260615_v1.md)

---

## Executive summary

The tool is **runtime-stable** (crashes fixed, `test_aps_imports.py` + `test_aps_runtime_callbacks.py` green) and the **Buildings⇄Landscape lane switch (Option D dual-notebook) is already implemented** (`app.py`, `domain_router.py`). The problem is no longer "it's broken" — it's that the surface reads as **functional but dated and inconsistent**:

- **Text** leaks engineering vocabulary into always-on chrome (literal code paths, gate IDs in panel titles, `ARCH-DNA`/`β`, schema names) and calls each core noun by three different names.
- **Layout** stacks ~5 always-on chrome bands that eat 26–31% of height at the min window, buries the footprint grid (the ship-truth object) below the fold, and improvises padding via ~399 hardcoded literals.
- **Tab/IA** — the live Option D is structurally sound but carries debt: a pipeline step (`Stamp`) with no tab, a dead landscape code path inside Buildings Catalog, a tab order that fights the workflow, and one material concept expressed in three vocabularies across three tabs.
- **Style** — `aps_theme.py` has good tokens that panels *bypass*; the typography ramp is degenerate (4 tokens, 1 size), ~40 hardcoded hexes encode 5 roles, three competing status vocabularies coexist, and a real sub-floor bug (`("Segoe UI", 7)`) slips past the font guard.

**Goal:** cross from "functional but patchwork" to "coherent, clear, intentional" by **enforcing one design system** (not replatforming). The honest ceiling is ~80% consistency / ~20% chrome — Tk can't do rounded corners/shadows/motion, and that's fine.

---

## Design north star — the feel we're building toward

The overhaul has one felt goal: a new artist should open the tool and feel **led, calm, and trusted**, and see a surface that reads **clean, slick, and professional — clear over clever** — not dropped into an engineer's control panel. Today it "sounds like its own source code." Within Tk's ceiling, "slick" means **disciplined restraint**: uncluttered surfaces, one consistent type/colour/spacing system, predictable controls, and responsive feedback on every action — never motion or flash. The north star, and the five highest-leverage moves toward it (all Tk-realistic), are the acceptance *lens* for every phase below — not a separate task:

1. **The tool leads.** One purposeful **pipeline spine** (Phase 4.5) replaces the inert progress pills + always-on flow buttons, so "where am I / what do I do next" is answered at a glance.
2. **A real first run.** Replace the auto-expanding engineer schema diagram (`MetadataFlowPanel`) with a plain "how this works" + friendly empty states — the make-or-break first 10 seconds.
3. **Every action is acknowledged.** Primary actions land feedback at their origin, not as a whisper in a collapsed status log.
4. **One status language** everywhere (`✓ ✗ ◐ ○ ⟳`, word-first) — no `●` material dialect, no PASS-rendered-in-blue.
5. **Calm, senior-artist voice** — zero jargon or IDs in anything the artist sees.

---

## How Cursor runs this (the executable contract)

This plan is authored to be picked up cold by Cursor with a strong model (Opus 4.8 / Sonnet 4.6). Per session:

1. **Boot the role:** `node .claude/skills/agent-lang/driver.mjs boot <agent>` (PRE → BOOT → HO).
2. **Owners are Cursor agent modes** in `.cursor/agents/`:
   - `@designer` — authors specs/copy/token specs/IA contracts and signs off. **Does not** write Python.
   - `@coder-mcp` — all Tkinter implementation under `tools/mcp/`.
   - `@orchestrator-mcp` — sequences phases, guards file ownership, no code.
3. **Guardrails auto-attach** via `.cursor/rules/*.mdc` (validation-first, mcp-production-rules, agent-lang-boot) and the MCP tools are native via `.cursor/mcp.json` (server `rust-engine-art`).
4. **Interpreter:** run the app and `pytest -k aps` with **`python` (3.14, has Pillow)** — `py -3.13` lacks PIL.
5. **Verification discipline (non-negotiable, every `@coder-mcp` phase):**
   - Keep `test_aps_imports.py` + `test_aps_runtime_callbacks.py` green (callback-level, catches the crash class import tests miss).
   - Add/extend a **guard test** for the phase's dimension (see each phase).
   - Run `python -m pytest -k aps -q` (from `tools/mcp/python`) and report counts; never claim a fix without re-reading the file + a passing assertion.
   - Visual items (anything about pixels/feel) are **NEEDS-DISPLAY** — flag for an operator eyeball; do not claim a visual fix headlessly.
6. **Definition of Done (the UX regression gate, extended):** imports/launch · font floor (≥9px, guard catches ≤8) · non-blocking feedback (no routine modals) · non-color status (glyph+word+color) · validity≠presence · tooltip-not-sole-channel · spacing tokens (no literal padding) · keyboard path · MIN-window (960×600) usable, no forced h-scroll · lane clarity + no cross-lane state bleed · witness honesty · **no jargon/gate-IDs in visible strings** · **one status atom** · **one terminology set**.

---

## Phase 0 — Design-system lock (GATE before any implementation)

**ID:** `OVR-P0-DESIGN-LOCK` · **Owner:** `@designer` · **Depends:** the four sweeps · **Blocks:** P1–P5

Consolidate the four sweeps into the canonical specs every later phase implements against. Write `src/dev/aps_design_system_v1.md` containing:

1. **Terminology + voice guide** — the canonical word per concept (Assembly / Piece / Module / Cell / Material / Color-Normal-Roughness / Building style / Variant / Atlas / Ship check / Check schema / "What ships:" / Layout graph / Landscape preset) and the 7 voice rules. Hard rule: **no program/gate/schema IDs in any visible string.**
2. **Design-token spec** — full typography scale (de-degenerate: caption 8 → body/hint 9 → bold 9 → mono/section 10 → title 13), color roles + canonical token names (PASS/FAIL/WARN/MUTED/ACCENT + surface + select + status tints), the 4px spacing scale (`GAP_XS/SM/MD/LG/XL`, `INSET_PANE/PANEL`, anti-starve `PANE_MIN_*`), and the **status atom** `{glyph} {word} [— detail]` with the colorblind-safe glyph set `✓ ✗ ◐ ○ ⟳`.
3. **IA contract** — the finalized lane/tab structure (below) + a tab-by-tab "owns what" map.

**Acceptance:** `@designer` sign-off; the doc is the single reference cited by P1–P5. **Verification:** N/A (design doc) — gate is human sign-off.

### Finalized IA contract (settled here; P4 implements)
```
LANE (persistent top segment, LIVE):  [ Buildings ]  [ Landscape ]
BUILDINGS (5, RE-ORDER):  Catalog → Materials → Assembly → Variants → Atlas
LANDSCAPE (4):            Presets → Grammar → States → Atlas        (Stamp folds into Atlas terminal state)
Material authority:  Materials = library/studio · Assembly = assignment (stays) · Variants = profile-id dropdown
Scaling rule: a 3rd lane = one more notebook + one *_BY_LANE entry + one radiobutton. Never nest. ≤6 tabs/lane.
```

---

## Phase 1 — Token-enforcement foundation (everything builds on this)

**ID:** `OVR-P1-TOKENS` · **Owner:** `@coder-mcp` · **Depends:** P0 token spec

Make the design system *real and enforced* before the dimension phases migrate onto it.

| Step | Detail | File |
|:---|:---|:---|
| De-degenerate the typography ramp | Give `FONT_SECTION` a distinct size (10); keep the P0 scale; no two tokens identical | `aps_theme.py` |
| Add spacing scale | `GAP_XS/SM/MD/LG/XL`, `INSET_PANE/PANEL`, `PANE_MIN_LIST/DETAIL/CANVAS` | `aps_theme.py` |
| Add color-role + status-tint tokens | `COLOR_SELECT_BG/ACTIVE`, `COLOR_OUTLINE`, `COLOR_PASS_BG/WARN_BG/FAIL_BG` (lift from `pipeline_pills.py`) | `aps_theme.py` |
| **Fix the font-floor guard** | regex must catch sizes **≤8** (currently only 8) — kills the `("Segoe UI", 7)` at `material_library_widget.py:512` | `tests/test_aps_font_floor.py` + the 7px site |
| Add hex-literal + font-literal guards | fail on raw `("Segoe UI", N)` / `#hhhhhh` on primary content outside the token module | new `tests/test_aps_style_tokens.py` |
| Implement the dead density guard | the cited density-token test is a stub — make it actually fail on padding integers outside the scale | `tests/test_aps_ux_polish_density_tokens.py` |

**Acceptance:** ramp non-degenerate; the 7px site gone; guards fail on a deliberately-added literal. **Verification:** new guard tests + `pytest -k aps` green.

---

## Phase 2 — Text overhaul (the words)

**ID:** `OVR-P2-TEXT` · **Owner:** `@coder-mcp` (impl) · `@designer` (copy authority) · **Depends:** P0 terminology

The single biggest clarity problem is the language: the UI leaks engineering vocabulary into always-on chrome and names one concept three ways. A new artist meets `assembly_snapshot`, `land_dna`, `topology_graph`, `material_profile`, `(ARCH-MAT-001)`, `(APS-PREVIEW-001)` in the first three visible rows — none explained. The full 57-string findings table + worked rewrites are in the [text sweep](aps_sweep_text_20260616_v1.md); the executable essentials:

### 2a. One word per concept (canonical glossary — replace EVERY variant in UI strings)
| Artist sees | **Use this word** | Replace these (live in code) |
|:---|:---|:---|
| The thing you assemble & save | **Assembly** | snapshot, assembly_snapshot, "the building" |
| One placed part | **Piece** | slot, cell, placement, node |
| Reusable source kit item | **Module** | (keep — Catalog only) |
| Footprint grid square | **Cell** | (keep — grid only) |
| A surface look | **Material** | profile, material_profile, pilot |
| Texture maps | **Color / Normal / Roughness** | albedo, "maps" |
| Auto-generation rules | **Building style** | grammar, ARCH-DNA, β, massing, DNA |
| State variation | **Variant** | variant_set, variant_key |
| Packed tile sheet | **Atlas** | tile_map, atlas_meta, tile_batch |
| Strict ship-gating check | **Ship check** | P0, P0 gate, QC |
| Schema-only check | **Check schema** | "Validate" (the loose one) |
| Source of truth | **"What ships:"** | ship truth, authority, AUTHORITY |
| Landscape layout graph | **Layout graph** | topology_graph |
| Landscape preset | **Landscape preset** | landscape_grammar, land_dna |
| Growth-over-time state | **Growth stage** | succession |
| Post-fire state | **Regrowth** | regrowth_macro |

> The artist sees **Modules** in the Catalog; an **Assembly** is built from **Pieces** (each Piece uses a Module); clicking a **Cell** selects a Piece. Keep raw schema names (`assembly_snapshot`, `material_profile`) ONLY in file-path field tooltips, never in labels — and NEVER rename the JSON keys themselves.

### 2b. The ban-list (never in any visible string OR tooltip — comments/witness only)
Program/gate IDs `(APS-…) (ARCH-…) (BUILD-SET) (DMCP-…) (LG-5) (G0–G5) (P0) (v1/v2)` · raw code paths (`placement.material_profile → … render extract`, `VegetationExtractFrame::BuildProfiles`) · type/schema names (`assembly_snapshot`, `variant_set_v1`, `tile_batch_v1`, `node_id`) · env vars (`RUST_ENGINE_ART_DEBUG_GUI=1`) · file globs (`tile_map_*.png`) · tool/lib names (`trimesh`, `tilemapgen`, `Cursor`, `MCP`, `rust_engine_mcp`) · agent handles (`@coder`).

### 2c. Worst offenders to fix first (P0 — before → after)
- **`"Material authority (APS-MAT-AUTH-UI-001)"`** → **"Where materials come from"**
- **Assembly engine-path** (a literal code path on screen) → **"The material you assign here is saved on each piece. The game and the preview both read it from this Assembly — not Catalog tags or Blender. Assign here, save, and it shows up everywhere."**
- **`"Store ARCH-DNA + β in snapshot"`** (checkbox) → **"Save shape settings with this building"**
- **`"P0 gate"`** (button) → **"Run ship check"**; sibling **`"Validate"`** → **"Check schema"** (+ one line saying which is required before saving)
- **`"Metadata → engine (ARCH-MAT-001)"`** (on every tab) → **"Where this data goes"**
- **`"variant_set_v1 — declarative layers … Bake via MCP variant_bake / tile_batch_run"`** → **"Variant set — states of the same building (lighting, damage, fill). Bake them into tiles from here; no manual Blender."**
- **`"-pk rename"`** (Atlas checkbox) → **"Rename keyframe PNGs for packing"**
- **Landscape authority strip** (`land_dna + topology_graph … keyframe_pack`) → **"What ships: the Landscape preset you select here. Tiles are baked through the keyframe step only."**

### 2d. Voice rules
Sentence case (tabs stay Title-case single words) · buttons = imperative verb + canonical noun, two different buttons must read differently · status word-first then glyph, never glyph-only · every FAIL states the fix in artist verbs · never print code/types/env/globs/tool names in body text · no agent/program/gate IDs · one **"What ships:"** phrasing for all source-of-truth messages.

### 2e. Tooltip strategy (content + coverage — a tooltip IS a visible string, ban-list applies)
A tooltip must state the **consequence / why / what-next**, not restate the label; load-bearing truth (saved≠shipped, "not ship art") must ALSO be on-screen, never hover-only. Cap ~16 words. Coverage rule: every primary action + every status pill gets one; utilities short-or-none; self-explanatory labels none. Rewrites (before → after):
- `asm_generate` → **"Builds the whole building from your style and footprint. Next: assign materials, then run the ship check."**
- `asm_save` → **"Saves your work to disk. It won't ship until it passes the ship check."**
- `pipeline_assembly` → **"Green = passed the ship check. Saved-only means on disk but not proven — run the check before you ship."**
- `cat_sidecar_truth` → **"Tags here are just hints. What actually ships is set on the Assembly tab — edit them there."**
- `atl_lod0` → **"Quick rough render for engine testing only — this is not the art you ship."**
- `mat_use_in_assembly` → **"Takes you to Assembly with this surface ready — pick a cell to put it on."**

**Acceptance:** a guard test asserts no ban-list token appears in any visible string OR tooltip value; one canonical word per concept survives in UI strings (no variant); every primary action + status pill has a ≤16-word tooltip; `@designer` copy sign-off. **Verification:** new `tests/test_aps_no_jargon.py` (scans panel strings + the `aps_tooltips` dict) + `pytest -k aps`.

---

## Phase 3 — Layout & density overhaul

**ID:** `OVR-P3-LAYOUT` · **Owner:** `@coder-mcp` (impl) · `@designer` (layout deltas) · **Depends:** P1 tokens

| Step | Detail | File(s) |
|:---|:---|:---|
| Reclaim the work area | collapse/merge the 5 always-on chrome bands; add `GAP_XL` separation chrome↔work; make secondary bands collapsible | `app.py` |
| Un-bury the footprint grid | `MetadataFlowPanel` default-collapsed; advanced sections collapsed; reorder so the grid is above the fold at 1280×800 | `assembly_panel.py`, `metadata_flow_panel.py` |
| Fix FootprintCanvas overflow | canvas + legends must fit the pane (no fixed 280px+2 legends in a 215px pane) | `footprint_canvas.py` |
| Fix forced horizontal scroll | `scrollable.py` must sync interior width when horizontal scroll is enabled | `scrollable.py`, `app.py` |
| Migrate padding literals | move the ~399 hardcoded paddings onto the spacing scale | all `*_panel.py` |
| Pane consistency | Variants uses `aps_paned` (minsize floors + visible sash); Grammar 3→2-pane responsive at MIN | `variants_panel.py`, grammar panels |

**Acceptance:** at MIN 960×600 no child `reqwidth` exceeds its pane and no tab-level horizontal scrollbar is mapped; footprint grid visible without scroll at 1280×800. **Verification:** new `tests/test_aps_min_window_layout.py` (headless: build panels in a withdrawn root, assert no pane starvation / no h-scroll) + the density guard from P1.

---

## Phase 4 — Tab design & IA refinement (on the live Option D)

**ID:** `OVR-P4-IA` · **Owner:** `@coder-mcp` (impl) · `@designer` (IA) · **Depends:** P0 IA contract

| Step | Detail | File(s) |
|:---|:---|:---|
| R1 — fold `Stamp` into Landscape Atlas | the pipeline declares 5 steps but the notebook has 4 → dead-end nav. Make Atlas the terminal state. **Update `domain_router.verify_option_d_ia_contract()` from 5→4 keys and any test pinned to it.** | `domain_router.py`, `atlas_panel.py`, `pipeline_status_bar.py` |
| R2 — delete dead landscape path in Buildings Catalog | unreachable preset reader duplicating the `Presets` tab (drift risk) | `catalog.py` |
| Reorder Buildings | `Catalog → Materials → Assembly → Variants → Atlas` (profiles before assignment; matches pipeline order) | `app.py`, `domain_router.py` |
| R3 — disambiguate the two "Atlas" tabs | distinct labels + surface G0–G5 scope (not one `register_green`) | `atlas_panel.py`, `domain_router.py` |
| Unify material authority | Materials=library, Assembly=assign (stays), Variants material layer = **profile-id dropdown** not free-text | `materials_panel.py`, `assembly_panel.py`, `variants_panel.py` |
| Align nav to reality | pipeline-bar step set == navigable tab set per lane; lane-isolation guard (no cross-lane state bleed) | `pipeline_status_bar.py`, `app.py` |

**Acceptance:** every pipeline step maps to a reachable tab; no duplicate preset reader; Variants material is validated against the catalog. **Verification:** update `domain_router` contract test; add a lane-isolation assertion to `test_aps_runtime_callbacks.py`.

---

## Phase 4.5 — Pipeline streamlining (the spine)

**ID:** `OVR-P45-SPINE` · **Owner:** `@coder-mcp` (impl) · `@designer` (interaction spec) · **Depends:** P4 (shares `app.py`; needs the Stamp-fold + Buildings re-order settled first)

The workflow "feels confusing" because it is described in **three competing places, none authoritative**: the progress pills (`pipeline_status_bar.py`) look like a stepper but are inert and don't even match the nav (Landscape declares 5 steps against 4 tabs); the flow verbs (`app.py`) actually drive flow but are always-enabled and only fail into a red string at the far end of the bar; and the only "Next:" guidance lives on Assembly alone. Worst: `on_bake_variants` silently runs three operations behind one button.

**Fix — collapse the three into ONE clickable spine:** promote the pills from *report* to *controller*, and make the flow verbs its "advance" action.

```
PIPELINE   (click a step to go there · ▣ = you are here)
  ✓ Catalog ─→ ▣ Assembly ─→ ○ Materials ─→ ○ Variants ─→ ○ Atlas
  Next step:  Run the ship check on this assembly.   [ Run ship check ▸ ]
  ─ Bake variants  (locked: pass the ship check first)
```

| Move | Detail | File |
|:---|:---|:---|
| S1 — clickable pills + `▣` current marker | pills navigate to their tab; mark "you are here" | `pipeline_status_bar.py` |
| S2 — readiness gates verb STATE | disable flow verbs that aren't ready; show the reason inline under the disabled verb (reuse `flow_prerequisite_message`) | `app.py` |
| S3 — advance-on-completion | light the next step on completion; **never auto-switch tabs** (focus-steal) | `pipeline_status_bar.py`, `app.py` |
| S4 — narrate hidden work | a flow verb doing multiple ops must show each step, not bypass tabs silently (fix `on_bake_variants`) | `app.py` |
| S5 — every pill maps to a reachable tab | depends on P4's Stamp-fold (5 pills vs 4 tabs) | `domain_router.py`, `pipeline_status_bar.py` |

**Acceptance:** the spine is the single source of "where am I / what's next"; every step is clickable and maps to a real tab; no flow verb runs hidden multi-step work without narration; disabled verbs show their reason. **Verification:** extend `test_aps_runtime_callbacks.py` — assert pill→tab navigation, readiness-driven verb state, and no auto-tab-switch on completion.

---

## Phase 5 — Visual-style unification

**ID:** `OVR-P5-STYLE` · **Owner:** `@coder-mcp` (impl) · `@designer` (review) · **Depends:** P1 tokens + P0 status spec

The bar for this phase is **clean, slick, professional, clear**. In Tk's ceiling that means: uncluttered surfaces, one type/colour/spacing/status system applied everywhere (no patchwork), predictable widget behaviour, and visual restraint — not chrome. "Slick" is achieved by *consistency and the removal of clutter*, not effects.

| Step | Detail | File(s) |
|:---|:---|:---|
| One status atom | single `status_atom()` helper used everywhere; retire the material-card `●` vocabulary; **fix PASS-rendered-in-blue** (`atlas_panel.py:67`); add glyphs to text status (atlas register, variants bake, grammar-iterate) | new helper in `aps_inline_feedback.py`; all status sites |
| `tk` → `ttk` migration | migrate plain labels/frames off raw `tk.*` (so they inherit the `clam` theme); document the legit `tk` exceptions (canvases, pills, image-buttons) | all `*_panel.py` |
| Section component | one canonical "section" recipe (replace the bare-frame+bold-label vs `Labelframe` split) | `aps_collapsible.py` / new helper |

**Acceptance:** one status path (grayscale/colorblind-safe); no PASS-in-blue; raw-`tk` count materially reduced with documented exceptions. **Verification:** extend `tests/test_aps_style_tokens.py` to assert single status helper usage; `pytest -k aps`.

---

## Phase 5.5 — Preview & presentation (how things are shown)

**ID:** `OVR-P55-PREVIEW` · **Owner:** `@coder-mcp` (impl) · `@designer` (states + labelling spec) · **Depends:** P1 tokens + P0 status spec

Previewing is the most fragmented part of the tool, and it's where the artist's confidence is won or lost. Today each preview surface behaves differently: slot/module thumbnails render in-Tk (and fall to a gray placeholder when `trimesh` is absent), the assembly 3D preview opens in a **browser**, material previews use a cache, the atlas draws a grid, the footprint is a canvas — each with its own loading/empty/error behaviour, sizing, and labelling. The lived result is "3D preview is just black" and "select-slot previews seem broken." Make previewing **one coherent, predictable system**.

| Move | Detail | File(s) |
|:---|:---|:---|
| One preview-surface contract | every preview has the same four states — **loading** ("rendering…"), **empty** ("nothing selected"), **error/placeholder** (labelled, never black/blank/crash), **result** — at a consistent size + placement | `slot_preview_panel.py`, `assembly_preview_panel.py`, `material_preview_modes.py`, `atlas_preview_panel.py` |
| Fidelity labelling | every preview says what it is and its fidelity — **"Quick preview"** vs **"Ship render"** — so a rough/placeholder is never mistaken for final art (ties to the "not ship art" truth) | all preview surfaces |
| Make the in-Tk vs browser story intentional | in-Tk = fast quick-look thumbnails; **browser/three.js = full interactive 3D**. Say so in the UI; make the browser preview a clear **one-click, acknowledged** action, not a silent fallback | `assembly_preview_panel.py` |
| Smooth update-on-select | selecting a piece/material/tile updates its previews immediately, with the loading state, **no jank, no stale image, no UI block** (async job pattern) | `assembly_panel.py`, `material_library_widget.py`, preview surfaces |
| Graceful degradation as a rule | when render deps (`trimesh`) are absent, show a clear labelled placeholder + a one-line "real thumbnails need trimesh" hint — never black, never crash (codify what the runtime-debug pass started) | `aps_slot_preview.py`, `assembly_preview.py` |
| Consistent "managed" surfaces | uniform selection feedback in lists/grids (Catalog modules, Materials, Atlas cells), clear save/load state, and a predictable "what's selected → what you see" mapping | `catalog.py`, `material_library_widget.py`, `atlas_panel.py` |

**Acceptance:** every preview surface implements the four states at a consistent size + fidelity label; no black/blank/crashing preview anywhere; selecting an item updates its previews without blocking the UI; the browser 3D preview is a clear one-click action with feedback. **Verification:** extend `test_aps_runtime_callbacks.py` to assert each preview surface returns a labelled image (never `None`/black) for both a representative input and a missing-asset input; the smoothness/feel items are **NEEDS-DISPLAY**.

---

## Phase 5.6 — Onboarding & first-run (intuitive to start)

**ID:** `OVR-P56-ONBOARD` · **Owner:** `@designer` (content/flow) · `@coder-mcp` (impl) · **Depends:** P4.5 spine (the spine is the teacher)

A new artist should be productive in minutes without docs. Today first run drops them into a dense panel and **auto-expands an engineer schema diagram** (`MetadataFlowPanel`).

| Move | Detail | File(s) |
|:---|:---|:---|
| First-run greeting | replace the auto-expanded schema diagram with a plain **"How this works"** 5-step overview (the pipeline spine, in words) + a "start here" pointer | `metadata_flow_panel.py`, `app.py` |
| Friendly empty states | before the artist has done anything, every tab/list/preview says what it's for and the one action to take ("No assembly yet — Generate one to begin") | each `*_panel.py` |
| Progressive disclosure defaults | advanced sections collapsed; the happy path is what's visible first; depth on demand | `assembly_panel.py`, grammar panels, others |
| The spine teaches | the Phase 4.5 spine doubles as the guide — "you are here → do this next" walks a first-timer through the whole pipeline | `pipeline_status_bar.py` |
| Once, not nagging | onboarding is dismissible and remembers state | `state.py`, `app.py` |

**Acceptance:** first launch shows a plain how-it-works + a clear first action (not an engineer diagram); every primary surface has a helpful empty state; advanced sections collapsed by default; onboarding dismiss is remembered. **Verification:** headless assert that first-run renders the greeting (not the schema) and that empty states render; the onboarding *feel* is **NEEDS-DISPLAY**.

---

## Phase 6 — Integration, regression, sign-off

**ID:** `OVR-P6-CLOSE` · **Owner:** `@coder-mcp` + `@designer` · **Depends:** P2–P5

- Full `pytest -k aps` green (the pre-existing `test_grammar_iter::test_refresh_aps1_witness` is the content lane — out of scope, must be explicitly excluded/noted, not silently passed).
- All guard tests from P1–P5 green; `test_aps_imports` + `test_aps_runtime_callbacks` green.
- **Operator eyeball gate** (the visual ceiling): launch `python tools/mcp/art_pipeline_suite/run.py`, walk both lanes, confirm text/layout/tabs/style against the P0 spec. This is a human gate — the plan cannot self-certify pixels.
- `@designer` final sign-off; refresh the honest witness (`debug_runs/...`), gated on build health (never green over a failing/un-run tree).

---

## Dependency graph & file-ownership (conflict control)

```
P0 (designer, GATE) ─▶ P1 tokens ─▶ P2 text ─▶ P3 layout ─▶ P4 IA ─▶ P4.5 spine ─▶ P5 style ─▶ P5.5 preview ─▶ P5.6 onboarding ─▶ P6 close
                                     (run sequentially — they all share app.py / assembly_panel.py / atlas_panel.py / the preview surfaces)
```

P2 through P5.6 touch overlapping files (esp. `assembly_panel.py`, `atlas_panel.py`, `app.py`, the preview surfaces). Because Cursor is single-agent-centric, **run them sequentially** (P2→P3→P4→P4.5→P5→P5.5→P5.6) unless using isolated worktrees. P4.5 (spine) MUST follow P4 (depends on its Stamp-fold + re-order); P5.6 (onboarding) follows P4.5 (the spine is the teacher). `@orchestrator-mcp` owns the order and the per-phase file lock. After each phase: commit, re-run the guard suite, hand off.

| Phase | Primary files | Must not also be edited by |
|:---|:---|:---|
| P1 | `aps_theme.py`, the guard tests | — (foundation) |
| P2 | string sites + `aps_tooltips.py`, `metadata_flow_panel.py`, `aps_mat_auth_ui.py` | P3/P4.5/P5 (text only) |
| P3 | `app.py` (chrome), `scrollable.py`, `footprint_canvas.py`, panes | P4/P4.5 (until P3 commits) |
| P4 | `domain_router.py`, `app.py` (tabs), material vocab sites | P3/P4.5 (serialize on `app.py`) |
| P4.5 | `pipeline_status_bar.py`, `app.py` (flow verbs), `domain_router.py` | P3/P4/P5 (serialize on `app.py` — runs after P4) |
| P5 | status sites, `aps_inline_feedback.py`, `tk`→`ttk` | P2/P4.5 (serialize on string sites) |
| P5.5 | preview surfaces (`slot_preview_panel.py`, `assembly_preview_panel.py`, `material_preview_modes.py`, `atlas_preview_panel.py`, `aps_slot_preview.py`) | P5 (serialize on status/render sites) |
| P5.6 | `app.py` (first-run), `metadata_flow_panel.py`, `state.py`, panel empty-states | P3/P4.5/P5.5 (serialize on `app.py` + panels) |

---

## Risk register

| Risk | Mitigation |
|:---|:---|
| Phases collide on `app.py`/`assembly_panel.py` | sequential P2→P5 + per-phase commit + file-lock table above |
| `domain_router` contract test breaks on R1 (5→4) | P4 updates the contract + test in the same change |
| "Verified" that isn't runtime-real (the recurring failure) | every phase keeps `test_aps_runtime_callbacks` green + adds a dimension guard; visual items flagged NEEDS-DISPLAY, never self-certified |
| Token migration misses sites | guard tests fail the build on any remaining literal — enforcement, not vigilance |
| Scope creep into generation/content quality | explicitly out of scope; route to the content lane (separate plan) |
| Tk styling ceiling oversold | accept ~20% chrome limit; success = consistency, not chrome |

---

## What success looks like
A new artist opens the tool, immediately knows which lane they're in, reads labels that name one concept one way, sees the work area (not a wall of chrome) on first paint, gets status that survives grayscale, and never sees a gate ID or a code path. The tool reads as one coherent, intentional product — built on the Tk it already uses.
