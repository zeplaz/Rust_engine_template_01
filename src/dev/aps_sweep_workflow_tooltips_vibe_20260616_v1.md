# APS sweep — Workflow · Tooltips · Vibe `v1`

| Field | Value |
|:---|:---|
| **Sweep ID** | `APS-SWEEP-WTV-001` |
| **Feeds** | [`plan_aps_uiux_overhaul_20260616_v1.md`](plan_aps_uiux_overhaul_20260616_v1.md) — adds a **workflow/pipeline-spine** phase + tooltip-content discipline + vibe punch-list the four dimension sweeps under-covered |
| **Date** | 2026-06-16 |
| **Author** | `@designer` |
| **Scope** | Three felt-experience axes grounded in LIVE code (`tools/mcp/art_pipeline_suite/`, Option D dual-notebook). **Design doc only — no code edits.** |
| **Non-goal** | Generation/content quality; replatform off Tk; re-deriving the text/layout/IA/style sweeps (cited, not repeated). |

The three settled sweeps treat the tool **tab-at-a-time** (what each surface says/looks like). They never trace the **line through** the tabs. That line is the user's #1 complaint — "it doesn't feel like a streamlined pipeline" — and it is a property of the *flow*, not of any one panel. This sweep covers the line.

---

## Axis 1 — Workflow: "it doesn't feel like a streamlined pipeline"

### 1.1 Where the flow lives in the code today

Three competing representations of "the pipeline" coexist, and **none of them is the thing the artist drives**:

1. **The Pipeline status bar** (`pipeline_status_bar.py`) — read-only pills (`○/◐/✓/✗`) per step. It *reports* state. It is **not clickable** (no `select(tab)` on click) and **not the same set** as the nav: Landscape declares 5 steps (`presets, grammar, states, atlas, stamp`) against a 4-tab notebook, so `Stamp` is a pill that points nowhere (already flagged P4-R1 in the plan).
2. **The Flow bar** (`app.py:_build_flow_bars`) — three verb buttons per lane (`Send to Assembly / Bake variants / Pack atlas`). These *do* drive the pipeline (they `notebook.select(...)` and call panel methods), but they are **always enabled**, regardless of readiness. Pressing one when you're not ready does nothing visible except a red hint string (`_flow_hint_var`, `flow_prerequisite_message`) that you have to be looking at the far end of the flow row to notice.
3. **The Notebook tabs themselves** — the artist's actual hands-on path. Tab order *is* the implied pipeline, but the tabs carry no "you are here / do this next" affordance beyond per-panel `next_step_var` callouts that exist on **Assembly only** (`assembly_panel.py:185`, `:434`, `:651`) and nowhere else.

So the pipeline is **described in three places, none authoritative, and the one band that looks like a stepper (the pills) is inert.** That is the root of the "non-linear / lost" feeling.

### 1.2 Current flow — Buildings lane, traced end-to-end

What an artist actually does, and where it breaks:

```
Catalog            pick a module → "catalog select: <id>"  (no nudge to a next tab)
   ?               three doors: Flow "Send to Assembly", or click Assembly tab, or click Materials tab
Assembly           Generate snapshot → next_step_var appears (only place in the app that says "Next:")
   ?               next_step says "→ Materials tab → Apply profile" but the TAB ORDER is Materials AFTER Assembly,
                   so "go back to a tab you skipped" — and Bake-variants flow SKIPS Materials entirely
Materials          browse, "Use in assembly" jumps you BACK to Assembly  (ping-pong: Asm→Mat→Asm)
Variants           on_new_from_assembly — but "Bake variants" flow auto-creates this for you, so the
                   tab is half-decoration: the flow verb does the tab's job behind your back
Atlas              Bake variants already dumped you here with a tile_batch "prepared"; now Pack atlas.
                   Two "Atlas"-named tabs exist across lanes (P4-R3). Stamp/register is a 4th concept here.
```

Felt problems, ranked:

