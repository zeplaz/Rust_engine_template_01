# PLAN-APS-EVOLUTION-VEG-CAPABILITY-001 — APS capability evolution: building → full art program (buildings + vegetation) `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-APS-EVOLUTION-VEG-CAPABILITY-001** |
| **Track** | **art_A / art_C** (APS product + landscape art lane) — MCP three-track |
| **Author** | `@planner-mcp` (architecture / phases / schema-ownership) |
| **Date** | 2026-06-16 |
| **Status** | **SIGNED** (@planner-mcp 2026-06-16) — architecture + wave-0 schemas; E0–E5 impl open |
| **Stance** | READ-ONLY planning deliverable. No production code. Schema sketches + job-JSON examples + phase tables ONLY. |
| **Parent** | `$ref:src/dev/plan_aps_artist_tool_exec_v1.md` · `$ref:src/dev/plan_landscape_grammar_exec_001_v1.md`[§7] · `$ref:src/dev/plan_veg_burn_extract_001_v1.md` |
| **Reconciliation truth** | `$ref:src/dev/veg_queue_reconciliation_20260616_v1.md` |
| **Charter (art)** | `$ref:src/dev/design_landscape_lg5_atlas_v1.md` |
| **Designer parallel** | `$ref:src/dev/design_aps_artist_ship_review_20260615_v1.md` (UX/IA owned by `@designer` — no pixel-level UX here) |
| **Three-track citations** | `debug_runs/unified_witness_index.json` · `debug_runs/agent_ops/ops_report_latest.json` |
| **Contract** | `$ref:tools/orchestrator/queues/OPS_WITNESS_SPINE.md` |
| **Skills** | `tile-generation` (tiles = deterministic state machines) · `mcp-asset-pipeline` (G0–G5) · `mcp-production-rules` |

---

## Summary

APS today is a **5-tab building-asset tool** (Catalog → Assembly → Materials → Variants → Atlas) whose data spine is hardwired to the **buildings** domain: Catalog reads `list_modules` (building `_module_index`), Variants exposes building state axes (damage/power/fill/lighting), and the Atlas tab has **no `atlas_domain` awareness** — it registers into `_tile_atlas_index` (buildings). The vegetation/landscape art program has a **separate, already-shipped backend index lane** (`landscape_atlas_index.py` → `_landscape_atlas_index.ron/json`) but **no GUI surface in APS at all**. This plan evolves APS from building-centric to **full-art-program** via a **domain-router architecture** (buildings | landscape) layered onto the existing tabs, a landscape-preset browse/author lane, an **LG-5 atlas expansion workflow** (3 pilot tiles → full topology/state set), **burn/scar/recovery state authoring modeled as tile STATE** (mirroring building `burning_00..07`, per `tile-generation`), and **veg variant + atlas QC without Blender**. It keeps artist-can-ship-without-Blender and the single-execution-path rule (APS GUI and `python -m rust_engine_mcp.cli` call the same functions). It reconciles four neighboring plans against the witnesses and flags where each is stale.

---

## Order critique — what I questioned, what is incomplete in the brief

```yaml
order_critique:
  request_summary: "Phased APS capability-evolution plan: grow the artist tool from building-centric to full art program (buildings + vegetation/landscape), integrate with the other program plans, keep ship-without-Blender."
  questioned:
    - "New tool or extension? VERDICT: EXTENSION of shipped APS + shipped rust_engine_mcp spine. A landscape art lane already exists in the BACKEND (landscape_atlas_index.py is SHIPPED); missing is a GUI surface + a domain router. Do NOT plan a second tool/server (single rust-engine-art MCP until Phase 4 of the master spine)."
    - "Survives batch-scale? burn/scar/recovery is a STATE-MACHINE expansion (per tile-generation: atlas budget for ALL states up-front). A 3-tile pilot atlas does not. Plan must size the full state×topology matrix BEFORE bake, or every added burn row re-bakes the sheet."
    - "All four production-rules structurally enforceable for veg? Yes — each (no-AI-art, deterministic/seeded, batch/atlas, grid-alignment) routes through the SAME validators the building lane uses (tile_batch_validate, tile_promotion_honest, atlas_meta_brief). No veg-specific shortcut."
    - "Minimum-correct phase set, not fastest demo? Rejected a 'just add a Landscape tab that lists presets' shortcut — browse without authoring/state/QC/parity is a demo. Minimum-correct: domain router -> preset browse/author -> state-axis authoring -> atlas expansion+QC -> extract/consumer parity."
  incomplete_in_brief:
    - "Brief says APS was 'recovered from a file-wipe.' VERIFIED at HEAD: variants_panel.py (429 ln), grammar_inspector.py (133), assembly_preview_panel.py (196), aps_tooltips.py (150), job_controller.py (120), scrollable.py (80), and backend aps_catalog_preview/aps_slot_preview/aps_mat_002 are all NON-zero; test_aps_imports.py exists. So the design_aps_artist_ship_review FAIL (2/10, app-won't-launch) is STALE vs HEAD. Plan treats APS as launching — but Phase E0 re-runs the E2E witness to PROVE it (the green E2E predates the wipe/recover cycle)."
    - "Brief asks for a 'burn/scar veg atlas' — but per plan_veg_burn_extract §6, the variant_key catalog (veg_burn_00..07) and _vegetation_variant_catalog.ron do NOT YET EXIST on disk. That schema is the planner-mcp/coder-mcp authoring gap gating veg-state atlas authoring. Surfaced as top-3 gap + Open Question."
  rules_audit:
    no_ai_generated_images: enforceable   # veg atlas uses keyframe_pack bake_source (charter design_landscape_lg5_atlas §2); ortho = smoke/CI only
    deterministic_output: enforceable      # render.seed on tile_batch (550005 pilot); succession frame_index = f(tick,seed)
    batch_processing: enforceable          # tile_batch_v1 + variant_set; veg atlas is a batch, not one-off
    grid_alignment: enforceable            # atlas_meta_v2 UV grid + atlas_meta_brief QC, same as buildings
  proceed: yes_with_open_questions
```

