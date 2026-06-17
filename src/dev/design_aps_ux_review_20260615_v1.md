# APS UX/UI heuristic re-audit — 2026-06-15 `v1`

| Field | Value |
|:---|:---|
| **Program** | APS-UX-AUDIT (re-run after ~12-day PAUSE) |
| **Prior audit** | `debug_runs/aps_ux_audit_001_live.json` (2026-06-03) |
| **Reviewer** | `@designer` |
| **Method** | Static read of `tools/mcp/art_pipeline_suite/*` + supporting `rust_engine_mcp` modules; bytecode recovery for emptied sources; live import probe |
| **Scope** | Catalog · Assembly · Materials · Variants · Atlas + flow bar / authority strip / pipeline bar / job strip / status log |
| **Tool runnable today?** | **NO — `ImportError` on launch** (see P0-A) |

---

## 0. Headline: the tool does not currently launch

A static + dynamic check shows **six APS Tk source files are 0 bytes** in both the working tree and `HEAD` (committed empty in `5a340510 "waves"`):

| Empty `.py` (0 lines) | Symbol it must export | Imported by |
|:---|:---|:---|
| `aps_tooltips.py` | `bind_aps_tooltip`, `bind_many`, `TOOLTIPS` | app + every panel |
| `job_controller.py` | `JobController`, `JobRecord`, `JobResult`, `JobState`, `JobWorker`, `DoneCallback` | app, atlas, material lib |
| `scrollable.py` | `ScrollableFrame` | app (`_add_scrollable_tab`) |
| `grammar_inspector.py` | `GrammarInspectorPanel` | assembly |
| `assembly_preview_panel.py` | `AssemblyPreviewPanel` | assembly |
| `variants_panel.py` | `VariantsPanel` | app |

Proof (import probe from `tools/mcp`):

```
IMPORT FAILED: ImportError cannot import name 'bind_aps_tooltip' from
'art_pipeline_suite.aps_tooltips' (.../aps_tooltips.py)
```

Their compiled `.pyc` bytecode survives in `__pycache__/` (16–38 KB each) and is committed — so the code was authored, compiled, then the `.py` sources were truncated to empty. **Python imports the empty `.py`, not the stale `.pyc`**, so the app is dead on arrival. Every heuristic finding below is moot until the source is restored; I scored on the assumption that restored source == the design captured in the bytecode + the prior-audit-era behavior. This is the single most important deliverable of this re-audit.

Recovery options for `@coder-mcp` (ranked):
1. Recover the six files from local bytecode (`uncompyle`/`decompyle3` against `cpython-313.pyc`, which are the larger/complete variants) — fastest path back to a runnable tree.
2. Restore from a pre-`waves` reflog/stash if one exists locally (HEAD is already empty, so plain `git checkout HEAD~n` will not help — needs reflog or an un-pushed branch).
3. Re-author from the recovered string/structure map in §7 + this doc if bytecode decompile fails on 3.14.

A CI guard must be added so an empty/uncompilable APS module fails fast (see fixes F1).

---

## 1. Prior-fix verification (the 5 from 2026-06-03)

Verified against current source where populated, and against recovered bytecode where the source is empty. **Caveat: all five live in modules that exist as code but cannot run while the six files above are empty.**