| # | Problem | Evidence in code |
|:--|:--|:--|
| W1 | **No single "you are here → next" spine.** The pills look like a stepper but don't act like one; the only real "Next:" copy is Assembly-local. | `pipeline_status_bar.py` (no click), `assembly_panel.py:651` (only `next_step_var`) |
| W2 | **Flow verbs do tabs' jobs behind the user's back.** `on_bake_variants` silently runs `variants.on_new_from_assembly()` AND `atlas.on_batch_from_variant_set()` then jumps to Atlas — three steps collapsed into one button with no trace of what it touched. | `app.py:762-778` |
| W3 | **Tab order fights the flow.** Pipeline order is `…assembly, materials…` but the snapshot's next-step sends you to Materials which sits *after* Assembly while Bake-variants skips it. (Plan P4 already re-orders to Catalog→Materials→Assembly; this sweep confirms *why* it matters for flow.) | `app.py:_build_buildings_tabs`, `assembly_panel.py:651` |
| W4 | **Readiness is invisible until you fail.** Every flow button is enabled always; the only feedback for "not ready" is a red string at the right end of the flow bar (`_flow_hint_var`). Nothing disables, dims, or points. | `app.py:380-394`, `_build_flow_bars` |
| W5 | **Decision points with no default.** After Catalog, there are 3 equally-weighted doors (Flow button, Assembly tab, Materials tab). After Generate, the "next" is buried in a callout most people won't read. | `app.py:_on_catalog_select` logs only |
| W6 | **Dead-end pill.** `Stamp` (Landscape) is in the stepper but not the notebook — a step you can't navigate to. | `domain_router.py:36-43` vs `LANDSCAPE_TAB_LABELS` |

### 1.3 Current flow — Landscape lane, traced

```
Presets    pick preset → Validate preset (PASS=Ship badge)   [good: validate is right here]
Grammar    refresh_from_state, Validate schema, then Flow "Generate grammar" → mark_saved (scaffold)
   ?       "Generate grammar" flow button ALSO lives up top; the tab has its own Validate — two entry points
States     Flow "Bake states" → mark_states_ready; sets a hard-coded expanded batch path behind the scenes
Atlas      Pack LG-5 atlas → sets landscape_stamp_registered; "Stamp" pill finally lights — but there's no Stamp tab
```

Landscape is **closer to linear** (4 real tabs, validate-in-place on Presets) but inherits W1/W4 (inert stepper, invisible readiness) and the `Stamp` dead-end (W6). The "Generate grammar" verb is a scaffold (`mark_saved` just flips a bool) — fine for now, but the artist can't tell a scaffold action from a real one.

### 1.4 The streamlined pipeline — recommendation

**Core move: make the Pipeline status bar the single, authoritative, clickable spine — and make the flow verbs its "advance" button.** Collapse three representations into one. The pills already compute readiness (`_refresh_buildings`/`_refresh_landscape`); we promote them from a *report* to the *controller*.

Five concrete changes:

**S1 — One clickable stepper = the pipeline.** Make each pill clickable: click selects that tab (`notebook.select`). Mark the **current** step (the active notebook tab) with a filled marker `▣` distinct from status. The artist always sees: where they are (`▣`), what's done (`✓`), what's next (the first `○` after the current). This is the "you are here" spine. (Resolves W1, W5.)

**S2 — Readiness gates the flow verbs.** The flow verbs already have `flow_prerequisite_message`; promote it from after-the-fact red text to **a-priori button state**. The "next" verb is the only **primary/enabled** button; verbs whose prerequisites aren't met render **disabled with a one-line reason inline under them** (not a far-away red string). Hover a disabled verb → tooltip = the exact unmet prerequisite. (Resolves W4.) Tk reality: `state=tk.DISABLED` + a muted reason `Label` directly beneath — no motion needed.

**S3 — Advance-on-completion.** When a step's gate flips to `✓` (P0 passes, preset validates, atlas packs), the spine **auto-advances the "next" marker** and the next flow verb becomes the lit primary. The artist is *led*: finish a step, the tool lights the next thing. Do **not** auto-switch tabs (that's a focus-steal, violates spatial trust) — light the next, let them click. (Resolves W1, W5.)

