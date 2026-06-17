# APS-ARTIST-SHIP-REVIEW-20260615 — Designer-MCP artist-acceptance re-verdict `v1`

| Field | Value |
|:---|:---|
| **Program** | **APS-ARTIST-SHIP-REVIEW-001** (re-verdict of APS-ARTIST-TOOL-E2E + APS-UX-AUDIT-001) |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-15 |
| **Lens** | Artist workflow coherence · material/module QC criteria · pilot acceptance (NOT general UX/heuristics) |
| **Verdict** | **FAIL** |
| **Ship score** | **2 / 10** (prior 5/10 — **−3**) |
| **Prior gate** | `design_aps_artist_tool_e2e_review_v1.md` (PASS WITH NOTES, 2026-06-03) |

---

## Headline

**The tool does not launch and the canonical artist E2E driver does not import.** Multiple core
source files have been truncated to **zero bytes** in the recent `5a340510 "waves"` /
`a982ff20 "tools and wild refactor"` commits. The green witnesses on disk
(`aps_artist_tool_e2e_live.json: green:true`, `aps_artist_tool_modules_live.json: "15 passed"`)
**no longer reflect the working tree** — they assert pass while the code they describe cannot be
collected by pytest, let alone run. Per my operating contract this is the
`honest_gate: dishonest_gate` red-stop: a witness claiming green over a broken tree.

This is a regression review, not a polish review. No amount of workflow nuance matters until the
suite imports.

---

## Order critique

```yaml
order_critique:
  request_summary: "Artist-acceptance re-verdict of APS authoring tool: would a working artist ship shippable assets end-to-end without Blender?"
  concerns:
    - "Canonical artist path (Catalog→Assembly→Materials→Variants→Atlas) cannot be exercised — app.py import chain is broken at first panel import."
    - "E2E witness driver aps_artist_tool_e2e.py imports symbols (render_module_list_thumb, render_material_preview) from zero-byte modules — the witness on disk is stale/dishonest vs current tree."
    - "9 APS-related pytest modules error at COLLECTION; the '15 passed' claim is not reproducible today."
  rules_audit:
    no_ai_generated_images: pass     # no AI-as-final-art affordance; procedural + drop-authored-PNG only
    deterministic_output: pass       # seed field present in Assembly generate; snapshot is deterministic input
    batch_processing: pass           # tile_batch_v1 + variant_set expansion path present in atlas_panel (when it runs)
    grid_alignment: n/a              # cannot exercise footprint canvas / atlas UV grid in a non-launching app
  blocked: true
  reroute: "Restore the six zero-byte UI files + three zero-byte backend modules BEFORE any artist-acceptance sign-off. Re-run pytest and regenerate witnesses honestly. Then re-review."
  foresight_flags:
    - "Witnesses are decoupled from build health — a green JSON can survive a code wipe. Add an import/launch smoke gate that the witness depends on."
    - "Variants tab is in the canonical path but the panel is now empty — Variants step of E2E is untestable."
    - "grammar_inspector empty → Assembly 'why this module' legibility (APS-UX-GRAMMAR-WHY) regressed to nothing."
  proceed: no
```

---

## Evidence (working tree at HEAD `5a340510`)

### Zero-byte UI files (`tools/mcp/art_pipeline_suite/`)

| File | Imported by | Status at `a982ff20` | Status now |
|:---|:---|:---:|:---:|
| `aps_tooltips.py` | every panel (`bind_aps_tooltip`) | 0 (already empty) | **0** |
| `job_controller.py` | `app.py` (`JobController`, `JobRecord`, …) | 0 (already empty) | **0** |
| `scrollable.py` | `app.py` (`ScrollableFrame`) | 80 lines | **0** |
| `variants_panel.py` | `app.py` (`VariantsPanel`) | 377 lines | **0** |
| `grammar_inspector.py` | `assembly_panel.py` (`GrammarInspectorPanel`) | 133 lines | **0** |
| `assembly_preview_panel.py` | `assembly_panel.py` (`AssemblyPreviewPanel`) | 157 lines | **0** |