---

## Current state — verified against HEAD + witnesses (SHIPPED / PLANNED / DEFER)

### APS tool capability (buildings)

| Capability | Surface | Label | Evidence |
|:---|:---|:---:|:---|
| 5-tab shell | `tools/mcp/art_pipeline_suite/app.py` | **SHIPPED** | `_build_tabs()` adds 5 scrollable tabs |
| Catalog module browse | `catalog.py` -> `module_viewer.list_modules` | **SHIPPED (buildings-only)** | `list_modules(batch_id, category)` — building `_module_index` only |
| Assembly grammar gen + slot previews + material assign + P0-gate-on-save | `assembly_panel.py` | **SHIPPED** | designer review: "best part of the tool" |
| Materials studio tree + preview modes | `materials_panel.py` | **SHIPPED** | nested category tree (APS-MAT-003) |
| Variants authoring (lighting/power/damage/fill axes) | `variants_panel.py` (429 ln) | **SHIPPED (building state axes)** | building state machine only — no topology/burn axis |
| Atlas pack + UV-grid + plain-language QC | `atlas_panel.py` + `aps_atlas_qc.py` | **SHIPPED** | `format_atlas_qc_display`; **no `atlas_domain` awareness** |
| Inline feedback (modal->inline) + 7 UX fixes + import-guard | `aps_inline_feedback.py`, `tests/test_aps_imports.py` | **SHIPPED** | recovery verified at HEAD |
| Same-code-path CLI/MCP parity | `python -m rust_engine_mcp.cli` == MCP | **SHIPPED** | flow bar: "All actions call rust_engine_mcp CLI/MCP" |

### Vegetation/landscape art (the gap)

| Capability | Surface | Label | Evidence |
|:---|:---|:---:|:---|
| Landscape preset browse/author in APS | — | **PLANNED (absent)** | state.py has no landscape/preset/topology; no Landscape tab |
| Landscape grammar validator | `validate-report landscape_grammar` | **SHIPPED · SIGNED** | MICRO_TOOLS Tier 1f; 10/10 ship presets pass |
| Landscape preset catalog (10 ship + 30 topology) | `_preset_index.json` + `presets/*.json` | **SHIPPED** | 10 JSON on disk; VEG-PRESET-CATALOG-001 DONE-CONFIRMED |
| Landscape atlas index lane (separate from buildings) | `landscape_atlas_index.py` -> `_landscape_atlas_index.ron/json` | **SHIPPED (backend, no GUI)** | `LANDSCAPE_ATLAS_INDEX_RON`; 1 pilot entry |
| LG-5 pilot atlas (3 tiles: patch/corridor/ring) | `tile_batch_landscape_lg5_pilot_v1.json` | **SHIPPED (pilot, ship:false)** | png_count=3, development_tier:pilot, G4/G5:planned |
| Burn/scar/recovery veg atlas (state-machine) | — | **PLANNED (absent)** | pilot variants all `state:clean`; no burn/scar/recovery rows |
| `_vegetation_variant_catalog.ron` (veg_burn_00..07 keys) | `assets/configs/landscape/_vegetation_variant_catalog.ron` | **PLANNED (absent on disk)** | does not exist; gates veg-state authoring |
| Veg atlas QC in-tool (no Blender) | — | **PLANNED (absent)** | atlas QC exists but domain-unaware |
| Engine extract lane (grammar->variant->map-stamp) | `landscape_grammar_extract_live.json` (sim) | **SHIPPED (sim/tint scope) · art-stamp PLANNED** | extract_glyph_deterministic=true; real-sprite variant_key resolution pending LG-5 art-ship |
| LG-4 pixel-heterogeneity proof | `landscape_grammar_lg4_preview_live.json` | **PLANNED (residual gap)** | on disk: `pixel_heterogeneity_wired:false`, `topology_tint_visible_chunks:0` |
| LG-6 flowers aesthetic | — | **DEFER** | charter LG-6; VEG-G03-LG6-FLOWERS-001 correctly deferred |