**S4 — Flow verbs stop doing hidden work; they narrate.** `on_bake_variants` collapsing three operations is fine *as a convenience* but must **leave a trace**: each sub-action logs a one-line "✓ created variant set / ✓ prepared tile batch / → Atlas" so the artist sees what the button did. Better: the verb that crosses a skippable tab (Materials) should *light that step as skippable* rather than silently bypass it. (Resolves W2.)

**S5 — Kill the dead-end.** Fold `Stamp` into Atlas terminal state (already P4-R1). The stepper's last pill becomes `Atlas → registered` — a state of the Atlas tab, not a phantom step. Now every pill maps to a reachable tab (the DoD "every pipeline step maps to a reachable tab"). (Resolves W6.)

### 1.5 Proposed spine — ASCII sketch

```
┌─ LANE ───────────────────────────────────────────────────────────────────────┐
│  ◉ Buildings    ○ Landscape          What ships: assembly_snapshot (the truth) │
├────────────────────────────────────────────────────────────────────────────── ┤
│  PIPELINE  (click a step to go there · ▣ = you are here)                        │
│                                                                                 │
│   ✓ Catalog ─→ ▣ Assembly ─→ ○ Materials ─→ ○ Variants ─→ ○ Atlas              │
│   done         YOU ARE HERE   next           pending       pending              │
│                                                                                 │
│   Next step:  Run the P0 ship check on this assembly.   [ Run ship check ▸ ]    │
│                                            (lit = ready · the only primary btn) │
│   ─ Bake variants  (locked: pass the ship check first)  ← disabled + reason     │
└──────────────────────────────────────────────────────────────────────────────┘
```

- **One row, one truth.** The pills row *is* the flow. The verb row beneath shows only the **one next action** as primary, the rest dimmed-with-reason. No second stepper, no scattered "Next:" callouts.
- `─→` connectors read left-to-right as a path. `▣` = current tab. `✓` done, `○` pending, `✗` failed (with the fix inline).
- The **"Next step:" line** is generated from the same readiness logic that gates the verbs — one source, three surfaces stay in sync.

### 1.6 Canonical artist journeys (write these into the tool as the "Next step" copy)

**Buildings** (assumes the P4 re-order Catalog → Materials → Assembly → Variants → Atlas):

```
1. Catalog   — Pick the building style / modules you want to ship.
2. Materials — Pick or generate the surfaces (color, normal, roughness) those pieces use.
3. Assembly  — Generate the building, assign a material to each cell, run the ship check.
4. Variants  — Declare the tile states (day/night, clean/damaged) this building should bake into.
5. Atlas     — Bake the tiles, pack them, and register the atlas. → Done: it ships.
```

**Landscape:**

```
1. Presets — Pick a landscape preset and validate it (Ship badge = good to go).
2. Grammar — Review/adjust the topology graph; save the grammar.
3. States  — Set the succession + burn + regrowth states this landscape goes through.
4. Atlas   — Bake the keyframes, pack, and register the stamp. → Done: it ships.
```

These are short enough to be the literal `Next step:` strings (one per step), and they're the artist's mental model, not the schema's.

### 1.7 Workflow recommendation → plan delta

Add a phase to the overhaul plan (suggest **Phase 3.5 — Pipeline spine**, between Layout and IA, since it touches `app.py` chrome + `pipeline_status_bar.py` and depends on the P4 re-order/Stamp-fold):