### Zero-byte backend modules (`tools/mcp/python/rust_engine_mcp/`)

| File | Imported by | Missing symbol |
|:---|:---|:---|
| `aps_catalog_preview.py` | `aps_artist_tool_e2e.py` | `render_module_list_thumb` |
| `aps_slot_preview.py` | `aps_artist_tool_e2e.py` | `render_material_preview` |
| `aps_mat_002.py` | material studio path | (module body) |

### Reproductions

```text
# App import — fails at first panel
$ python -c "from art_pipeline_suite import app"
ImportError: cannot import name 'bind_aps_tooltip' from 'art_pipeline_suite.aps_tooltips'

# Canonical E2E witness driver — fails at import
$ python -m pytest tests/test_aps_artist_tool_e2e.py -q
ImportError: cannot import name 'render_module_list_thumb' from 'rust_engine_mcp.aps_catalog_preview'

# Full APS test collection — 9 modules error before a single test runs
$ python -m pytest tests/ -k aps -q
220 deselected, 9 errors in 0.67s
ERROR test_aps_artist_tool_e2e / test_aps_grammar_labels / test_aps_mat_bevy_witness /
      test_aps_ux_async_001 / test_aps_witness_refresh / test_build_worker_p0 /
      test_material_authority_bake / test_material_studio / test_pg_module_audit_002
```

The rot also bleeds into shared modules: `pg_module_audit_002.py` no longer exports `BATCH_ID`.

**Symbols `bind_aps_tooltip`, `JobController`, `ScrollableFrame`, `VariantsPanel`,
`GrammarInspectorPanel`, `render_module_list_thumb`, `render_material_preview` are defined
NOWHERE in the repo.** They were deleted, not relocated.

---

## Canonical artist path — per-step verdict

Walked against the code as it stands. Where the panel itself is intact I still rate the design,
but every step is gated behind "app does not launch."

| Step | Surface | Verdict | Note |
|:---|:---|:---:|:---|
| **0. Launch** | `app.py` | **BLOCK** | `from .aps_tooltips import bind_aps_tooltip` fails; tool never opens. |
| **1. Catalog validate** | `catalog.py` (intact, 333 ln) | **BLOCK** | Panel code present; list-thumb backend `aps_catalog_preview` is empty so thumbs are dead even if launched. |
| **2a. Assembly grammar generate** | `assembly_panel.py` (intact, 923 ln) | **BLOCK** | Strong design (seed, P0-gate-on-save, deterministic) — but imports empty `grammar_inspector` + `assembly_preview_panel`, so the module won't import. |
| **2b. Slot previews** | `slot_preview_panel.py` (intact) | **BLOCK** | Backend `aps_slot_preview.render_material_preview` is empty → no preview render. |
| **2c. Assign material** | `material_browser.py` → `_apply_material_profile` | **BLOCK** (design PASS) | The authority story is genuinely good (see below) — but unreachable. |
| **2d. Save snapshot → engine truth** | `assembly_panel.on_save` + `aps_mat_auth_ui` | **PASS (design)** | This is the best part of the tool and the backend module is intact. See "Metadata as truth." |
| **3. Materials browse/preview** | `materials_panel.py` (intact) | **BLOCK** | Depends on empty `aps_mat_002` / `aps_slot_preview`. |
| **4. Variants** | `variants_panel.py` | **BLOCK / MISSING** | Panel is **zero bytes**. The E2E "covers" this step only by checking an example JSON exists on disk — it never touches the panel. |
| **5. Atlas QC** | `atlas_panel.py` (intact) + `aps_atlas_qc.py` (intact, good) | **PASS (design)** | Plain-language PASS/FAIL, grid cell count, "v1 frozen" messaging — genuinely shippable QC copy. Unreachable while app won't launch. |

---

## Metadata-as-truth clarity (the one thing that is right)

When intact, the authority story is the tool's strongest asset and I want it preserved verbatim
on restore:

