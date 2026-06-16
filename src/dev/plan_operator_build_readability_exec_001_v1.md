# PLAN-BUILD-READABILITY-001 — Footprint · grammar · world scale (operator exec) `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-BUILD-READABILITY-001** |
| **Slice ID** | **⟨PLAN-BUILD-READABILITY-001⟩** |
| **Date** | 2026-06-13 |
| **Status** | **ACTIVE** — operator-reported; feeds `@designer` + `@coder` |
| **Parent** | [`construction_procedural_growth_index_v1.md`](construction_procedural_growth_index_v1.md) · [`plan_building_grammar_evolution_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_building_grammar_evolution_v1.md) · [`plan_aps_artist_tool_exec_v1.md`](plan_aps_artist_tool_exec_v1.md) |
| **Baseline spec (operator)** | [`prompts/guides/build_grammer2_exman.md`](../prompts/guides/build_grammer2_exman.md) — ARCH-DNA · pressure β · site scale · simple warehouse pilot (**v0 baseline**, not full vNext graph solver) |
| **Related (P0 polish)** | [`plan_product_polish_exec_001_v1.md`](plan_product_polish_exec_001_v1.md) — zoom ghost · MAP-PICK · cursor |
| **Owner routing** | `@orchestrator` → `@designer` (readability + APS UX) → `@coder` (ECS spine) · `@designer-mcp` / `@coder-mcp` (grammar RON + APS fields) |

---

## Operator report (2026-06-13, cleaned)

> Building UX is **click-to-place OK**, but **scale and rotation do not read** — the footprint does not visibly grow when scaling; rotation is hard to validate because everything is square. **Ghost → footprint → final building** is uncompelling: missing colored tiles / meshes on the committed sprite or object. **Buildings feel tiny** compared to the world; terrain **chunks read as flat homogeneous green** (forest/grass) with no interior variation, so either **terrain needs granularity** or **building/world scale** (likely both). **Cursor** sometimes appears over widgets (build menu) while the in-world crosshair is still slightly misaligned — we should **hide the world cursor** when menus capture the pointer and **debug-track** visibility, click point, and action point. For **grammar and the artist tool**: define **aesthetic levels** and a **roundness** (or similar) metric as **β parameters** in grammar — optionally **preset per style pack** — so warehouses/factories can be **L-shaped or other non-rect footprints** that match real industrial massing, not default squares; **scale should grow the footprint** through the same parametric path used at commit.

**Planner baseline:** [`prompts/guides/build_grammer2_exman.md`](../prompts/guides/build_grammer2_exman.md) — **ARCH-DNA → pressure β → shape candidates → site composition** as **v0** (simple warehouse pilot). **Not** full vNext (program/flow/adjacency graphs, operator history, settlement grammar). **Site > building:** primary structures ≈ **15–40%** of site — fixes “doll-house on green chunk.”

---

## Grammar baseline vs vNext (planner plot)

| Layer | **Repo today** | **v0 baseline (this plan)** | **vNext (defer)** |
|:---|:---|:---|:---|
| Labeling | Era → Style → Type → Rect/L | **ARCH-DNA** → **β** → ranked candidates | ProgramGraph → FlowGraph → Adjacency |
| Shape | `footprint_mode` enum | β-weighted `long_hall` / `l_shape` / `yard_complex` + **FootprintMatrix** | Volume graph; L emerges without enum |
| Site | One building per chunk | **Site stub:** primary + yard + service (+ rail edge ASCII) | Full SiteGraph (roads, parking, tanks…) |
| Artist | W×D + massing id | **DNA preset** + **β v0 subset** sliders | Operator lattice, growth epochs |

**Meta-rule (guide):** Function > shape · Site > building · Pressure > shape. Era = weak; lineage + function = strong.

### v0 pipeline

```text
ARCH-DNA { F,L,C,D,W,I,S,P,M,A }  ← district / style-pack preset
      ▼
PRESSURE-FIELD { βsym, βirr, βyard, βsvc, βmod, βexp, βvert, βroof }  ← v0: 8 β
      ▼
SHAPE CANDIDATES (weighted)  ← existing massing ids
      ▼
SITE-COMPOSITION stub  ← 2–4 zones (primary, yard, service)
      ▼
MODULE-RUNS → MATERIAL → WEATHERING  ← PG-2 / APS (unchanged)
```

### Simple pilot — Industrial Rail Warehouse (guide §ARCH-DNA EXAMPLE)

| DNA | Value |
|:---|:---|
| F=Logistics · L=Industrial-British · C=Temperate · D=Sparse · W=Industrial · I=Rail · S=Controlled · P=Utilitarian · M=Steel · A=Weathered |

| β (preset) | .72 sym · .24 irr · .84 exp · .18 vert · .93 yard · .88 svc · .92 mod · .63 roof |

**Candidates:** RailEdge > DoubleBar > FactoryCluster > SawtoothHall · **Site plan:** warehouse + loading wing + utility yard + rail spur (guide ASCII).

> **Sample only — not the product.** The warehouse row in the guide is **one reference preset** for ARCH-DNA + site ASCII. Production path = **catalog-driven pilots** (`_pilot_catalog.ron` + `_mock_shapes.ron`), not warehouse-named Rust branches.

**Program stub (data only v0):** `storage=high, loading=high, office=low, service=medium, expansion=high` → topology hint Loading → Storage → Office|Utility.

---

## Pilot sample policy (BUILD-READ-PILOT-001 — anti-hardcoding)

**Problem:** Agents over-fit **Industrial Rail Warehouse** and add `if preset == "logistics_rail_warehouse_v0"` stubs that must be deleted later.

**Correct model:** Incremental **pilot set** on disk → generic loaders → witnesses that fail on warehouse-only shortcuts.

### Authority (data, not code)

| Artifact | Role |
|:---|:---|
| [`assets/configs/buildings/_pilot_catalog.ron`](../../assets/configs/buildings/_pilot_catalog.ron) | **Single pilot registry** — id, label, `FootprintMatrix` cells, `arch_dna_preset`, grammar row |
| [`assets/configs/buildings/_mock_shapes.ron`](../../assets/configs/buildings/_mock_shapes.ron) | Shape QA tray (rect / T / O / L) — rotate·scale tests without art |
| [`assets/configs/buildings/pilots/*.json`](../../assets/configs/buildings/pilots/) | Optional per-pilot site-zone grid (when site stub needed) |
| [`tools/mcp/schemas/examples/arch_dna_*.json`](../../tools/mcp/schemas/examples/) | ARCH-DNA preset **examples** — warehouse JSON is **one file**, not special in Rust |

**Expand the set by adding rows** to `_pilot_catalog.ron` + optional site JSON — **no new Rust `match` arms per building.**

### Incremental test matrix (v0 minimum)

| Pilot id | Shape source | Why |
|:---|:---|:---|
| `shape_rectangle_2x2` | mock_shapes | Baseline rect · scale monotonic |
| `shape_l_3x2` | mock_shapes | Non-square rotate QA |
| `shape_t_3x3` / `shape_o_3x3` | mock_shapes | Asymmetric massing |
| `logistics_rail_warehouse_v0` | pilot catalog | Guide sample · L-yard · site stub |

Witness exit: **`pilot_catalog_parity_witness`** — `pilots.len() >= 4` and every pilot loads matrix + ghost raster without preset-specific Rust.

### Forbidden (agent anti-cheat)

| Pattern | Why reject |
|:---|:---|
| `if preset_id == "logistics_rail_warehouse_v0"` (or any single pilot id) in `src/construction/` | Bypasses generic grammar → matrix path |
| Rust fn named `logistics_rail_warehouse_*` except **test fixtures** or **RON loader** | Becomes second catalog |
| Witness green only on warehouse catalog id | Proves one stub, not pipeline |
| Site overlay hardcoded to one preset without catalog lookup | Blocks N-site expansion |

**Allowed:** Generic `export_footprint_matrix_from_grammar(result, footprint_mode)` · load site grid by `pilot.site_json_path` from catalog · ARCH-DNA preset loader keyed by **any** preset id string from JSON.

### Refactor debt (known today — replace, do not extend)

| Location | Debt |
|:---|:---|
| `building_grammar.rs` | `logistics_rail_warehouse_l_matrix()` + preset id early-return |
| `site_stub_overlay.rs` / `site_zone_grid.rs` | Warehouse-only site grid fn |
| Witness self-checks | Single pilot id in `build_read_shape_002` / `visual_001` |

**ΔWF:** @coder — route matrix/site through `_pilot_catalog.ron` loader; delete preset-id branches; widen witnesses to **pilot set parity**.

---

### β v0 subset (APS + RON first)

βsym, βirr, βyard, βsvc, βmod, βexp (link Shift+scale), βvert, βroof — **defer:** βorn, βdef, βctl, βentropy, βinertia, βdepth (document only in schema doc).

### Shape map (guide INDUSTRIAL → repo)

| Guide | Repo `industrial_warehouse_v1.ron` |
|:---|:---|
| Bar | `long_hall` |
| DoubleBar | `double_hall` |
| ServiceYard / RailEdge | `l_shape`, `yard_complex` |
| SawtoothHall, FactoryCluster, … | labels in `grammar_labels_v1.json`; β weights only in v0 |

---

## Problem clusters (for routing)

| Cluster | Operator acceptance | Repo gap today | Primary owner |
|:---|:---|:---|:---|
| **A — Placement read** | Scale/rotate in Adjust mode visibly changes occupied tiles before commit | Weighted raster wired at commit; tray still mostly **filled rectangles**; validation bbox may lag weights | **@coder** |
| **B — Shape catalog** | At least one **non-square** pilot (L-yard warehouse) in build tray for rotate/scale QA | Grammar has `footprint_mode: l_shape` in `building_grammar.rs`; **catalog `FootprintMatrix` + tray defs remain rects** | **@designer-mcp** spec → **@coder** catalog |
| **C — ARCH-DNA + pressure β** | Artist sets **DNA preset** + **β v0 subset**; grammar ranks shape candidates | Only `width_depth_ratio` + `footprint_mode`; no ARCH-DNA or β field | **@planner-mcp** schema → **@coder-mcp** APS → **@coder** evaluator |
| **D — Visual finish** | Committed site shows **production or lod0 tiles/mesh**, not grey void | PG-2 extract + tile production lane incomplete for sim viewport | **@coder** + **@coder-mcp** (tiles) |
| **E — World / site scale** | **Site** reads at chunk scale; primary building **15–40%** of site; not one rect filling chunk | Building ≈ 95% of chunk; flat green fill | **@designer** site-composition mock → **@coder** |
| **F — Pointer chrome** | No OS cursor over HUD widgets; no world crosshair under panels; debug triage fields | **In progress** — `SimulationMapPointerGate` + placement debug | **@coder** (verify) · **@designer** (HUD hit regions) |

**Hard invariant (unchanged):** preview/ghost never mutates gameplay; single commit funnel via `src/construction/`. Grammar/APS outputs are **data** consumed by commit — not parallel spawn paths.

---

## Authority spine (do not invert)

```text
BuildGhostState (origin, scale_factor, rotation, mirror)
  → placement_snapshot_for_building → weighted_footprint::rasterize_with_effective_scale
  → ConstructionVisualRequests (egui footprint tiles)
  → CommitConstructionSiteEvent + CommittedPlacementSnapshot
  → ProceduralBuildingSpec / PG-2 assembly extract → registry GPU / iso tiles
```

Footprint **geometry authority** for overlap: **weighted tiles**, not axis-aligned `FootprintTiles` alone. Rectangular `ghost.footprint` is a **validation bbox** derived from weight bounds.

---

## Phase map

| Phase | ID | Owner | Goal |
|:---:|:---|:---|:---|
| **0** | **BUILD-READ-P0** | @coder | Close operator P0: zoom coherence witness, pointer gate verify, weighted scale on map |
| **1** | **BUILD-READ-DESIGN-001** | @designer | Readability + **site composition** mock (guide ASCII); **15–40%** occupancy rule |
| **2** | **BUILD-READ-SHAPE-001** | @designer-mcp → @coder | Rail warehouse **FootprintMatrix** + site stub zones |
| **3** | **BUILD-READ-GRAMMAR-v0-001** | @planner-mcp → @coder-mcp → @coder | **ARCH-DNA** + **β v0** schema; APS presets; evaluator ranks massing |
| **3b** | **BUILD-READ-SITE-v0-001** | @designer + @coder | Site stub overlay (yard/service/rail) — view-only, no new commit path |
| **4** | **BUILD-READ-VISUAL-001** | @coder + @coder-mcp | Ghost → commit → lod0/production tile or mesh on map (PG-2 / PT lane) |
| **5** | **BUILD-READ-WORLD-001** | @designer + @coder | Chunk interior variation **or** building iso scale bump — pick one primary lever per witness |

---

## Todo board (machine rows)

| ID | Owner | Task | Depends | Witness / exit |
|:---|:---|:---|:---|:---|
| **BUILD-READ-P0-001** | @coder | Confirm `parametric_placement_snapshot` uses weighted raster; Adjust mode Shift+scroll grows overlay tiles | — | Manual: lock ghost → scale → tile count ↑; `scaling_audit_s1` green |
| **BUILD-READ-P0-002** | @coder | Refresh zoom coherence: `map_zoom_coherence_live.json` after `--test vfx` scroll torture | PLAN-PRODUCT-POLISH P1 · **REWIRE-003** | No double-world trail ≥5 zoom cycles · **blocked** — witness module unwired |
| **BUILD-READ-P0-003** | @coder | Pointer gate: Construction toolbox + submenus block pick; debug overlay shows `egui_blocks`, `os_cursor_hidden` | TRIAGE-CURSOR-UNIFY · **REWIRE-001/002** | Pick under BuildToolbox = no commit · **on disk, not compiled** |
| **BUILD-READ-DESIGN-001** | @designer | **Readability brief** (≤2 pages): min/max building screen height vs chunk; footprint valid/risky/invalid at partial alpha; when to show mesh vs tiles | BUILD-READ-P0-001 | Sign-off row in `designer_signoff_registry.json` |
| **BUILD-READ-DESIGN-002** | @designer | Annotate **Build Toolbox** + Adjust mode affordances (Ctrl=rotate, Shift=scale) — not hover-only | BUILD-READ-DESIGN-001 | HUD copy in `contextual_tip` / `build_toolbox` · witness `build_read_ux_live.json` |
| **BUILD-READ-SHAPE-001** | @designer-mcp | **Industrial Rail Warehouse** pilot: `FootprintMatrix` + ARCH-DNA preset row in RON (guide §EXAMPLE) | PG-MODULE-AUDIT · [`build_grammer2_exman.md`](../prompts/guides/build_grammer2_exman.md) | Matrix + DNA preset on disk |
| **BUILD-READ-SHAPE-002** | @coder | Wire catalog matrix + DNA preset into tray; ghost overlay uses matrix cells | BUILD-READ-SHAPE-001 | L matrix → N occupied ≠ width×depth |
| **BUILD-READ-SHAPE-003** | @coder | β-weighted massing pick → `footprint_mode` + matrix export (not enum-only) | BUILD-READ-GRAMMAR-v0-003 | Witness: RailEdge preset → `l_shape` or yard |
| **BUILD-READ-GRAMMAR-v0-001** | @planner-mcp | Schema: `ArchDna` axes + **β v0** (8 keys); preset `logistics_rail_warehouse_v0`; map to `industrial_warehouse_v1.ron` | [`build_grammer2_exman.md`](../prompts/guides/build_grammer2_exman.md) | **🟢 done** — [`arch_build_grammar_v0_baseline_v1.md`](arch_build_grammar_v0_baseline_v1.md) + schema + example JSON (2026-06-13) |
| **BUILD-READ-GRAMMAR-v0-002** | @coder-mcp | APS: DNA preset dropdown + β sliders (v0 subset); snapshot stores `arch_dna` + `pressure_field` | v0-001 | pytest + snapshot JSON fields |
| **BUILD-READ-GRAMMAR-v0-003** | @coder | Evaluator: DNA+β → score massing strategies → pick + `effective_scale` from βexp | v0-001, PG-QUALITY-001 | `grammar_diversity_witness.json` adds `logistics_rail_warehouse_v0` row |
| **BUILD-READ-SITE-v0-001** | @designer | Site-composition mock: primary + yard + service + rail (guide ASCII); **15–40%** primary footprint rule | BUILD-READ-DESIGN-001 | Annotated mock in design brief |
| **BUILD-READ-SITE-v0-002** | @coder | View-only site stub overlay (yard void tiles + labels) from committed/archived site plan — no gameplay mutation | BUILD-READ-SITE-v0-001 | Overlay witness in placement debug |
| **BUILD-READ-VISUAL-001** | @coder | After commit, site extract attaches **lod0+** module mesh or iso tile stamp — no footprint-only fallback in sim | PROC-PG-2-001, PLAN-PROC-TILE-PROD-001 | `construction_stage_live.json` mesh_tier_used ≠ fallback_primitive for pilot |
| **BUILD-READ-VISUAL-002** | @coder-mcp | Production tile bake for pilot warehouse variant; promote → Bevy registry | BUILD-READ-SHAPE-001 | MCP validate + registry path green |
| **BUILD-READ-WORLD-001** | @designer | **Site-scale** targets: primary structure **15–40%** of visible site/chunk; yard/service readable at default zoom | BUILD-READ-DESIGN-001 · guide §SITE-GRAMMAR | Scale table + site ASCII in brief |
| **BUILD-READ-WORLD-002** | @coder | Implement chosen lever: (A) iso building draw scale multiplier **or** (B) chunk sub-tile variation pass — **one primary**, other deferred | BUILD-READ-WORLD-001 | Before/after screenshot + witness JSON |
| **BUILD-READ-DEBUG-001** | @coder | Extend placement debug: `cursor_visible`, `click_screen`, `pick_world`, `action_tile`, `scale_factor`, `weight_tile_count` | BUILD-READ-P0-003 · **REWIRE-001** | Overlay fields populated in Adjust mode · **blocked** |
| **BUILD-READ-PILOT-001** | @coder | **Pilot catalog authority:** load matrix + site from `_pilot_catalog.ron`; remove warehouse-named Rust branches | BUILD-READ-SHAPE-002 · **REWIRE-004** | `pilot_catalog_parity_witness` ≥4 pilots green · **partial** |
| **BUILD-READ-PILOT-002** | @designer-mcp | Add pilot rows (rect + T/O/L + warehouse) in catalog only — no Rust | BUILD-READ-PILOT-001 | `_pilot_catalog.ron` rows on disk |

---

## `@designer` delegation prompt

```text
Lane: PLAN-BUILD-READABILITY-001 · BUILD-READ-DESIGN-001/002 + BUILD-READ-SITE-v0-001 + BUILD-READ-WORLD-001

Read: src/dev/plan_operator_build_readability_exec_001_v1.md
      prompts/guides/build_grammer2_exman.md  (§SITE-GRAMMAR, §ARCH-DNA EXAMPLE)

Operator pain: buildings feel doll-house small because site = one building filling chunk.
Guide rule: primary structures ≈ 15–40% of site; yards/service/rail are first-class void.

Deliver:
1. Readability brief: site-composition ASCII (rail spur + warehouse + loading + utility yard)
2. 15–40% occupancy rule at default sim zoom (px + world tiles)
3. Footprint partial-alpha language; mesh vs iso tile authority
4. HUD copy for Adjust mode (Ctrl rotate, Shift scale)
5. Sign-off: BUILD-READ-DESIGN-001 + BUILD-READ-SITE-v0-001

Do NOT: program/flow graph solver, operator history, settlement grammar (vNext).
```

---

## `@coder` delegation prompt

```text
Lane: PLAN-BUILD-READABILITY-001 · BUILD-READ-P0 + SHAPE-002/003 + VISUAL-001 + DEBUG-001

Read: src/dev/plan_operator_build_readability_exec_001_v1.md
Skills: bevy-simulation-grade (07 authority map) · validation-first

Spine: weighted_footprint is footprint authority; construction invariants unchanged.

Priority order:
1. P0 verify: weighted scale overlay, pointer gate over BuildToolbox, zoom witness refresh
2. BUILD-READ-SHAPE-002: non-rect FootprintMatrix from catalog → tray → visual requests
3. BUILD-READ-SHAPE-003: grammar l_shape → FootprintMatrix on commit path
4. BUILD-READ-VISUAL-001: post-commit lod0/production extract visible in sim (not greybox-only)
5. BUILD-READ-DEBUG-001: placement debug triage fields

Tests: cargo test -p proc_A_dine01 --lib construction::scaling_audit construction::weighted_footprint
Witness: debug_runs/construction_stage_live.json · map_zoom_coherence_live.json

Defer world-scale lever (BUILD-READ-WORLD-002) until @designer site-scale table lands.
```

---

## `@planner-mcp` delegation prompt (v0 baseline schema)

```text
Lane: BUILD-READ-GRAMMAR-v0-001

Read: prompts/guides/build_grammer2_exman.md (full)
      src/dev/plan_operator_build_readability_exec_001_v1.md (§Grammar baseline vs vNext)
      assets/configs/buildings/grammars/industrial_warehouse_v1.ron (repo today)

Deliver arch_build_grammar_v0_baseline_v1.md:
1. ArchDna struct: F,L,C,D,W,I,S,P,M,A (enums from guide §ARCH-DNA)
2. PressureField v0: 8 β keys with ranges + preset logistics_rail_warehouse_v0
3. Mapping: β → massing strategy weights (not new geometry)
4. Explicit OUT OF SCOPE list: ProgramGraph, FlowGraph, AdjacencyMatrix, OperatorStack,
   GrowthEpochs, SettlementGrammar, file split program/topology/growth/
5. Migration path note: v0 keeps footprint_mode; vNext retires shape-as-input

Do NOT implement Rust/Python — schema + preset tables only.
```

---

## vNext explicitly deferred (from guide — do not assign yet)

| Guide concept | Why defer |
|:---|:---|
| ProgramGraph / FlowGraph / Adjacency solver | Needs new ECS + solver; v0 uses program **stub** string only |
| Massing operators (AddVolume, Carve, …) | Requires volume graph; v0 uses perimeter grid |
| Operator history / growth epochs | Simulation + save slice |
| βctl, βentropy, βinertia, βdepth | Document in schema; implement after v0 witness |
| Topology classes LINEAR/RADIAL/NETWORK | Classify in planner doc only |
| Settlement grammar / steelworks-scale graphs | OG-4 / district lane |
| Remove `massing_strategy` enum | Only after volume graph lands |

---

## `@designer-mcp` + `@coder-mcp` delegation prompt

```text
Lane: PLAN-BUILD-READABILITY-001 · BUILD-READ-SHAPE-001 · BUILD-READ-GRAMMAR-v0-002

Read: src/dev/plan_operator_build_readability_exec_001_v1.md
      prompts/guides/build_grammer2_exman.md  (§ARCH-DNA, §PRESSURE FIELD, §SHAPE-GRAMMAR)

designer-mcp:
- Spec Industrial Rail Warehouse pilot: ARCH-DNA row + FootprintMatrix + site zones
- β v0 semantics (βyard, βsvc, βsym, …) — deterministic presets per style pack
- Critique: v0 uses existing massing ids; do not author 50 new shape enums

coder-mcp:
- APS: DNA preset dropdown + β v0 sliders → snapshot `arch_dna` + `pressure_field`
- Extend grammar_labels for RailEdge, FactoryCluster, SawtoothHall (labels only in v0)

Exit: BUILD-READ-SHAPE-001 RON; BUILD-READ-GRAMMAR-v0-002 pytest green
```

---

## Acceptance probes (operator replay)

| # | Action | Pass |
|:---:|:---|:---|
| 1 | Sim → Industry tool → pick warehouse → lock → Shift+scroll up | Footprint tile count or partial-alpha area **monotonic increase** |
| 2 | Same ghost → Ctrl+scroll | Occupied tiles **rotate** 90° steps; L-shape ≠ square bounding box |
| 3 | Commit valid ghost | Within 2 frames, **visible structure** (tile or mesh) — not empty footprint |
| 4 | Default zoom, pan site | Primary structure **15–40%** of site/chunk; yard/service zones visible (guide §SITE-GRAMMAR) |
| 5 | Open Build Toolbox over map | OS cursor on panel; **no** world crosshair; LMB does not place |
| 6 | Toggle placement debug | Fields: egui_blocks, scale_factor, weight_tile_count populated |

---

## Already landed (steward lane — do not re-assign)

**Unwired recovery (2026-06-13):** Several rows below exist **on disk** but were **removed from `mod.rs`** to keep `cargo check` green. Re-wire per [`build_read_unwired_spine_v1.md`](build_read_unwired_spine_v1.md) before closing BUILD-READ-P0.

| Fix | File / note | Wire status |
|:---|:---|:---:|
| Weighted raster on commit snapshot | `parametric_commit.rs` → `rasterize_with_effective_scale` | 🟢 compiled |
| Ghost footprint bbox from weights | `build_refresh_placement_validation_system` | 🟢 compiled |
| Zoom dirty-all + raised chunk budget | `tile_world_fallback.rs` | 🟢 compiled |
| Post-egui pointer gate + widget hit test | `simulation_pointer_gate.rs` | ⚠ **unwired** — REWIRE-002 |
| Placement debug overlay + pick probe | `placement_debug.rs` | ⚠ **unwired** — REWIRE-001 |
| Footprint draw single projection | `visual_authority.rs` | 🟢 compiled (witness helpers may be missing) |
| Map zoom coherence witness | `map_zoom_coherence_live_proof.rs` | ⚠ **unwired** — REWIRE-003 |
| Minimap Bevy GPU interaction | `minimap_bevy_interaction.rs` | ⚠ **unwired** — MINIMAP-REWIRE-001 |
| Pilot catalog loader | `pilot_catalog.rs` | 🟡 partial — REWIRE-004 |

Verify in runtime before closing **BUILD-READ-P0**.

---

## Index hook

Add to [`construction_procedural_growth_index_v1.md`](construction_procedural_growth_index_v1.md) deliverables when Phase 1 designer brief is signed:

| Queue ID | Doc | Owner | Status |
|:---|:---|:---|:---:|
| **PLAN-BUILD-READABILITY-001** | this doc | @designer + @coder | **ACTIVE** |

---

## Planner sign-off

| Role | Status | Date |
|:---|:---|:---|
| @planner (thin exec) | **SIGNED** — operator report + **build_grammer2_exman v0 baseline** plot | 2026-06-13 |
| @planner-mcp (v0 schema) | **SIGNED** — `arch_build_grammar_v0_baseline_v1.md` | 2026-06-13 |
| @designer | **PASS** BUILD-READ-DESIGN-001/002 + SITE-v0-001 + WORLD-001 + POINTER-HUD + MAP-ZOOM + FIRE-PLAY-VIS + P0-MINIMAP | 2026-06-13 |
| @designer-mcp | **PASS** BUILD-READ-SHAPE-001 + SITE-ZONE-MAP + VISUAL-002 spec | 2026-06-13 |
| @coder | **unwired** BUILD-READ-P0 — code on disk; **mod.rs wiring pending** — see [`build_read_unwired_spine_v1.md`](build_read_unwired_spine_v1.md) | 2026-06-13 |