- Make pills clickable (select tab); add `▣ current` marker.
- Drive flow-verb `state` from `flow_prerequisite_message` (enabled = next, disabled = reason inline).
- One "Next step:" line bound to the readiness model; retire per-panel `next_step_var` callouts in favor of the spine (keep Assembly's material-assign callout, which is contextual, not flow-spine).
- Advance-on-completion (light next; never auto-switch tabs).
- Guard test: assert every `pipeline_steps_for(lane)` key has a matching navigable tab; assert exactly one flow verb is enabled given a fixture state.

---

## Axis 2 — Tooltips: content + behavior strategy

The lifecycle bug (floating/persisting) is fixed in `aps_tooltips.py` (single shared `_Tooltip`, 450ms hover, drop on click/wheel/unmap/tab-change). **The remaining problem is content.** The dictionary (~110 entries) is solid coverage but the *voice* is engineer-to-engineer: most entries **restate the label** or **name a schema/path** instead of telling the artist the *consequence*. That buries the why and the what-happens-next behind a hover — and several put **critical info** (e.g. "saved ≠ shipped") into hover-only, which violates the plan's "tooltip-not-sole-channel" DoD.

### 2.1 Tooltip content principles

1. **Say the consequence, not the label.** A tooltip on "Generate snapshot" must not say "writes assembly_snapshot JSON" — the artist sees the word "Generate". Say *what they get and what to do next*: "Builds the whole building from your style + footprint. After this, assign materials and run the ship check."
2. **Why / consequence / what-happens-next — in that priority.** If you only have room for one, keep the consequence.
3. **Critical truth lives on-screen, not in hover.** "Saved ≠ shipped" / "this isn't ship art" / "snapshot is the ship truth" are load-bearing facts. They belong in always-on labels or status atoms. A tooltip may *reinforce* them; it must not be their only home. (DoD: tooltip-not-sole-channel.)
4. **Artist's words, no jargon/IDs.** Never surface `assembly_snapshot`, `tile_batch_v1`, `keyframe_pack`, `P0`, `LOD0`, schema names, or paths in a tooltip. (Same no-jargon rule as visible strings — a tooltip *is* a visible string.)
5. **One tooltip per primary action; cap at ~16 words.** Long tooltips don't get read. If a control needs a paragraph, it needs on-screen text, not a tooltip.
6. **Progressive disclosure.** Tooltips explain *primary* controls and *non-obvious* state. Self-evident controls (Refresh, Open folder) get a short tip or none — don't tooltip every widget into noise.
7. **No tooltip for a label that already says everything.** A read-only status label whose text is self-explanatory doesn't also need a hover.

### 2.2 Eight before→after rewrites (from the real dictionary)

| Key | Before (live) | After |
|:--|:--|:--|
| `asm_generate` | "Run grammar pipeline — writes assembly_snapshot JSON." | "Builds the whole building from your style and footprint. Next: assign materials, then run the ship check." |
| `pipeline_assembly` | "Assembly — valid only after QC/P0 gate; saved (QC not run) = snapshot on disk only." | "Green = passed the ship check. Saved-only means it's on disk but not proven — run the check before you ship." |
| `flow_bake_variants` | "Expand variant_set → tile_batch and jump to Atlas (needs assembly + variants)." | "Turns your tile states into bake-ready tiles and takes you to Atlas. Needs a built assembly first." |
| `cat_sidecar_truth` | "Sidecar tags are hints only — assembly snapshot semantic_tags win at ship." | "Tags here are just hints. What actually ships is set on the Assembly tab — edit them there." |
| `atl_lod0` | "CI/smoke ortho batch — not ship art. Use only for engine smoke tests." | "Quick rough render for engine testing only — this is not the art you ship." |
| `asm_save` | "Save current snapshot — validate before ship." | "Saves your work to disk. It won't ship until it passes the ship check." |
| `mat_use_in_assembly` | "Jump to Assembly and highlight this profile for assign." | "Takes you to Assembly with this surface ready — pick a cell to put it on." |
| `pipeline_step` | "Valid = gate passed. Saved (QC not run) = data on disk only. Pending = not started." | "✓ passed the check · ◐ saved but not proven yet · ○ not started." |

Two more worth fixing in the same pass (over the 8):

| Key | Before | After |
|:--|:--|:--|
| `atl_keyframe_rename` | "Rename keyframe PNGs to pack-friendly names before tilemapgen." | "Renames the rendered frames so they pack cleanly. Run this before you pack." |
| `asm_grammar_dna` | "Advanced massing pressure (yard / module density). Leave default unless tuning grammar." | "Advanced: how dense or spread-out the building is. Leave it alone unless you're tuning." |

### 2.3 Coverage rule (write into the design system / a guard)

```
COVERAGE
  • Every PRIMARY action button → exactly one tooltip (consequence-first).
  • Every status PILL / badge   → one tooltip explaining the glyph meanings.
  • Self-evident utilities (Refresh, Open folder, Search) → short tip or none.
  • Read-only labels that fully explain themselves → no tooltip.

CONTENT
  • ≤ 16 words. No schema names, paths, gate IDs, or program IDs (same no-jargon
    set as visible strings — a tooltip IS a visible string).
  • Leads with the consequence / what-happens-next, in the artist's verbs.

CHANNEL
  • Load-bearing truths (saved ≠ shipped · not ship art · this is the ship truth)
    MUST appear on-screen. Tooltips may reinforce, never be the only home.
```

A guard test can assert: every entry in `TOOLTIPS` ≤ 16 words, no entry matches the jargon regex (`APS-|ARCH-|LG-\d|G[0-5]|P0|_v\d|\.json|snapshot|tile_batch|keyframe_pack|schema`), and every `flow_*`/`pipeline_*` key exists.

---

## Axis 3 — Overall vibe / product feel

The honest read of the current tool: **competent and dense, but it reads as a developer's instrument, not a finished product.** It's calm enough (muted palette, no motion clutter — good for Tk), but it's **intimidating on first open** (a wall of chrome bands + jargon before you've done anything) and **under-acknowledges** the artist (most actions land as a log line, not a felt response). The good news: within Tk's limits (no rounded corners/shadows/motion), the gap to "real product" is mostly *copy, empty states, and micro-feedback* — all cheap.