### Three greens — operator vocabulary (per reconciliation §G4/G5)

"green" is overloaded for the LG-5 atlas. Exit witnesses MUST preserve scope per green:

| Green scope | Witness | Certifies | NOT |
|:---|:---|:---|:---|
| **SCHEMA/SIGN** | `mcp_landscape_grammar_sign_live.json` | 10/10 presets pass schema validate | art, runtime |
| **BAKE (G0–G3)** | `tile_tile_landscape_lg5_pilot_v1_live.json` | PNGs baked + atlas meta + index row (3 tiles) | G4 art-ship, G5 |
| **RUNTIME-STAMP** | `landscape_grammar_lg5_live.json` | registry_stamp + bevy_chunk_uv_stamp wired | art-ship; still pilot, ship:false |

**Art-ship green (G4/G5) is NOT achieved.**

---

## Capability gaps to "handle vegetation" — concretely

| # | Gap | What the tool/backend needs | Owner | Label |
|:---|:---|:---|:---|:---:|
| **G-A** | **No domain router** — every tab assumes `buildings` | `SuiteState.art_domain` selector + per-tab data-source switch (Catalog->presets, Variants->state axes, Atlas->`atlas_domain` register target). Architectural, not cosmetic. | `@coder-mcp` (impl) · `@designer` (IA) | **PLANNED** |
| **G-B** | **No landscape-preset browse/author** | Catalog (or Landscape tab) lists `_preset_index.json` (10 ship + 30 topology), shows topology-graph summary, runs `validate-report landscape_grammar` inline, opens preset to author land_dna/pressure/topology_graph | `@coder-mcp` (browse) · `@designer-mcp` (author criteria) | **PLANNED** |
| **G-C** | **No burn/scar/recovery STATE authoring** | Variants tab landscape state axis (succession_stage gap/regen/shrub/sapling/canopy + ActiveBurn veg_burn_00..07 + RegrowthMacroPhase) modeled as tile STATE per `tile-generation`, expanding tile_batch variants beyond `state:clean`. Requires `_vegetation_variant_catalog.ron` schema FIRST. | `@designer-mcp` (state content) · `@planner-mcp` (catalog schema) · `@coder-mcp` (axis UI) | **PLANNED** |
| **G-D** | **No LG-5 atlas EXPANSION workflow** | Atlas tab batch path accepts a full topology×state matrix (not 3 tiles), atlas-budget sizing up-front (state-machine rule), `atlas_domain: landscape` register to `_landscape_atlas_index`, domain-aware QC | `@coder-mcp` (impl) · `@designer-mcp` (matrix sign) | **PLANNED** |
| **G-E** | **No engine extract/consumer parity surface in-tool** | Read-only "Engine reads: variant_key -> resolver -> map stamp" callout + a QC that authored veg `variant_key`s match `VegetationExtractFrame`/resolver expectations (parity), so authored veg appears in game | `@coder-mcp` (parity check) · `@coder` (resolver authority) | **PLANNED** |

### Top 3 capability gaps that block "veg in the artist tool"

1. **G-A domain router** — without it the tool is structurally buildings-only; nothing else lands cleanly.
2. **G-C `_vegetation_variant_catalog.ron` + state-axis** — burn/scar/recovery cannot be authored as STATE until the catalog schema exists (it does not on disk) and Variants exposes a landscape state axis.
3. **G-D atlas-budget sizing + domain-aware register** — a 3-tile pilot cannot grow to a full burn/scar atlas without up-front budget (state-machine rule) and `atlas_domain` routing to the landscape index.

---

## Target architecture

