# APS Sweep — Tab Design & Information Architecture dimension `v1`

| Field | Value |
|:---|:---|
| **Program** | APS-OVERHAUL · dimension audit 1 of 4 — **Tab/IA structure only** |
| **Owner** | `@designer` (IA · navigation · authority placement) |
| **Date** | 2026-06-16 |
| **Scope** | `tools/mcp/art_pipeline_suite/*` — tab order, tab labels, per-tab authority/content ownership, cross-tab flow, lane structure |
| **Out of lane (other auditors)** | text/copy, layout/density, visual style/tokens. Flagged only where they collide with IA. |
| **Grounding** | LIVE read: `app.py`, `domain_router.py`, `state.py`, `pipeline_status_bar.py`, `aps_inline_feedback.py`, `catalog.py`, `assembly_panel.py`, `materials_panel.py`, `variants_panel.py`, `atlas_panel.py`, `landscape_{presets,grammar,states}_panel.py`; prior IA proposal `design_aps_uiux_style_quality_20260616_v1.md` §2 |

---

## 0. State-of-the-tool — the lane switch is ALREADY LIVE (not just proposed)

The brief asks me to "evaluate and FINALIZE" the lane-switch proposal. Critical correction grounded in code: **Option D is already implemented and shipping on disk.** `app.py` sets `OPTION_D_DUAL_NOTEBOOK = True` and builds **two** `ttk.Notebook`s (`_notebook_buildings`, `_notebook_landscape`) swapped by a top-of-window `Lane:` radiobutton segment (`_build_lane_bar`, `_apply_lane`), with `Ctrl-1`/`Ctrl-2` accelerators, a lane-tinted underline + chip, a lane-scoped flow bar, lane-scoped authority strip, and a lane-scoped pipeline bar (`pipeline_status_bar.py` keys `STEPS` by lane). `domain_router.py` declares the canonical tab sets:

- **Buildings:** `Catalog · Assembly · Materials · Variants · Atlas` (5)
- **Landscape:** `Presets · Grammar · States · Atlas` (4)

So the FINALIZE task is no longer "should we do Option D" — it is "is the *implemented* Option D structurally correct, and what IA debt did the build leave behind." My verdict (§4) **CONFIRMS Option D with refinements**, because the build already proved the lane switch out but introduced three concrete IA inconsistencies that an audit-on-paper would have missed.

---

## 1. Tab-by-tab — OWNS WHAT (live) vs SHOULD OWN

### Buildings lane