### 3.1 First-run + empty states (the weakest moment)

On first open with nothing selected, the artist sees: a lane bar, an authority strip quoting `assembly_snapshot`, a Pipeline row of `○ … pending` pills, a Flow row of always-enabled buttons, a Catalog list, and the summary "Select a module." There is **no welcome, no orientation, no sense of "here's what this tool does / start here."** The `MetadataFlowPanel` auto-expands on first visit (`_initial_expanded`) — but it expands a **schema diagram in engineer vocabulary** (`assembly_snapshot (AUTHORITY)`, `module_placements → GLB resolve`), which is the *opposite* of a friendly first impression. Empty panels say `(none)`, `—`, `Select a module` — functional, but flat.

**Moves:**
- **Empty Catalog / Presets** → a one-line, friendly orientation in the detail pane: *"Pick a building style to start — you'll build it, skin it, and bake it to ship."* (Mirror the canonical journey, §1.6.)
- **Replace the auto-expanding schema diagram** as the first-run greeting. First run should expand a *plain* "How this works" (the 5-step journey), not the metadata-flow internals. Keep the metadata diagram available, collapsed, for power users.
- Empty status lines get a **verb**, not a dash: `States: not set yet — set succession + burn states` beats `—`.

### 3.2 Micro-feedback (does an action feel acknowledged?)

Today most actions resolve into the **collapsed-by-default Status log** (`_pack_status_log`, `expanded=False`) and an 80-char summary var. That's a whisper. Generate/validate land inline (good), but flow verbs, lane switches, and "sent to X" land mostly as log lines the artist can't see. Jobs do disable+`⟳ …` their button (`_start_job`) — that's the *best* micro-feedback in the app; it should be the template.

**Moves:**
- **Every primary action gets an inline acknowledgement** at its origin, not only in the log: a transient status atom (`✓ Snapshot built` / `→ Sent to Assembly`) next to the button or in the spine's "Next step:" line. The `⟳ button` pattern from `_start_job` is the model — extend its spirit to non-job actions with a status atom that updates in place.
- **The spine itself is feedback.** When a step flips to `✓` and the next lights up (§1.3 S3), *that* is the acknowledgement — the artist sees the pipeline move. This is the single highest-leverage feedback in the tool because it's structural, not per-button.
- Tk has no toast/motion; a status atom that *changes text+glyph+color in place* reads as acknowledgement without animation.