| # | Prior fix | Verdict | Evidence |
|:--|:--|:--|:--|
| 1 | **GRAMMAR-WHY human labels** in grammar inspector | **DONE (code), BLOCKED (runtime)** | `rust_engine_mcp/aps_grammar_labels.py` fully populated — `GRAMMAR_LABELS` + `GRAMMAR_WHY` + `human_label()`/`grammar_why_detail()`; `grammar_labels_v1.json` present. Inspector bytecode reads it (`Building:/Archetype:/District:/Massing:/Roof:/Facade:/Material strategy:` headings, `grammar_why_detail`). But `grammar_inspector.py` source is **empty** → panel won't construct. |
| 2 | **Status text beside glyphs** (materials + pipeline bar) | **DONE** | `material_library_widget.py:443` `_status_label()` → `"Ready · {id} · ●"` (word **first**, glyph suffix). `pipeline_status_bar.py:62-63` → `"✓ Catalog complete"` / `"○ Catalog pending"` (glyph + **word state**). Materials grid status line `"N shown · M ready · K total"`. |
| 3 | **Catalog list thumb + sidecar authority line** | **DONE** | `catalog.py:209` `render_module_list_thumb()` per row; `catalog.py:33-35,112-122` `SIDECAR_TRUTH` line + `cat_sidecar_truth` tooltip. Module-side `aps_catalog_preview.py` present. |
| 4 | **Atlas meta plain-language validate + UV grid** | **DONE** | `atlas_preview_panel.py:51-77` `_draw_uv_grid()` (columns×rows overlay + blue highlight); `atlas_panel.py:194` `on_validate_atlas_meta` → `aps_atlas_qc.format_atlas_qc_display()` plain text with color; grid legend string at `atlas_preview_panel.py:233`. |
| 5 | **Assembly next-step callout after generate** | **DONE** | `assembly_panel.py:609-613` sets `next_step_var` "Next: Select a footprint cell → Materials tab → Apply profile → Save snapshot…" after generate; `show_material_assign_callout()` at :396. |

**Net:** all five prior fixes were implemented in code. Four are fully self-contained; #1 depends on the now-empty `grammar_inspector.py`. The prior audit's recommended work landed — but the tool regressed to non-runnable afterward.

---

## 2. Re-scores (1–10) with delta vs 2026-06-03

Scores reflect the **delivered design** (bytecode + populated sources), then a separate **runnable-state** verdict that dominates ship readiness.

| Dimension | Prior (06-03) | Now (design) | Δ | Justification |
|:---|:--:|:--:|:--:|:---|
| **Clarity** | 6 | **7** | +1 | Authority strip, per-tab intros, metadata-flow panels, word-first status, grammar human labels, next-step callouts all present. Held back by dense Assembly tab (3-pane + 6 collapsibles + inline auth frame) and Consolas-8/Segoe-8 secondary text. |
| **Discoverability** | 5 | **6** | +1 | Flow bar (Send→Bake→Pack), pipeline bar, "Next:" callout, metadata-flow auto-expand-on-first-visit, tree counts. Still: critical actions are buttons-only with no menubar/keyboard accelerators; lod0/Advanced still inline not collapsed. |
| **Error recovery** | 5 | **6** | +1 | P0 "proceed anyway?" dialog with plain hints, inline FAIL strings, `format_p0_display`/`format_atlas_qc_display` plain language, JSON-parse guard in Catalog save. Still: failures don't jump to the offending field; some errors only land in collapsed status log. |
| **Accessibility** | 4 | **5** | +1 | Pipeline/materials status no longer color-only (text words). Still: footprint heatmap + material swatch + atlas QC remain **color-as-primary**; Segoe-8/Consolas-8 used for real labels; no keyboard path to apply material; MIN 960×600 over-packs Assembly. |
| **Workflow efficiency** | 6 | **6** | 0 | Job strip async + cancel, "From variant set"→batch wiring, folder auto-fill after batch are good. Offset by Assembly cramming and no save-state persistence across tabs beyond the metadata-flow prefs. |

**Runnable-state verdict: 0/10 — `artist_would_ship_today: false`.** The design trajectory is positive (composite ~6.0, up from 5.2), and would clear the plan's target of 7 after the §6 fixes **once the tree is restored**. But an artist literally cannot open the tool today.

`production_lens_ship_score`: **2/10** (was 5) — regressed purely on launch failure; **9/10 of the path to 7 is already built.**

---

## 3. Heuristic review per tab (Nielsen + game-tools lens)

### Flow bar / authority strip / pipeline bar (global chrome)
- **Good:** Authority strip ("Ship truth: assembly_snapshot…") is always visible and dynamically wrapped. Flow bar models the macro pipeline. Pipeline bar gives ✓/○ + word state per step and an honest "keyframe bake is behind Atlas" caveat.
- **Issues:** Flow-bar buttons (`Send to Assembly`, `Bake variants`, `Pack atlas`) duplicate per-tab actions and can fire no-op log lines when prerequisites are missing — the message is in the **collapsed-by-default status log**, so a new artist clicks and sees nothing happen. Pipeline bar "complete" = "has data", not "valid/shippable" — a snapshot that fails P0 still shows ✓ Assembly.