| Tab | Owns today (live code) | Authority object | Misplacement / leak | Should own |
|:---|:---|:---|:---|:---|
| **Catalog** (`catalog.py`) | Module browser/filter (batch·category), AssetSpec sidecar editor, Index-entry view, GLB validate, reindex, browser/trimesh preview. **Also re-sources landscape presets** via `_refresh_landscape_presets` + `set_domain(landscape)`. | module library + sidecar (input) | **LEAK A — dead dual-source:** the Buildings Catalog panel still contains a full landscape-preset code path (`_refresh_landscape_presets`, lane-aware `set_domain`) that is **unreachable** now that Landscape has its own `Presets` tab built from `LandscapePresetsPanel`. Two panels (`CatalogPanel` + `LandscapePresetsPanel`) both read `list_landscape_presets`. | module library + sidecar **only**. Strip the landscape branch — it is pre-Option-D residue and a future bug magnet (two preset readers drifting). |
| **Assembly** (`assembly_panel.py`, 966 ln) | Footprint grid, grammar toggle/archetype/district, **the Material library browser + per-slot `material_profile` ASSIGNMENT** (`MaterialBrowserPanel` → `_apply_material_profile`), semantic/variant tags, slot preview, assembly preview, P0 validate, snapshot gen/save. | `assembly_snapshot` (ship truth) | **LEAK B — material authority split across 2 tabs** (see §2). Assembly is the de-facto heaviest tab and also the place material is *assigned*. | snapshot authoring: grid + modules + tags + **material assignment** (assignment is correctly here — see §2 ruling). Shed nothing structurally; this is the spine. |
| **Materials** (`materials_panel.py`) | Material **Studio**: browse/generate/edit profiles, drop PNGs, reload preview, preview modes. Cross-link "Open in Assembly". Intro literally says *"Assign on the Assembly tab."* | `material_profiles` library (input) | Reads as a sibling of Assembly but is **upstream** of it. The two-way cross-link (`on_open_in_materials` ↔ `on_open_in_assembly`) is good, but the artist sees "Materials" *after* "Assembly" in tab order while the data flows Materials→Assembly. | profile *library* management (create/edit/preview). Assignment stays in Assembly. Tab ORDER should put it before Assembly (see §3). |
| **Variants** (`variants_panel.py`) | `variant_set_v1` layers (lighting/power/damage/fill), **a `Material` free-text entry** (`material_var`, line ~115), tags, agent-patch strip, bake-selected. | `variant_set` (derived) | **LEAK C — third material surface:** a raw `Material` text field for `wall_material` lives here too, disconnected from both the Materials studio and Assembly's profile assignment. Free-text = no validation against the profile catalog. | variant layer authoring. The material layer should reference a profile id from the catalog (dropdown sourced from Materials), not a free-text field — otherwise three tabs each touch "material" with three different mental models. |
| **Atlas** (`atlas_panel.py`) | `tile_batch_run` + `tile_atlas_pack`, register target banner, atlas preview, LOD0/CI controls inline. Lane-aware register (`_tile_atlas_index` vs `_landscape_atlas_index`). | packed atlas (output) | CI/lod0 controls inline make it long (carried P2 from prior audit — layout dimension owns the declutter). IA-wise it is correctly the terminal output tab. | atlas pack + QC. **Shared panel class** (`AtlasPanel`) is reused for both lanes — good reuse, but it forces "Atlas" to mean two different ship gates (building tile QC vs LG-5 G0–G5). |

### Landscape lane

| Tab | Owns today (live) | Authority object | Misplacement / leak | Should own |
|:---|:---|:---|:---|:---|
| **Presets** (`landscape_presets_panel.py`) | Browse/clone 10 presets, `land_dna`+topology plain summary, validate vs `landscape_grammar_v0` schema. | landscape preset (input) | Overlaps the dead Catalog landscape path (LEAK A) — but this is the *correct* home; Catalog's copy is the duplicate to delete. | preset browse + validate. Correct. |
| **Grammar** (`landscape_grammar_panel.py`) | Topology-graph workspace (Network/Corridor/Ring/Patch/Cluster/Fringe), no footprint grid. | `topology_graph` (ship truth) | None structural. The veg analogue of Assembly, correctly graph-not-grid. | grammar authoring. Correct. |
| **States** (`landscape_states_panel.py`) | Succession/disturbance matrix (burn/scar/recovery/harvest), catalog axes, extract-parity panel. | state matrix (derived) | None structural. Veg analogue of Variants. | succession + disturbance. Correct. |
| **Atlas** (shared `AtlasPanel`) | LG-5 tile atlas pack + landscape register (`_landscape_atlas_index`). | LG-5 atlas (output) | The G0–G5 scope-explicit gate the prior doc demanded is **not visibly enforced** in the shared panel — register shows a single PASS/FAIL (`register_green`), risking the "one green word flattens 6 gates" warning. | LG-5 atlas QC with scope-explicit G-gate. Needs the gate copy (text dimension) + a structural sub-state, not one ✓. |

---

## 2. The material-authority question — three surfaces, one ruling

Material touches **three** tabs today:

1. **Materials tab** — the profile *library* (create/edit/preview profiles). Upstream input.
2. **Assembly tab** — per-slot `material_profile` *assignment* via the embedded `MaterialBrowserPanel`. This is where a profile gets bound to a placement.
3. **Variants tab** — a free-text `Material` field writing `wall_material` into a variant layer.

**Ruling (IA, not layout):**
- Assignment-in-Assembly is **correct and must stay** — material is a property of a placement in the snapshot (the ship-truth object), so assigning it where you author placements is the right mental model. The split is not "assignment is in the wrong place."
- The real defect is that **all three use different vocabularies for the same concept**: Materials calls them `profiles`, Assembly assigns `material_profile`, Variants takes free-text `wall_material`. An artist cannot form one mental model of "material." 
- **Fix:** Variants' material layer must become a **profile-id dropdown** sourced from the same catalog Materials manages (kills LEAK C's free-text drift). Materials tab's purpose must be relabeled in the IA as the *library/studio* (it already says "Assign on the Assembly tab" — make that the structural contract, not a sentence). Net: Materials = library, Assembly = assignment, Variants = references-by-id. One concept, three roles, never three vocabularies.