```text
                 +---------------- APS shell (app.py) ----------------+
                 |  art_domain in {buildings | landscape}  <- G-A router|
                 +------+--------------------------------+-------------+
        buildings domain |                                | landscape domain
   +------------------+---v----------+         +----------v-----------------------+
   | Catalog  list_modules           |         | Catalog  _preset_index.json       |  <- G-B
   | Assembly grammar (building_dna)  |         | Assembly landscape_grammar_v0     |
   | Variants damage/power/fill/light |         | Variants succession+burn STATE    |  <- G-C
   | Atlas    -> _tile_atlas_index    |         | Atlas    -> _landscape_atlas_index|  <- G-D
   +------------------+---------------+         +----------+-----------------------+
                      |   single execution path (rust_engine_mcp)   |
                      +-----------------------+----------------------+
                                              v
              tile_batch_run · variant_set · tile_atlas_pack · validate-report
                                              v
        registry stamp --> VegetationExtractFrame / resolver --> map stamp  <- G-E parity
```

| Concern | Owner | Boundary |
|:---|:---|:---|
| **Tool-categories** | geometry · tile · prop · material · validation · library · reference — landscape lands in **tile** (iso state-machine atlas), browse in **library/reference** | — |
| **Schema-ownership** | `landscape_grammar_v0.schema.json` (SHIPPED), `tile_batch_v1.schema.json` (SHIPPED, atlas_domain field), `_vegetation_variant_catalog.ron` schema (**PLANNED** `@planner-mcp` authors, `@coder-mcp` validates) | `@designer-mcp` owns CONTENT of preset + variant specs; `@planner-mcp` owns schema shape |
| **Adapter-boundaries** | APS panels call `rust_engine_mcp` functions; `landscape_atlas_index.py` is the landscape register adapter (parallel to `tile_index.py`); same `tile_pipeline` module for manual/CLI/MCP | no forked behavior — CLI-proven == MCP |
| **Registry-contract** | buildings -> `_tile_atlas_index.ron` (ship_allowed); landscape -> `_landscape_atlas_index.ron` (ship_allowed:false at pilot); both feed Bevy via UV-grid stamp; `BuildingDefinition`/`StylePack`/tile-atlas compat preserved | crossover to `RepresentationResult`/asset-registry/tile ECS -> `@planner` engine-authority review |

---

## Implementation phases (gated G0–G5)

> **Principle preserved:** every phase keeps artist-can-ship-without-Blender — authoring is JSON/spec + keyframe_pack bake; no phase requires opening Blender for everyday work. Bake stays a separate worker step.

### Phase E0 — APS launch + E2E re-witness (prove recovery)