### Catalog
- **Good:** Thumb-per-row list, batch+category filters, sidecar-vs-truth line, AssetSpec/Index notebook split (index read-only). Validate gives `PASS/FAIL · verts · issues` inline.
- **Issues:** Module list rows use **Segoe-8** for the primary `module_id` (`catalog.py:223`) — the most-read label is the smallest font. Selection state on a row is only a `RIDGE` relief (subtle); no clear "selected" highlight. Thumb render is synchronous in `refresh_list` loop — many indexed GLBs = visible stall (no spinner, no thumb-job).

### Assembly (the heavy tab)
- **Good:** Strong information model — generate (grammar/plain), footprint canvas with token legend + selected outline, material library, four slot previews, semantic/variant tag pickers, grammar inspector, material-authority frame, P0 gate. "Next:" callout + save reminders are genuinely helpful.
- **Issues:** **Density overload.** A single tab stacks: intro, metadata-flow, grammar build-set, material-authority frame, Generate (with 2 collapsibles incl. an *expanded-by-default* "ARCH-DNA + β v0"), file row, then a **3-pane** workspace (footprint / materials / inspector) where the inspector pane itself holds slot previews + assembly preview + selected-slot editor + 2 more collapsibles + validation label. At MIN 960×600 the three panes (min 240+220+260 = 720 + sashes + padding) leave almost no inspector width and force horizontal scroll. The "ARCH-DNA + β v0" section title is pure jargon and is the only expanded-by-default advanced section.
- **Workflow risk:** material apply requires (1) select footprint cell → (2) pick profile → (3) Apply. The 3-step is documented in tooltips/hint but there is **no keyboard path** and the "Apply to selected slot" button lives inside the material preview strip, far from the footprint grid.

### Materials
- **Good:** Studio tree (Categories tree with counts + Profiles list + Preview/maps strip), search + category filter, word-first map status, thumb cache warming, generate/open-folder/registry actions, "Use in Assembly" round-trip.
- **Issues:** **Bug** — `material_library_widget.py:309` in `_category_matches_filter` references `entry.category` but the only bound name in that branch is the param `entry_category`; `entry` is undefined → `NameError` when a flat (non-tree) category filter is applied in studio layout. Map status still leans on glyphs (●◐○) as the at-a-glance signal even though the word is now present. Preview pane is a flat sphere/wall/section — fine, but the "Regenerate all pilots" button sits beside "Reload preview" with no separation (destructive-ish dev action next to a benign one).