This is the single biggest IA smell in the Buildings lane and it predates the lane work.

---

## 3. Tab ORDER vs workflow

**Buildings tab order today:** `Catalog → Assembly → Materials → Variants → Atlas`.
**Actual data/authoring flow:** Catalog (pick modules) → **Materials** (have profiles ready) → Assembly (place + assign profiles + validate P0) → Variants (derive) → Atlas (pack).

The tab row puts **Materials *after* Assembly**, but the artist needs profiles to exist *before* they can assign them in Assembly. The flow bar even routes `Send to Assembly` from Catalog, skipping Materials entirely. Two defensible orders:

- **Option 3a (recommended): `Catalog → Materials → Assembly → Variants → Atlas`.** Matches the prerequisite order (profiles before assignment). Cost: re-orders an established tab the team has muscle memory for; the flow bar (`send_to_assembly`) is unaffected.
- **Option 3b (status-quo, defensible): keep `Catalog → Assembly → Materials → Variants`** and treat Materials as a *just-in-time side trip* reached via Assembly's "Open in Materials tab" cross-link. The existing bidirectional cross-link already supports this. 

**Verdict:** keep current order **only if** the cross-link is elevated to a first-class "no profile? → Materials" affordance at the point of assignment in Assembly. Otherwise adopt 3a. I lean **3a** — the pipeline bar already lists `materials` as a *step between* assembly and variants, so the pipeline mental model and the tab order disagree today (pipeline says Catalog·Assembly·Materials·Variants·Atlas, which is itself out of authoring order). Aligning tab order to true prerequisite order removes that contradiction.

**Landscape tab order** (`Presets → Grammar → States → Atlas`) matches its flow (`Generate grammar → Bake states → Pack LG-5 atlas`) cleanly. No change.

---

## 4. Lane-switch verdict — CONFIRM (with refinements)

**CONFIRM Option D** (top-level Buildings⇄Landscape lane switch, dual notebook page sets — NOT a 6th tab). It is already live and the architecture is sound:
- Lanes are isolated (separate notebooks, `clear_cross_lane_selection` on switch — satisfies multiview-safety, no silent cross-lane state bleed).
- Each lane has purpose-built tabs (no mega-tab; the rejected 6th-tab option would have crammed preset+graph+states+atlas-QC into one tab worse than Assembly).
- Chrome is lane-scoped (authority strip, flow verbs, pipeline steps all keyed by lane).
- Scales to a 3rd lane without touching existing lanes (add a notebook + a `*_BY_LANE` dict entry).

**Refinements required (IA debt the build left):**

- **R1 — `Stamp` step has no tab (authority-without-home).** `PIPELINE_STEPS_BY_LANE[landscape]` has **5** steps `presets·grammar·states·atlas·stamp`, but the Landscape notebook has **4** tabs (no `Stamp`). The pipeline pill `stamp` (`stamp_pending`/`stamp_done`, set by `on_pack_lg5_atlas`) is a dead-end: the artist sees a pipeline step they can never navigate to. Either (a) fold "stamp registered" into the Atlas tab's terminal state (recommended — it is the atlas register action), or (b) add a real terminal tab. Today it is an orphan. *(Note: `domain_router.verify_option_d_ia_contract` even hard-codes the 5-key landscape pipeline as the pass condition, cementing the mismatch.)*

- **R2 — dead landscape path in Catalog (LEAK A).** `CatalogPanel` still carries a full landscape-preset branch made unreachable by the dedicated `Presets` tab. Delete it so there is one preset reader.

- **R3 — "Atlas" is overloaded across lanes.** Both lanes' last tab is literally labeled `Atlas` using the same `AtlasPanel` class but they are different ship gates (tile QC vs LG-5 G0–G5). Acceptable reuse, but the **label** should disambiguate per lane (e.g. Buildings `Atlas`, Landscape `Atlas (LG-5)`), and the LG-5 tab must surface the G-gate scope, not collapse to one PASS/FAIL `register_green`.