| Field | Value |
|:---|:---|
| **ID** | `APS-EVO-E0-RELAUNCH-001` |
| **Goal** | Prove APS launches and the canonical E2E witness reflects HEAD (prior green predates wipe/recover) |
| **Files/paths** | `tools/mcp/art_pipeline_suite/*` (read-only verify) · `debug_runs/aps_artist_tool_e2e_live.json` · `tests/test_aps_imports.py` |
| **Authority-owner** | `@coder-mcp` (run) · `@designer-mcp` (artist-acceptance re-verdict) |
| **Rule-enforcement** | import/launch smoke as PRECONDITION of E2E witness (designer fix #5); refuse green:true if collection errors |
| **Diagnostics/witnesses** | `aps_artist_tool_e2e_live.json` (regenerated honestly), `aps_artist_tool_modules_live.json` |
| **Acceptance** | `pytest -k aps` collects clean · `python -c "from art_pipeline_suite import app"` succeeds · E2E green AFTER real run |
| **Rollback-trigger** | any zero-byte panel/backend module reappears -> re-run designer ship-review FAIL path |
| **Label** | **SHIPPED-VERIFY** (code exists; this re-asserts) |

### Phase E1 — Domain router (G-A)

| Field | Value |
|:---|:---|
| **ID** | `APS-EVO-E1-DOMAIN-ROUTER-001` |
| **Goal** | `SuiteState.art_domain` + per-tab data-source switch (buildings | landscape) — no new tab yet, route existing tabs |
| **Files/paths** | `state.py` (add `art_domain`) · `app.py` (domain selector) · per-panel `set_domain()` hooks |
| **Authority-owner** | `@coder-mcp` (impl) · `@designer` (IA: selector placement — see designer parallel plan) |
| **Rule-enforcement** | domain selection must NOT fork code paths — both domains call the same `tile_pipeline`/`validate-report` functions |
| **Diagnostics/witnesses** | `debug_runs/aps_domain_router_live.json` (domain switch toggles Catalog source) |
| **Acceptance** | `pytest tools/mcp/python/tests/test_aps_domain_router.py` · switching domain re-sources Catalog with no crash |
| **Rollback-trigger** | landscape domain breaks building tabs -> feature-flag off, buildings default |
| **Label** | **PLANNED** |

### Phase E2 — Landscape preset browse + inline validate (G-B)

| Field | Value |
|:---|:---|
| **ID** | `APS-EVO-E2-PRESET-BROWSE-001` |
| **Goal** | Landscape-domain Catalog lists `_preset_index.json` (10 ship + 30 topology), shows topology-graph summary, runs `validate-report landscape_grammar` inline (plain-language PASS/FAIL) |
| **Files/paths** | `catalog.py` (landscape branch) · backend `landscape_preset_browse.py` (NEW, `@coder-mcp`) wrapping SHIPPED `validate-report landscape_grammar` |
| **Authority-owner** | `@coder-mcp` (browse impl) · `@designer-mcp` (QC criteria an artist needs to read a preset) |
| **Rule-enforcement** | browse is read-only; author/edit defers to E3; uses SHIPPED validator (no new validation logic) |
| **Diagnostics/witnesses** | `debug_runs/aps_landscape_preset_browse_live.json` (10/10 listed + validate green inline) |
| **Acceptance** | select `fire_recovery_v0` -> topology summary (Network/Patch/Corridor/Cluster/Fringe) + validate PASS without JSON dump |
| **Rollback-trigger** | preset index drift (count != index) -> surface FAIL, do not silently list |
| **Label** | **PLANNED** |

### Phase E3 — Veg state-machine catalog schema + Variants state axis (G-C)

| Field | Value |
|:---|:---|
| **ID** | `APS-EVO-E3-VEG-STATE-AXIS-001` (+ schema sub-row `APS-EVO-E3-VEGCATALOG-SCHEMA-001`) |
| **Goal** | Author `_vegetation_variant_catalog.ron` schema (planner-mcp), then Variants landscape state axis: succession_stage + ActiveBurn frames (veg_burn_00..07) + RegrowthMacroPhase — modeled as tile STATE per `tile-generation` |
| **Files/paths** | schema `tools/mcp/schemas/vegetation_variant_catalog_v1.schema.json` (NEW) · catalog `assets/configs/landscape/_vegetation_variant_catalog.ron` (NEW, designer-mcp content) · `variants_panel.py` (landscape state branch) |
| **Authority-owner** | `@planner-mcp` (schema shape) · `@designer-mcp` (variant_key set + state content) · `@coder-mcp` (axis UI + validator) |
| **Rule-enforcement** | **state-machine atlas-budget rule** — variant_key set fixed up-front (per `plan_veg_burn_extract §6`); deterministic frame_index = f(tick, seed); validator rejects unseeded variation |
| **Diagnostics/witnesses** | `debug_runs/aps_veg_state_axis_live.json` · validator green for `_vegetation_variant_catalog.ron` |
| **Acceptance** | catalog validator passes · Variants emits a tile_batch with burn/scar/recovery variants (not just clean) |
| **Rollback-trigger** | catalog schema disagrees with resolver expectations (E5 parity) -> freeze schema, reconcile with `@coder` resolver authority |
| **Label** | **PLANNED** (catalog absent on disk — top-3 blocker) |

### Phase E4 — LG-5 atlas EXPANSION + domain-aware QC (G-D)

| Field | Value |
|:---|:---|
| **ID** | `APS-EVO-E4-ATLAS-EXPAND-001` |
| **Goal** | Atlas tab batch path accepts full topology×state matrix (beyond 3 tiles), sizes atlas budget up-front, registers via `atlas_domain: landscape` -> `_landscape_atlas_index`, runs domain-aware QC |
| **Files/paths** | `atlas_panel.py` (atlas_domain branch) · `landscape_atlas_index.py` (SHIPPED — wire register call) · `aps_atlas_qc.py` (domain param) · example `tile_batch_landscape_*_v1.json` (expanded, designer-mcp) |
| **Authority-owner** | `@designer-mcp` (full matrix sign + atlas budget) · `@coder-mcp` (register + QC wiring) |
| **Rule-enforcement** | `bake_source: keyframe_pack` (no ortho ship); `tile_promotion_honest` rejects dry-run/ortho ship; grid-alignment via `atlas_meta_v2`; **plan all state rows before bake** (adding a burn row later re-bakes the sheet) |
| **Diagnostics/witnesses** | `debug_runs/art_pipeline/tile_landscape_<expanded>_live.json` (png_count > 3, atlas_domain landscape) · `aps_atlas_qc` domain-aware PASS |
| **Acceptance** | expanded batch bakes (keyframe_pack) · row in `_landscape_atlas_index.ron` · QC reports grid C×R + cells indexed for landscape domain |
| **Rollback-trigger** | atlas exceeds vertex/texture budget OR mid-batch validate FAIL -> keep pilot atlas, do not register partial |
| **Label** | **PLANNED** |

### Phase E5 — Extract/consumer parity surface (G-E)

| Field | Value |
|:---|:---|
| **ID** | `APS-EVO-E5-EXTRACT-PARITY-001` |
| **Goal** | Read-only "Engine reads: variant_key -> resolver -> map stamp" callout + parity QC that authored veg variant_keys match `VegetationExtractFrame`/resolver expectations — so authored veg appears in game |
| **Files/paths** | APS read-only panel (mirror `aps_mat_auth_ui` ENGINE_READ_PATH pattern) · parity backend `aps_veg_extract_parity.py` (NEW) reading `landscape_grammar_extract_live.json` |
| **Authority-owner** | `@coder-mcp` (parity check in-tool) · `@coder` (resolver/extract authority — `RepresentationResult`/tile ECS) -> `@planner` engine-authority review |
| **Rule-enforcement** | parity is read-only diagnostic; APS must NOT write ActiveBurn/extract (single-writer rule from `plan_veg_burn_extract §1`) |
| **Diagnostics/witnesses** | `debug_runs/aps_veg_extract_parity_live.json` (authored variant_keys subset-of resolver-known keys) |
| **Acceptance** | parity green when veg catalog keys resolve · artist can answer "will this burn tile show in game?" from UI |
| **Rollback-trigger** | parity FAIL (authored key not consumable) -> block atlas promote, route to `@coder` resolver |
| **Label** | **PLANNED** (extract sim-green; art-stamp PLANNED) |

---

## Schema plan — new/changed JSON schemas + version ids

| Schema | Status | Owner | Note |
|:---|:---:|:---|:---|
| `landscape_grammar_v0.schema.json` | **SHIPPED** | `@planner-mcp` | preset authority; no change |
| `tile_batch_v1.schema.json` | **SHIPPED** | `@planner-mcp` | already has `atlas_domain` (landscape used in pilot) — no version bump; veg state variants use existing `variants[]` shape |
| `vegetation_variant_catalog_v1.schema.json` | **PLANNED (NEW)** | `@planner-mcp` authors · `@coder-mcp` validates | veg_clean_day / veg_damaged / veg_burn_00..07 / veg_regrowth_* / veg_old_growth keys + condition mapping (sketch `plan_veg_burn_extract §6`) |
| `_vegetation_variant_catalog.ron` (instance) | **PLANNED (NEW)** | `@designer-mcp` content | the ship/pilot catalog; gates E3/E4 |
| `_landscape_atlas_index.ron/json` | **SHIPPED** | `@coder-mcp` | register target; gains expanded rows in E4 (no schema change, more entries) |
| `variant_graph_v1.schema.json` | **SHIPPED (ARCH-002)** | `@planner-mcp` | reuse pattern for veg per-node state patches if needed |

**No new schema VERSION bump for tile_batch** — landscape is an existing `atlas_domain` value; the only genuinely new schema is the vegetation variant catalog.

---

## Gate alignment (phases -> @orchestrator-mcp gates G0–G5)

Using `mcp-asset-pipeline` G0–G5 (`<G:art> = G0 ; G1 ; G2 ; G3 ; G4 ; G5`):

| Gate | Meaning | Phase coverage |
|:---|:---|:---|
| **G0** order-critique + rules-audit (`@designer-mcp`) | this plan's order-critique + E0 designer re-verdict | E0; per-phase G0 for E2–E5 |
| **G1** spec valid (`validate-report mcp_spec` / `landscape_grammar` / veg-catalog) | E2 (preset validate), E3 (veg catalog schema validate) |
| **G2** tool runs (`tile-spine-run` / `tile-batch-run`) | E4 (expanded atlas bake) |
| **G3** validate green (`asset_glb` / `tile_promotion_honest` / `atlas_meta_brief`) | E4 (domain-aware QC), E5 (parity) |
| **G4** staging sign-off (`@designer-mcp`, `list-staging`) | E4 atlas matrix sign — **the art-ship gate NOT yet achieved for LG-5** |
| **G5** promote + register + witness | E4 register to `_landscape_atlas_index`; E5 consumer-parity confirms map stamp |

**Critical:** reconciliation proves LG-5 sits at **bake-green (G3) + runtime-stamp**, NOT **art-ship-green (G4/G5)**. E4 earns G4 for an EXPANDED veg atlas; the existing pilot stays `ship:false` until then.

---

## Integration with the OTHER plans (the "improve all other plans" ask)

| Plan | Current claim | Witness reality | Misalignment / action |
|:---|:---|:---|:---|
| **`plan_aps_artist_tool_exec_v1.md`** (building-centric, Phases 0–9) | building-only roadmap; Phase 9 E2E green | E2E witness predates wipe/recover; **no vegetation lane** | **STALE in scope, not content.** This plan is the vegetation superset — add a cross-ref; do NOT rewrite its building phases. Its Phase 9 E2E must be re-witnessed (our E0). |
| **`plan_veg_burn_extract_001_v1.md`** (SIGNED) | LG-5 catalog `veg_burn_00..07` (PT-4 pattern); `_vegetation_variant_catalog.ron` after sign | sign SHIPPED; **catalog .ron does NOT exist on disk** | **ALIGNED but BLOCKED.** Our E3 executes its §6 + §9 hooks. Flag to `@orchestrator-mcp`: the `_vegetation_variant_catalog.ron` schema row (planner-mcp) is the gating dependency for veg-state atlas in APS. |
| **`plan_landscape_grammar_exec_001_v1.md`** §7 (LG-5/LG-6) | "LG-5 minimal iso atlas after LG-4 green" | LG-4 on disk: `pixel_heterogeneity_wired:false`, `topology_tint_visible_chunks:0` — **LG-4 NOT fully green** | **MISALIGNED vs §7 precondition.** §7 gate ("LG-4 green before LG-5") is technically violated: the 3-tile pilot baked while LG-4 pixel-heterogeneity is pending. Action: annotate the pilot as a teach exception (`not_a_ship_target:true`); E4 EXPANSION must NOT reach G4 until LG-4 pixel proof lands (reconciliation §8 row 8). |
| **`design_aps_artist_ship_review_20260615_v1.md`** (FAIL 2/10) | app won't launch; 6 zero-byte UI + 3 zero-byte backend | **STALE vs HEAD** — all files non-zero; import-guard test exists | **STALE-PESSIMISTIC.** E0 re-runs E2E to supersede the FAIL honestly; designer re-verdict expected to return toward 5–6 ("recovery should be fast" per the review). |
| **Reconciled queue (`veg_queue_reconciliation_20260616_v1.md`)** | 35/41 DONE-CONFIRMED; VEG-LG2-HARVEST GENUINELY-OPEN; nested_depth/recovery AMBIGUOUS; VEG-F01 art-ship OPEN | authoritative | **ALIGNED.** E3/E4 close the **art** side of Phase F (VEG-F01 art-ship). SIM residuals (harvest=0, nested_depth=2<3, recovery_ticks=0) are NOT APS concerns — route to `@coder` veg sim lane; do not absorb into APS phases. |
| **Building KIT002 ship-closure + consumer parity** | building atlas -> `_tile_atlas_index` (ship_allowed) + RT-REG/runtime-lookup briefs | RT-REG-001 + runtime_lookup_brief SHIPPED (Tier 1d) | **ALIGNED.** The landscape lane MIRRORS this exact consumer-parity pattern (E5) but registers to `_landscape_atlas_index`. KIT002 building closure is the template; do not fork the parity tooling. |

### Where an existing plan is stale vs witnesses (explicit)

1. **`plan_landscape_grammar_exec_001` §7** says "LG-5 after LG-4 green" — but the pilot atlas baked while `pixel_heterogeneity_wired:false`. Annotate §7: pilot allowed as teach exception; **expansion gated on LG-4 pixel proof**.
2. **`design_aps_artist_ship_review_20260615`** FAIL verdict is stale (files recovered) — must be superseded by a fresh honest E2E (E0).
3. **`plan_aps_artist_tool_exec_v1`** has no veg lane — scope-stale; this plan supersedes its scope without rewriting its building content.

---

## Scalability — "make sure the tool can handle this"

Adding a vegetation lane to a 5-tab building tool creates real **architectural pressure**; the plan addresses it structurally, not by piling tabs:

| Pressure | Risk if ignored | This plan's structural answer |
|:---|:---|:---|
| **Tab×domain explosion** | 5 tabs × 2 domains = 10 logical surfaces; naive approach adds 5 new tabs | **Domain router (E1)** routes EXISTING tabs by `art_domain`; tab COUNT stays 5; data source switches. UI/IA call (toggle vs tab) is `@designer`'s. |
| **State-space growth** | building states (damage/power/fill/lighting) vs veg states (succession×burn×regrowth) co-mingle | Variants tab branches on domain (E3); veg state axis is a separate code path that emits the SAME `tile_batch_v1` shape — no schema fork. |
| **Atlas budget** | 3-tile pilot doesn't scale to topology×state matrix; late additions re-bake | Atlas-budget sizing up-front (E3 catalog fixes the variant_key set; E4 sizes the matrix before bake) — the `tile-generation` state-machine rule. |
| **Two register targets** | buildings (`_tile_atlas_index`) vs landscape (`_landscape_atlas_index`) | already separated in backend (`landscape_atlas_index.py` SHIPPED); E4 wires the GUI to the correct target by `atlas_domain`. |
| **UX/IA specifics** | how the artist navigates domains, where the selector lives, veg-tab affordances | **Handed to the parallel `@designer` plan** (`design_aps_*`). This plan defines the architecture + data contracts; it does NOT specify pixel-level UX. |

---

## Edge cases

| Edge case | Handling |
|:---|:---|
| **Blender absent on CI** | Authoring (preset browse, state-axis spec, QC schema check) is Blender-free. Bake (E4) is the only Blender step (keyframe_pack worker) — gate E4 behind `pipeline_preflight.blender_ok`; CI smoke uses ortho dry-run (never ship). |
| **Large-batch timeout** (full topology×state atlas) | atlas-budget sizing up-front; chunk the bake by topology kind; `tile-spine-run` per-batch witness; no register until all cells present (no partial atlas). |
| **Failed validation mid-batch** | `tile_promotion_honest` / `atlas_meta_brief` FAIL -> keep pilot atlas registered, do not overwrite; plain-language FAIL in QC; rollback per E4. |
| **Partial atlas rebuild** | adding a burn/recovery row after ship re-bakes the sheet. Plan all state rows in E3 BEFORE E4 bake; a later row = new atlas version (`_v2`), not in-place mutation. |
| **Domain-router state leak** | switching domain mid-authoring must clear domain-specific state (assembly_id vs preset_id) — E1 acceptance covers no cross-domain leakage. |
| **Stale green witness over changed tree** | E0 import/launch smoke as precondition (designer fix #5); witness-honesty gate (`validate-report witness_honesty`) before any Q-check. |

---

## Risk register

| Risk | Likelihood | Mitigation |
|:---|:---:|:---|
| `_vegetation_variant_catalog.ron` schema churns against resolver expectations | High | E3 freezes schema with `@coder` resolver review (E5 parity); single-writer rule keeps APS read-only on extract |
| Domain router bloats a 5-tab tool into an unmaintainable matrix | Med | route existing tabs by domain (not 5 new tabs); hand UI/IA to `@designer` parallel plan |
| LG-5 expansion proceeds before LG-4 pixel proof | Med | E4 G4 gated on LG-4 `pixel_heterogeneity_wired:true`; pilot stays `ship:false` |
| Overloaded "green" hides bake-vs-art-ship scope | High | every exit witness labels scope (schema / bake / runtime-stamp / art-ship) per reconciliation §G4/G5 |
| Building lane regresses when landscape lands | Med | feature-flag landscape domain; buildings default; E1 rollback-trigger |
| Designer ship-review FAIL treated as current | Low | E0 supersedes with fresh honest E2E |

---

## Definition of Done (this planning deliverable)

- [x] order-critique + rules-audit included
- [x] SHIPPED / PLANNED / DEFER labels on every capability + phase
- [x] all four production-rules addressable in design (no veg shortcut)
- [x] gates G0–G5 mapped to phases
- [x] integration-with-other-plans reconciles 6 plans vs witnesses; 3 stale flags explicit
- [x] open questions explicit (below); no shortcut phase without documented tradeoff
- [x] no production code edited

---

## Open questions (route to @designer-mcp / @orchestrator-mcp / @planner)

1. **[@planner-mcp + @coder]** `_vegetation_variant_catalog.ron` schema shape — reuse `variant_graph_v1` per-node patches, or a flat variant_key->condition map (`plan_veg_burn_extract §6`)? Resolver authority (`@coder`) must confirm key naming before E3 freeze.
2. **[@designer]** Domain-selection IA — top-level toggle vs a 6th "Landscape" tab vs domain-aware existing tabs? Owned by the parallel designer plan; this plan assumes domain-aware existing tabs but defers the call.
3. **[@orchestrator-mcp]** Sequencing — does E3 (veg catalog schema) block on closing SIM residuals (VEG-LG2-HARVEST, nested_depth>=3, recovery_ticks>=1), or proceed in parallel? Recommendation: parallel — art STATE keys are independent of sim event counters.
4. **[@designer-mcp]** Full topology×state matrix size — how many of the 20 topology kinds × how many states get atlas cells in the FIRST expansion (atlas budget)? Needs a sign before E4 bake.
5. **[@planner engine-authority]** E5 parity touches `RepresentationResult`/resolver/tile ECS — confirm the veg map-stamp contract before APS asserts consumer parity.

```text
[/PLAN-APS-EVOLUTION-VEG-CAPABILITY-001] dWF->@orchestrator-mcp <APS-EVO-E0..E5> · veg-catalog schema <@planner-mcp> · LG-4 pixel proof gate <@coder>
```