### Variants (source empty — assessed from bytecode)
- **Design (recovered):** Load / Load example / New from assembly / Save JSON / Save RON / Validate; Layers section (Lighting day·night_off·night_on, Damage, fill, material override) → variant_key rows; bake hint "Atlas tab → From variant set → Run tile batch."
- **Issues:** Coherent layer model, but it is the least-connected tab — its only forward affordance is a text hint, not a button that jumps to Atlas with the batch staged (the flow-bar "Bake variants" does this, but it's not on the Variants tab itself). Cannot be reviewed live (empty source).

### Atlas
- **Good:** tile_batch path + "From variant set", Run tile batch (async job + cancel), PNG-folder field with live preview trace, Pack (tilemapgen), Refresh, **Validate atlas meta (plain language)**, Open PNG folder, packed-atlas-with-UV-grid + clickable source-cell strip with grid highlight, honest "keyframe bake in Blender is separate."
- **Issues:** Tab is a **long vertical stack** of ~10 button rows + log; lod0 batch + phase combobox are advanced/CI controls inline with artist QC controls (plan called for "collapse lod0 behind Advanced" — **not done**). Two separate status surfaces (`_inline_status_lbl` and `_atlas_qc_lbl`) can disagree. "-pk rename" checkbox label is cryptic (tooltip explains, but label alone is unreadable).

---

## 4. Accessibility & readability pass

| Checklist item | Status | Detail |
|:---|:---|:---|
| Status not color/glyph alone | **PARTIAL** | Pipeline + materials list now word-first. **Still color-primary:** footprint heatmap tokens (`footprint_canvas.py` — token letter is shown only at ≥24px cells, otherwise color-only), material swatch (`assembly_panel.py:846` color block, no text), atlas inline status (color via `_inline_hint` fg only). |
| Min font sizes (no Consolas/Segoe ≤8 for primary) | **FAIL** | `aps_theme.py` base fonts OK (Segoe 9, Consolas 10). **But hardcoded `("Segoe UI", 8)` for primary-ish content:** catalog row module_id label (`catalog.py:223`), several hint/legend/slot-why labels, footprint cell glyph `("Consolas", 8, "bold")`, material list category line, slot preview titles. The plan explicitly flagged "avoid Consolas 8 / Segoe 8 for primary labels." |
| Critical actions not tooltip-only | **PASS (mostly)** | Primary actions are visible buttons. Risk: the 3-step material-apply sequence and the meaning of "-pk rename" / "ARCH-DNA + β v0" rely on hover tooltips for comprehension. |
| Scroll regions obvious | **PASS** | `aps_scroll` adds debounced scrollregion + wheel areas; scrollbars on lists/canvases/text. Good. |
| Paned usable at 960×600 (MIN) | **FAIL** | Assembly 3-pane min widths (240+220+260) + sashes + 8px paddings exceed comfortable 960 content width → horizontal scroll and a near-unusable inspector. Materials studio nested panes (240 + 320, then 140+180 inside) also tight. |
| Metadata→engine understandable w/o ARCH-MAT doc | **PASS** | `metadata_flow_panel.py` per-context plain-language blocks; auto-expands on first visit and remembers collapse state via `aps_ui_prefs.json`. Strong. |
| Pipeline mental model matches | **PASS (with caveat)** | Steps map to tabs. Caveat: "complete" = data-present, not valid (see §3 flow bar). |
| Keyboard path (list→grid→apply) | **FAIL** | No accelerators, no menubar, no Tab-order guarantee into the canvas grid (footprint canvas is click-only — `_on_click`, no key bindings). An artist cannot complete the core loop without a mouse. |

---

## 5. Top 10 ranked issues

| # | Pri | Issue | File(s) |
|:--|:--|:--|:--|
| 1 | **P0** | **Tool will not launch** — 6 source files empty (`aps_tooltips`, `job_controller`, `scrollable`, `grammar_inspector`, `assembly_preview_panel`, `variants_panel`); `ImportError` on `bind_aps_tooltip`. | the 6 empty `.py` |
| 2 | **P0** | No regression guard catches an empty/uncompilable APS module → a "waves" commit silently shipped a dead tool. | CI / `pytest`, `tools/mcp/python/tests/` |
| 3 | **P1** | `NameError` latent in studio-tree category filter (`entry` undefined; param is `entry_category`). | `material_library_widget.py:293-309` |
| 4 | **P1** | Assembly tab unusable at MIN 960×600 — 3-pane min widths overflow; inspector pane starved. | `assembly_panel.py:210-222`, `aps_theme.py` MIN_WINDOW_SIZE |
| 5 | **P1** | Primary labels at Segoe/Consolas **8px** (catalog module_id, footprint glyph, slot titles, hint lines). | `catalog.py:223`, `footprint_canvas.py:233`, `slot_preview_panel.py:74`, `aps_theme.py` (no FONT_SMALL token) |
| 6 | **P1** | No keyboard path through the core loop (select cell → pick profile → apply); footprint canvas is mouse-only. | `footprint_canvas.py`, `assembly_panel.py`, `material_library_widget.py` |
| 7 | **P1** | Pipeline bar "complete" = has-data, not P0-valid — a failing snapshot reads ✓ Assembly. | `pipeline_status_bar.py:42-63` |
| 8 | **P2** | Atlas tab over-long; lod0/CI controls inline with artist QC (plan wanted lod0 behind "Advanced"). | `atlas_panel.py:133-169` |
| 9 | **P2** | Color-as-primary still present: footprint heatmap (<24px cells), material swatch, atlas inline status. | `footprint_canvas.py`, `assembly_panel.py:846`, `atlas_panel.py:189-192` |
| 10 | **P2** | Flow-bar buttons no-op into the **collapsed** status log when prerequisites missing → silent click; "ARCH-DNA + β v0" jargon section expanded by default. | `app.py:223-247`, `assembly_panel.py:180-188` |

---

## 6. Top fixes for @coder-mcp (specific, implementable)

> **F1 must land before any other — the tool is dead until then.**

### F1 — Restore the six empty APS source files + add a launch guard `[P0]`
- **Files:** `aps_tooltips.py`, `job_controller.py`, `scrollable.py`, `grammar_inspector.py`, `assembly_preview_panel.py`, `variants_panel.py`; new test in `tools/mcp/python/tests/`.
- **Change:** Decompile the committed `__pycache__/*.cpython-313.pyc` (larger/complete) to restore each source verbatim; if 3.14 decompile fails, fall back to 3.13 bytecode or re-author from the string map in §7. Then add `test_aps_imports.py` that does `import art_pipeline_suite.app` and asserts `bind_aps_tooltip`, `JobController`, `ScrollableFrame`, `VariantsPanel`, `GrammarInspectorPanel`, `AssemblyPreviewPanel` are importable and non-empty.
- **Acceptance:** `python -m art_pipeline_suite.run` opens to the Catalog tab without exception; new test green in CI; `git diff --stat` shows the six files non-zero.

### F2 — Fix studio-tree category filter `NameError` `[P1]`
- **File:** `material_library_widget.py`, `_category_matches_filter` (~:293-309).
- **Change:** Rename the parameter to `entry` (and pass the entry object), OR replace the trailing `entry.category.lower()` with `entry_category.lower()`. Add a unit test that calls the studio-tree filter with a flat category selected.
- **Acceptance:** Selecting a flat category in Materials studio layout filters without raising; `pytest` covers the branch.

### F3 — Make Assembly usable at MIN window; add an "Advanced" gate `[P1]`
- **Files:** `assembly_panel.py` (workspace panes + the two always-on advanced sections), `aps_theme.py`.
- **Change:** (a) Default-collapse "ARCH-DNA + β v0" (`expanded=False`) and group it + "Iterate grammar" + grammar inspector under one "Advanced" accordion. (b) At widths < ~1100, switch the 3-pane workspace to a 2-pane (footprint+materials) with the inspector in a `ttk.Notebook` tab or below, OR reduce pane minsizes to fit 960 (e.g. 220/200/220) and verify no horizontal scroll at MIN.
- **Acceptance:** At 960×600 the Assembly tab shows footprint grid, material list, and slot editor with no horizontal scrollbar; advanced sections start collapsed.

### F4 — Introduce a readable small-font token + bump primary labels `[P1]`
- **Files:** `aps_theme.py` (add `FONT_SMALL = ("Segoe UI", 9)` and stop using literal `("Segoe UI", 8)`), `catalog.py:223`, `slot_preview_panel.py`, `footprint_canvas.py`.
- **Change:** Replace hardcoded size-8 fonts on *primary* content (catalog `module_id`, slot-preview titles, footprint cell glyph) with `FONT_UI`/`FONT_SMALL` (≥9). Keep 8 only for truly tertiary captions if at all.
- **Acceptance:** No primary label renders below 9px; visual check that catalog list and footprint tokens are legible at 100% scale.

### F5 — Pipeline bar shows validity, not just presence `[P1]`
- **File:** `pipeline_status_bar.py:42-63`, plus a hook from `assembly_panel` P0 result into `SuiteState`.
- **Change:** Add a third state. For Assembly: `○ pending` → `◐ saved (P0 not run)` → `✓ valid` (only after P0 passed). Persist last P0 status on `SuiteState` and read it in `refresh()`. Keep word-first text.
- **Acceptance:** A snapshot that fails P0 never shows `✓ Assembly`; passing P0 flips it to `✓ valid`.

### F6 — Footprint + swatch + atlas status: add non-color signal `[P2]`
- **Files:** `footprint_canvas.py` (always draw the token glyph regardless of cell size, or add a per-cell hatch by role), `assembly_panel.py:846` (`_update_material_swatch` — add the profile_id text/initial next to the swatch), `atlas_panel.py` (`_inline_hint` — prefix `PASS:`/`FAIL:` text, not just fg color).
- **Change:** Ensure every status currently conveyed by color also carries a glyph/word at all sizes.
- **Acceptance:** With a grayscale/colorblind sim, footprint roles, material identity, and atlas pass/fail remain distinguishable.

### F7 — Stop silent no-op clicks on the flow bar `[P2]`
- **File:** `app.py:223-247`.
- **Change:** When a flow-bar action lacks prerequisites, surface the guidance via a transient inline banner near the flow bar (or auto-expand the status log) instead of only `self._log(...)` into the collapsed log.
- **Acceptance:** Clicking "Bake variants" with no snapshot shows visible guidance without the artist opening the status log.

---

## 7. Tooltip copy review (`aps_tooltips.py` — recovered from bytecode; source is empty)

The tooltip dictionary (`TOOLTIPS`, ~90 keys) was recovered from `__pycache__/aps_tooltips.cpython-314.pyc`. **It is high quality** — designer-reviewed, plain-language, consistently reinforces "snapshot is ship truth." It must be restored verbatim under F1. Below: keys that are jargon-heavy or could be sharper. No keys are *missing* for existing widgets (every primary button binds a key), but note the gaps where UI lacks a key.

| Key | Current | Verdict | Proposed |
|:---|:---|:---|:---|
| `atl_keyframe_rename` | "Rename keyframe PNGs to pack-friendly names before tilemapgen." | Label "-pk rename" is cryptic; tooltip OK but rename the **label**, not the tip. | Keep tip; change checkbox label to "Rename keyframe PNGs (-pk)". |
| `asm_iterate` | "Change one grammar layer without full seed reroll — Apply iteration when Phase 2 ships." | "when Phase 2 ships" leaks roadmap state to the artist. | "Change one grammar layer without a full seed reroll. Re-rolls only that layer." |
| `atl_lod0` / `atl_batch` | "CI/smoke ortho batch — not ship art by itself. Collapse under Advanced when polish lands." | "Collapse under Advanced when polish lands" is a dev TODO, not artist help. | "CI/smoke ortho batch — not ship art. Use only for engine smoke tests." |
| `asm_grammar_dna` (ARCH-DNA section) | *(no tooltip key bound to the section header)* | The section title "ARCH-DNA + β v0" is the worst jargon in the app and has no explanatory hint at the header. | Add key `asm_grammar_dna`: "Advanced massing pressure (yard/module density). Leave default unless tuning grammar." and bind to the section. |
| `asm_preview_thumb` | "...set RUST_ENGINE_BEVY_PREVIEW=0 for browser-only." | Exposes an env var to artists. | "...Blank? Wait for the spinner, then Retry. (Browser-only fallback exists for dev.)" |
| `pipeline_step` | "Done = you have data for this step. Pending = visit the tab..." | Reinforces the data≠valid problem (F5). Update once F5 lands. | "Valid = step passed its check. Saved = data exists, not yet validated. Pending = not started." |
| `cat_list_thumb` | "Select module — thumb shows isolated GLB when indexed." | Fine. | — |
| `asm_footprint` / `asm_footprint_heatmap` | clear, color-meaning called out | Good — already says "Colors show role density … not ship status." | — |

**Action for @coder-mcp:** restore the dictionary verbatim (F1), then apply the ~5 string edits above and add the `asm_grammar_dna` key. Net change is small; the corpus is solid.

---

## 8. Diagnostics required (this re-audit)

- Re-run the import probe in CI after F1 (`test_aps_imports.py`).
- Existing `pytest test_aps_preview_001.py test_aps_atlas_preview.py` must stay green; add `test_aps_imports` and a `material_library` filter test (F2).
- Re-emit `debug_runs/aps_ux_audit_001_live.json` with this re-audit's scores and `artist_would_ship_today: false` (reason: launch failure), distinct from the design-trajectory scores.

## 9. Risks / tradeoffs

- **Decompile fidelity:** restoring from `.pyc` may lose comments/formatting and can fail on 3.14 bytecode. Mitigation: prefer 3.13 pyc; review diffs against this doc's behavioral notes; the modules are self-contained widgets so behavioral verification is straightforward (launch + click-through).
- **F3 layout change** risks disturbing the sign-off baseline (1280×800). Mitigation: keep 1280×800 layout as-is; only change behavior below ~1100px and collapse-state defaults.
- **F5 validity state** adds a `SuiteState` field that must be reset when a new snapshot is generated/loaded to avoid a stale ✓.