- **R4 — lane switch lives below the title bar but the *flow bar also moved* per lane.** This is correct, but means the artist's "where am I" cue is now spread across lane chip + underline + flow verbs + authority strip + pipeline. Good redundancy (non-color-only), but the audit confirms it must stay *consistent*: every lane-scoped surface must repaint on switch. `_apply_lane` does refresh all five — verified. Keep it; add a guard test (handoff).

**Counter-arguments considered and rejected:** A 6th tab (rejected — mega-tab), nested notebooks (rejected — Tk keyboard/a11y nightmare), context-auto-morph (rejected — silent re-meaning violates spatial consistency). All three were correctly rejected in the prior doc and the live build agrees.

---

## 5. FINALIZED tab/lane structure

```text
┌ LANE (persistent top segment, Ctrl-1/Ctrl-2) ──────────────────────────────┐
│  [ ▣ Buildings ]   [ Landscape ]     ← swaps the whole notebook + chrome     │
└──────────────────────────────────────────────────────────────────────────-─┘

BUILDINGS lane   (5 tabs — order corrected)
   Catalog  →  Materials  →  Assembly  →  Variants  →  Atlas
   │           │            │            │            └ pack + tile-QC (terminal)
   │           │            │            └ variant_set; material layer = profile-id dropdown (not free-text)
   │           │            └ snapshot authority: grid + tags + MATERIAL ASSIGNMENT (stays here)
   │           └ material profile LIBRARY/studio (create/edit/preview) — assignment is NOT here
   └ module library + sidecar ONLY (landscape branch deleted)

LANDSCAPE lane   (4 tabs — order already correct)
   Presets  →  Grammar  →  States  →  Atlas (LG-5)
   │           │           │          └ LG-5 pack + G0–G5 scope-explicit gate (stamp folds in here)
   │           │           └ succession + disturbance matrix
   │           └ topology-graph ship truth (graph, not grid)
   └ preset browse + validate (the ONE preset reader)
```

Pipeline bars (lane-scoped, one pill per **real navigable tab**):
- Buildings: `Catalog · Materials · Assembly · Variants · Atlas` (re-ordered to match tabs).
- Landscape: `Presets · Grammar · States · Atlas` — **drop the orphan `Stamp` pill** (fold into Atlas terminal state) OR give it a tab. Pick fold-in.

**Scaling rule for a future 3rd lane (e.g. Interiors/Props):** add one `ttk.Notebook` + one entry in each `*_BY_LANE` dict in `domain_router.py` + one lane radiobutton. No existing lane is touched. The flat-per-lane notebook (4–5 tabs) is the right container: it stays under the ~7-tab cognitive limit per lane, avoids nested notebooks, and the lane segment absorbs growth horizontally. Do **not** let any single lane exceed ~6 tabs — beyond that, split the lane, do not nest.

---

## 6. Cross-tab flow recommendations

1. **Flow bar is the spine — keep it lane-scoped (already done), but fix the order contradiction.** Buildings flow `Send to Assembly · Bake variants · Pack atlas` jumps Catalog→Assembly, silently skipping Materials. If tab order becomes `Catalog→Materials→Assembly` (3a), the flow still works, but add an implicit "profiles ready?" check to `send_to_assembly` prereq (today it only checks a module is selected).
2. **Prerequisite messaging is good but buried.** `flow_prerequisite_message` returns rich lane-aware guidance, but `_show_flow_prerequisite` writes it to `_flow_hint_var` (a small label) AND the collapsed status log. Confirm it surfaces *at the button*, not only in the log (prior P1 "flow-bar silent no-op" — verify it is fully closed; the hint label exists but is shared and easy to miss).
3. **The two cross-links Assembly↔Materials are the model for all cross-tab nav.** `_open_material_in_assembly` / `_open_material_in_materials_tab` jump tabs + highlight + callout. This is the right pattern. Replicate it for Variants→Materials (when the material layer becomes a profile dropdown, "edit this profile" should jump to Materials).
4. **Atlas is a true terminal — no dead-ends, but the `Stamp` orphan is one.** Resolve R1 so the artist never sees a pipeline step with no destination.
5. **Lane switch must never strand selection.** `clear_cross_lane_selection` handles this; keep it and add the isolation guard test.
6. **Pipeline bar = the navigation map.** Today its step list disagrees with tab order (Buildings) and tab count (Landscape +Stamp). Make pipeline-step keys === navigable-tab set per lane, in tab order. Then the pipeline bar doubles as a legible "you-are-here / what's-next" map and clicking a pill could (future) select that tab.