- `assembly_panel.py` shows a dedicated **"Material authority (APS-MAT-AUTH-UI-001)"** frame with
  the literal runtime read path: `placement.material_profile → material registry → worker bake /
  Bevy preview bind → render extract. Assembly snapshot is authority — not Catalog sidecar or
  Blender viewport.` (`aps_mat_auth_ui.ENGINE_READ_PATH`).
- `on_save` runs a **P0 gate before write** and surfaces `save_hint` ("N of M placements missing
  material_profile — assign before ship"). That is exactly the honest ship-gate an artist needs.
- `metadata_flow_panel.py` gives a per-tab "Metadata → engine (ARCH-MAT-001)" diagram that
  correctly states sidecar tags are hints and `semantic_tags` on the snapshot are ship truth.

**Verdict on authority legibility: PASS (when running).** This part did not regress in design; it
regressed only in that the app can't open to show it.

---

## QC sufficiency (atlas)

`aps_atlas_qc.py` is intact and good: `validate_atlas_folder` → `validate_atlas_meta_v2`,
plain-language sentences keyed by signature, `format_atlas_qc_display` prefixes PASS:/FAIL: (not
color-only) and appends `Grid C×R · N cells indexed · facings OK`. An artist could make an honest
"is this atlas registerable" call from this **if the app launched**. The E2E correctly treats the
pilot v1 folder as a **negative** fixture (`meta_v2_validate: false`) — that honesty is preserved.

**Gap that remains even after restore:** atlas QC validates *schema/lookup completeness*, not
*visual correctness of the keyframe stills*. An artist still cannot make a "does this tile look
shippable" call inside APS — that is by-design deferred to Track B keyframe + designer G4, and the
copy says so. Acceptable, but it means APS atlas QC is "schema ship-gate," not "art ship-gate."
Keep that boundary explicit in any future score >7.

---

## Production-rules audit (against what the code *would* do)

| Rule | Finding |
|:---|:---|
| **No AI-generated final art** | PASS — no affordance to drop an AI image as albedo; Materials tab is "drop authored PNGs → Reload preview" + procedural. |
| **Deterministic / seeded** | PASS — Assembly `Seed` spinbox feeds `generate_assembly_snapshot`; snapshot is the deterministic input to worker/runtime. |
| **Batch / atlas processing** | PASS — `tile_batch_v1` + `expand_variant_set_to_tile_batch` give a real batch path (no one-off tile escape hatch). |
| **Grid alignment** | n/a to exercise (footprint canvas + UV-grid unreachable). No affordance observed that bypasses grid. |

No rule-violating affordance found. The failure is integrity/build, not a rules breach.

---

## Re-verdict

| | Prior (2026-06-03) | Now (2026-06-15) |
|:---|:---|:---|
| **Verdict** | PASS WITH NOTES | **FAIL** |
| **Ship score** | 5 / 10 | **2 / 10** |
| **artist_would_ship_today** | false | **false (and tool will not open)** |

**What moved it (−3):** The 5/10 was "shippable for modules+materials+assembly, atlas QC must
target v2." That assessment assumed a launching app and reproducible witnesses. Since then, six UI
files and three backend modules were zeroed, the app no longer imports, the canonical E2E driver no
longer imports, and 9 APS test modules fail at collection — while the on-disk witnesses still claim
green. An artist cannot ship with a tool that does not start. I cannot, in good conscience, sign a
witness as green over this tree; doing so would be the dishonest-gate failure my contract forbids.

I am **not** crediting the +2 that the score keeps (vs 0) to luck — the architecture, the material
authority UX, and the atlas QC copy are genuinely well-designed and intact. The regression is a
file-wipe, not a design rot, so recovery should be fast (restore from `a982ff20` + re-author the
two never-populated stubs). Once it launches and the suite is green honestly, this returns toward
5–6 quickly.

---

## Prioritized fix list (route to @coder-mcp)

Ordered by what most blocks an artist shipping. Each names the surface.

1. **[P0 BLOCKER] Restore the 4 wiped UI files with prior content.** `scrollable.py`,
   `variants_panel.py`, `grammar_inspector.py`, `assembly_preview_panel.py` are zero bytes at HEAD
   but had real content at `a982ff20` (80/377/133/157 lines). `git checkout a982ff20 -- <files>` is
   the fast path; review the diff vs the "wild refactor" intent before committing.
   *Surface: `tools/mcp/art_pipeline_suite/`.*

2. **[P0 BLOCKER] Author the two never-populated UI stubs.** `aps_tooltips.py` (must export
   `bind_aps_tooltip`) and `job_controller.py` (must export `JobController`, `JobRecord`,
   `JobResult`, `JobState`, `JobWorker`, `DoneCallback`). These were already zero at `a982ff20`, so
   there is no clean source to restore from — they must be written. `app.py` and every panel import
   them. *Surface: `tools/mcp/art_pipeline_suite/aps_tooltips.py`, `job_controller.py`.*

3. **[P0 BLOCKER] Restore the 3 wiped backend render modules.** `aps_catalog_preview.py`
   (`render_module_list_thumb`), `aps_slot_preview.py` (`render_material_preview`), `aps_mat_002.py`.
   The E2E driver and the Catalog/Materials preview paths import these. They were zero across recent
   history, so search older history or re-implement against the importers' signatures.
   *Surface: `tools/mcp/python/rust_engine_mcp/`.*

4. **[P0] Fix shared-module breakage that errors APS tests.** `pg_module_audit_002.py` no longer
   exports `BATCH_ID`; this errors `test_pg_module_audit_002` and is part of the same wipe.
   *Surface: `tools/mcp/python/rust_engine_mcp/pg_module_audit_002.py`.*

5. **[P1 INTEGRITY] Make witnesses depend on build health.** Add an import/launch smoke
   (`python -c "from art_pipeline_suite import app"` + `pytest -k aps` collection) as a *precondition*
   of `run_artist_tool_e2e`, and refuse to write `green:true` if collection errors. Regenerate
   `aps_artist_tool_e2e_live.json` and `aps_artist_tool_modules_live.json` only after a real run.
   *Surface: `tools/mcp/python/rust_engine_mcp/aps_artist_tool_e2e.py`, `aps_witness_refresh.py`.*

6. **[P1 WORKFLOW] E2E must exercise the Variants panel, not just an example JSON.** The current
   `variants_example` step (`aps_artist_tool_e2e.py:76`) only checks a file exists on disk — it gives
   false coverage to a step whose panel is empty. After restore, drive `VariantsPanel.on_new_from_assembly`
   in the witness path. *Surface: `tools/mcp/python/rust_engine_mcp/aps_artist_tool_e2e.py`.*

7. **[P2 CARRYOVER] Re-confirm the still-open notes from the 5/10 audit once green:**
   APS-UX-GRAMMAR-WHY human labels (depends on restored `grammar_inspector.py`), status-text-beside-glyphs,
   catalog list thumb + sidecar-authority line. These were the path from 5→7 and remain valid.
   *Surface: `grammar_inspector.py`, `pipeline_status_bar.py`, `catalog.py`.*

---

## Definition-of-done check (this review)

- [x] order-critique + rules-audit emitted
- [x] canonical artist path walked, per-step verdict
- [x] metadata-as-truth + QC sufficiency assessed
- [x] production-rules audit
- [x] re-verdict with score + delta + what moved it
- [x] prioritized fix list routed to @coder-mcp with surfaces
- [x] no production code edited (review + doc only)

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **FAIL — tool does not launch; witnesses dishonest vs tree** | 2026-06-15 |

```text
APS-ARTIST-SHIP-REVIEW-20260615 complete
Verdict: FAIL · score 2/10 (−3 vs 5/10)
Blocker: 6 zero-byte UI files + 3 zero-byte backend modules → app + E2E will not import
Honest-gate: on-disk green witnesses do NOT reflect HEAD — regenerate after restore
Re-review after P0 1–4 land and pytest -k aps collects clean
```