### 3.3 Tone / copy personality

Current copy oscillates between **terse-engineer** ("P0 validator — missing GLBs, materials, schema") and **occasionally warm** ("Blank? Wait for spinner, then Retry" — good!). The warm register is the one to standardize: **plain, second-person, consequence-first, never apologetic, never cute.** The product voice rules are already in the plan's P0 — this sweep adds: *the tool should sound like a calm senior artist sitting next to you*, not a schema validator. No exclamation marks, no jargon, no "Error:" where "couldn't" works.

### 3.4 Density / calm

The plan's layout sweep already targets the ~5 always-on chrome bands eating 26–31% of height. From a *vibe* angle the fix is the same and the win is emotional: **the work area appearing on first paint** (not a wall of bands) is the difference between "tool" and "instrument I trust." The spine (§1.5) replaces *two* of those bands (the Pipeline pills + the Flow verbs) with **one** purposeful band — a density win that's also a clarity win. That's the highest-leverage calm move available.

### 3.5 The 5 highest-leverage vibe moves

| # | Move | Why it lifts "real product" | Tk-realistic? |
|:--|:--|:--|:--|
| V1 | **The pipeline spine** (§1.5): one clickable stepper + one lit "next" verb, replacing the inert pills + always-on flow buttons. | Turns three confusing bands into one purposeful one; the tool now visibly *leads*. Biggest single lift to both calm and clarity. | Yes — relabel/rebind existing widgets, `state=DISABLED`, glyph swaps. |
| V2 | **A real first-run greeting** (§3.1): plain 5-step "how this works" + friendly empty states, replacing the auto-expanded schema diagram. | The make-or-break first 10 seconds stop saying "you need to be an engineer." | Yes — change `_initial_expanded` target content + empty-state strings. |
| V3 | **Inline acknowledgement on every primary action** (§3.2): status atom at the origin, the `⟳ button` pattern generalized. | Actions feel *heard*; the silent-log whisper is the #1 "unfinished" tell. | Yes — reuse `set_inline_status` + the job-button disable pattern. |
| V4 | **One status atom, everywhere** (already plan P5, reinforced here): `✓ ✗ ◐ ○ ⟳` glyph+word+color, kill the `●` material vocabulary and PASS-in-blue. | Visual consistency is what the eye reads as "designed by one person." Inconsistent status atoms read as "assembled by committee." | Yes — single `status_atom()` helper. |
| V5 | **Voice pass on the always-on chrome** (P2 + §3.3): authority strip, pill tooltips, empty states in the calm-senior-artist voice; zero jargon/IDs visible. | Tone is 80% of "feels pro." The tool currently *sounds* like its own source code. | Yes — string edits only. |

---

## Summary of plan deltas this sweep adds

1. **New phase — Pipeline spine** (suggest P3.5, after Layout, depends on P4 Stamp-fold + re-order): clickable stepper, readiness-gated flow verbs, advance-on-completion, one "Next step:" line, guard test (every step → reachable tab; exactly one verb enabled per fixture). *This is the direct answer to the user's #1 complaint.*
2. **Tooltip-content discipline** (extends P2): consequence-first rewrites, ≤16-word cap, no-jargon = no-jargon-in-tooltips, critical-truth-not-hover-only, coverage rule + guard.
3. **Vibe punch-list** (threads P2/P3/P5): first-run greeting + friendly empty states; inline acknowledgement on every primary action; calm-senior-artist voice on chrome.

The honest ceiling is unchanged (~80% consistency, Tk can't do chrome). But the **streamlined-pipeline feeling is not a chrome problem — it's a flow problem**, and flow is fully in reach within Tk: it's relabeling and rebinding widgets the tool already has into one spine that leads.