---

## 7. Top IA problems (current 5-tab + lane build), ranked

| # | Problem | Severity | Fix |
|:--|:---|:--|:---|
| 1 | **Material authority uses 3 vocabularies across 3 tabs** (Materials `profile` / Assembly `material_profile` assign / Variants free-text `wall_material`). | P0 | One concept: Materials=library, Assembly=assign, Variants=profile-id dropdown. §2. |
| 2 | **`Stamp` pipeline step has no tab** — dead-end navigation; artist sees a step they can't reach. | P0 | Fold stamp into Atlas terminal state; drop the orphan pill. R1. |
| 3 | **Dead landscape path inside Buildings Catalog** — two preset readers, unreachable code, drift risk. | P1 | Delete Catalog's landscape branch; Presets tab is sole reader. R2/LEAK A. |
| 4 | **Buildings tab order fights the prereq flow** — Materials sits *after* Assembly though profiles are needed *before* assignment; tab order also disagrees with pipeline-bar order. | P1 | Re-order to `Catalog→Materials→Assembly→Variants→Atlas` (3a). §3. |
| 5 | **"Atlas" label/class overloaded across lanes** — same word, two different ship gates (tile QC vs LG-5 G0–G5). | P1 | Disambiguate label (`Atlas (LG-5)`); surface G0–G5 scope, not one `register_green`. R3. |
| 6 | **Pipeline-bar step set ≠ navigable-tab set** (order in Buildings, +Stamp in Landscape). The nav map lies about the workflow. | P1 | Keys === tabs, in tab order, per lane. §6.6. |
| 7 | **Variants' free-text material field bypasses the profile catalog** — no validation, invites typos that silently diverge from ship truth. | P1 | Profile-id dropdown sourced from Materials catalog. §2/LEAK C. |
| 8 | **Cross-tab nav is excellent in one place (Assembly↔Materials) and absent elsewhere** — no Variants→Materials jump; no pipeline-pill→tab jump; flow prereq hint easy to miss. | P2 | Generalize the cross-link pattern; verify prereq surfaces at the button. §6.2–6.3. |

---

## 8. Diagnostics / guards (handoff to @coder-mcp)

- **Lane-IA contract guard exists but is wrong:** `domain_router.verify_option_d_ia_contract()` asserts the Landscape pipeline === 5 keys incl. `stamp`. After R1 (drop stamp), this contract + any test pinned to it must change to 4 keys. Flag before refactor.
- **New guard — pipeline-keys === navigable-tab-labels per lane** (catches the order/+Stamp drift permanently). 
- **New guard — single preset reader** (assert Catalog no longer imports `list_landscape_presets`) after R2.
- **New guard — lane-state isolation** (no cross-lane selection bleed; already behaviorally handled by `clear_cross_lane_selection`).
- Keep `test_aps_imports.py` (every panel imports) and the IA-contract test (updated).

---

## Sign-off

```text
APS-SWEEP-TABDESIGN-001 complete (dimension 1 of 4 — IA/tab structure only)
Lane verdict: CONFIRM Option D (already LIVE, OPTION_D_DUAL_NOTEBOOK=True) + 4 refinements (R1 stamp-orphan, R2 dead catalog path, R3 atlas overload, R4 keep-consistent-chrome)
Finalized: Buildings = Catalog·Materials·Assembly·Variants·Atlas (re-ordered) · Landscape = Presets·Grammar·States·Atlas(LG-5)
Top P0s: material 3-vocabulary split · Stamp dead-end pipeline step
3rd-lane rule: +1 notebook +1 *_BY_LANE entry, never nest, ≤6 tabs/lane
```
