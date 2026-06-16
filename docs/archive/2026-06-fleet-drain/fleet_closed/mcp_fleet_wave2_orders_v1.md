# MCP fleet Wave 2 orders `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **MCP-FLEET-WAVE2-001** |
| **Replaces** | Wave 1 drained — see [`mcp_active_queue.json`](../../tools/orchestrator/queues/mcp_active_queue.json) |
| **Owner** | `@orchestrator-mcp` |
| **Date** | 2026-06-02 |
| **Status** | **CLOSED** — ART/DATA/TILE schema complete; see Wave 3 |
| **Successor** | [`mcp_fleet_wave3_engine_orders_v1.md`](mcp_fleet_wave3_engine_orders_v1.md) |
| **Roadmap** | [`plan_kit_lod0_roadmap_v1.md`](plan_kit_lod0_roadmap_v1.md) **SIGNED** |
| **Machine queue** | [`mcp_active_queue.json`](../../tools/orchestrator/queues/mcp_active_queue.json) |

**Wave 1 done:** 10 lod0 modules (001+002), registry tier filter, tier validators, roadmap signed.

**Wave 2 goal:** **40 more lod0 modules** (003→010) + **engine PG-1/PG-2** + **style packs** + tile schema prep.

---

## Coverage snapshot (final — 2026-06-02)

| Metric | Count |
|:---|---:|
| Canonical kit modules (lod0) | **50 / 50** |
| Art batches G5 | **10 / 10** |
| Style pack RON files | **7 / 7** |
| Index rows (incl. smoke) | 81 |
| `src/construction/procedural/types.rs` | **pending** — Wave 3 MCP-PG-1-001 |

**Closure:** [`mcp_fleet_wave2_art_closure_live.json`](../../debug_runs/art_pipeline/mcp_fleet_wave2_art_closure_live.json)

---

## Three parallel streams

```text
Stream ART   @designer-mcp → @coder-mcp     kit_lod0_003 … kit_lod0_010
Stream DATA  @planner-mcp → @designer-mcp   style pack schema + 7 pack RON specs
Stream ENGINE @planner → @coder             PG-1 types/loaders → PG-2 footprint extract
Stream TILE  @planner-mcp → @coder-mcp      tile_batch_v1 schema (no execution)
```

**Rule:** designer-mcp may run **G0–G1 for batch N+1** while coder-mcp executes **G2–G5 for batch N**.

---

## Stream ART — lod0 batches (designer-mcp + coder-mcp)

Module picks are **fixed** in [`plan_kit_lod0_roadmap_v1.md`](plan_kit_lod0_roadmap_v1.md). Do not renegotiate ids.

| Task | Agent | batch_id | Modules | Status |
|:---|:---|:---|:---|:---|
| MCP-PLN-SP-001 | planner-mcp | — | StylePack RON schema | **DONE** |
| MCP-PLN-PG2-001 | planner | — | PG-1/PG-2 exec v1.1 | **DONE** |
| MCP-T0-001 | planner-mcp | — | tile plan SIGNED | **DONE** |
| MCP-D0-003 | designer-mcp | `kit_lod0_003` | wall_concrete_2u, roof_sawtooth, door_warehouse, win_industrial_3u, prop_vent | **READY** |
| MCP-C0-004 | coder-mcp | `kit_lod0_003` | execute G2–G5 | blocked on D0-003 G1 |
| MCP-D0-004 | designer-mcp | `kit_lod0_004` | wall_brick_2u, wall_wood_2u, roof_pitched_hip, door_garage, win_arched_1u | **READY** (G0–G1 parallel) |
| MCP-C0-005 | coder-mcp | `kit_lod0_004` | execute G2–G5 | blocked on D0-004 G1 |
| MCP-D0-005 | designer-mcp | `kit_lod0_005` | wall_glass_curtain_1u, wall_industrial_panel_2u, roof_shed, door_office, win_strip_2u | after 004 G1 |
| MCP-C0-006 | coder-mcp | `kit_lod0_005` | execute | blocked on D0-005 |
| MCP-D0-006 | designer-mcp | `kit_lod0_006` | wall_military_bunker_1u, roof_parapet, door_civic, win_shop_2u, prop_light | after 005 G1 |
| MCP-C0-007 | coder-mcp | `kit_lod0_006` | execute | blocked on D0-006 |
| MCP-D0-007 | designer-mcp | `kit_lod0_007` | roof_metal_low, roof_tile, door_military, win_house_1u, corner_L | after 006 G1 |
| MCP-C0-008 | coder-mcp | `kit_lod0_007` | execute | blocked on D0-007 |
| MCP-D0-008 | designer-mcp | `kit_lod0_008` | roof_bunker, roof_canopy, door_factory, win_office_1u, corner_T | after 007 G1 |
| MCP-C0-009 | coder-mcp | `kit_lod0_008` | execute | blocked on D0-008 |
| MCP-D0-009 | designer-mcp | `kit_lod0_009` | door_double_shop, door_gate_industrial, win_bunker_slit, win_skylight_1u, corner_parapet | after 008 G1 |
| MCP-C0-010 | coder-mcp | `kit_lod0_009` | execute | blocked on D0-009 |
| MCP-D0-010 | designer-mcp | `kit_lod0_010` | prop_fence, prop_tank, prop_transformer, prop_ac, prop_chimney | after 009 G1 |
| MCP-C0-011 | coder-mcp | `kit_lod0_010` | execute — **50/50 kit complete** | blocked on D0-010 |
| MCP-C0-012 | coder-mcp | — | `kit_lod0_batch_runner.py` — parametric runner from roadmap table | **READY** (P2) |

### Per-batch designer-mcp deliverables (every D0-00N)

1. `debug_runs/art_pipeline/<batch>_g0_rules.yaml`
2. `tools/mcp/schemas/examples/batch_<batch>.manifest.json`
3. `assets/staging/specs/<module_id>.json` × 5
4. `tools/mcp/schemas/examples/<module_id>_lod0_job.json` × 5
5. `validate_report mcp_spec` + `mcp_job` green on all 5

**Every spec:** `development_tier: lod0`, canonical `module_id`, seed 42, real `profile` from roadmap — **no greybox cheats**.

### Per-batch coder-mcp deliverables (every C0-00N)

1. `geometry_run_job` × 5 (or batch runner)
2. `validate_asset_report` per GLB (validation-first)
3. G4 sign-off review → promote × 5 → `library_register`
4. `write_witness <batch>` → G5 pass
5. `pytest tools/mcp/python/tests/ -q`

**Template:** `tools/mcp/scripts/kit_lod0_001_batch.py`, `kit_lod0_002` pattern.

---

## Stream DATA — style packs (planner-mcp + designer-mcp)

PG-2 cannot demo distinct facades without StylePack → module_id maps.

| Task | Agent | Goal | Output |
|:---|:---|:---|:---|
| MCP-PLN-SP-001 | planner-mcp | StylePack RON schema + loader contract | `docs/archive/2026-06-src-dev/plans/plan_style_pack_ron_v1.md` + JSON schema draft |
| MCP-D0-SP-001 | designer-mcp | 7 style pack RON **specs** (not meshes) | `assets/configs/buildings/style_packs/style_*.ron` × 7 |
| MCP-D0-SP-002 | designer-mcp | Map each pack → subset of lod0 module_ids | witness `debug_runs/art_pipeline/style_packs_manifest_live.json` |

**Style packs (from module kit):** victorian, modern, industrial_west, industrial_soviet, military, rural, colonial (+ port/railway tags).

**Depends on:** at least lod0_001+002 modules (10 ids) — **unblocked now**.

---

## Stream ENGINE — PG-1 / PG-2 (@planner + @coder)

| Task | Agent | Goal | Files (≤3 per PR) |
|:---|:---|:---|:---|
| MCP-PLN-PG2-001 | planner | Expand [`plan_procedural_build_gen_exec_001_v1.md`](plan_procedural_build_gen_exec_001_v1.md) PG-1/PG-2 with witness keys + test names | plan doc only |
| MCP-PG-1-001 | coder | Archetype + StylePack types + RON loaders | `src/construction/procedural/types.rs`, `load.rs`, `mod.rs` |
| MCP-PG-2-001 | coder | Footprint W/D/C grid + procedural build extract | `footprint_grid.rs`, `procedural_build_extract.rs` |
| MCP-PG-2-002 | coder | Wire `RepresentationResult.procedural_module_meshes` + scene catalog | `procedural_module_extract.rs`, extract graph |
| MCP-PG-2-WIT | coder | Witness `debug_runs/procedural_assembly_live.json` | lib test + optional `--test visual` hook |

**PG-2 exit:** tactical view shows **different module GLBs** per StylePack for same footprint — using **lod0 only**, never smoke.

**Read:** [`construction_procedural_growth_index_v1.md`](construction_procedural_growth_index_v1.md) § PG-2 mesh authority.

---

## Stream TILE (defer execution)

| Task | Agent | Goal |
|:---|:---|:---|
| MCP-T0-001 | planner-mcp | Sign [`plan_tile_batch_v1_planner_mcp_v1.md`](plan_tile_batch_v1_planner_mcp_v1.md) |
| MCP-T0-002 | coder-mcp | Ship `tools/mcp/schemas/tile_batch_v1.schema.json` + `validate_report tile_batch` stub |
| MCP-T0-003 | designer-mcp | Author `tools/mcp/schemas/drafts/tile_batch_factory_floor_v1.json` validate-only |

**No `tile.generate` execution** until registry says SHIPPED.

---

## Stream UX (@designer — not designer-mcp)

| Task | Agent | Goal |
|:---|:---|:---|
| MCP-DUX-PG2-001 | designer | PG-2 assembly **player read** charter — what players see at lod0 vs production | `docs/archive/2026-06-src-dev/plans/design_procedural_assembly_read_v1.md` |
| MCP-DUX-PG2-002 | designer | Sign-off rubric for coder PG-2 witness (after MCP-PG-2-WIT) | YAML in `debug_runs/` |

---

## Priority order (orchestrator)

| P | Tasks | Why |
|:---|:---|:---|
| **P0** | MCP-D0-003 → MCP-C0-004 | Unblocks prop + sawtooth silhouettes |
| **P0** | MCP-PLN-SP-001 → MCP-D0-SP-001 | Unblocks PG-1 loader data |
| **P1** | MCP-PG-1-001 → MCP-PG-2-001 | Engine can consume lod0 modules |
| **P1** | MCP-D0-004 + MCP-C0-005 | Keep art pipeline ahead of PG-2 |
| **P2** | MCP-D0-005 … D0-010 (designer ahead) | Fill 50-module kit |
| **P2** | MCP-C0-012 batch runner | Reduce coder-mcp toil |
| **P3** | MCP-T0-* tile schema | Parallel, no execution |

---

## Paste prompts

### @designer-mcp (start now)

> Wave 2 **MCP-D0-003** + **MCP-D0-004** from `docs/archive/2026-06-fleet-drain/fleet_closed/mcp_fleet_wave2_orders_v1.md`. Read `plan_kit_lod0_roadmap_v1.md` for fixed module picks. Author G0–G1 only (no promote). validate_report all specs/jobs. Then **MCP-D0-SP-001** style pack RON specs for 7 packs using current 10 lod0 module ids.

### @coder-mcp (after D0-003 G1)

> Wave 2 **MCP-C0-004** `kit_lod0_003` G2–G5 from `docs/archive/2026-06-fleet-drain/fleet_closed/mcp_fleet_wave2_orders_v1.md`. validation-first. write_witness. Then continue C0-005… sequential. Optional **MCP-C0-012** batch runner when 003 green.

### @planner-mcp

> Wave 2 **MCP-PLN-SP-001** style pack RON schema + **MCP-T0-001** tile batch sign from `docs/archive/2026-06-fleet-drain/fleet_closed/mcp_fleet_wave2_orders_v1.md`. No bpy. SHIPPED/PLANNED honest.

### @planner

> Wave 2 **MCP-PLN-PG2-001** — expand PG-1/PG-2 exec with witness keys, file paths, test names. Reference `mcp_fleet_wave2_orders_v1.md` Stream ENGINE. StylePack loads from `assets/configs/buildings/style_packs/`.

### @coder

> Wave 2 **MCP-PG-1-001** then **MCP-PG-2-001** from `docs/archive/2026-06-fleet-drain/fleet_closed/mcp_fleet_wave2_orders_v1.md` + `plan_procedural_build_gen_exec_001_v1.md`. Use `ProceduralModuleRegistry::modules_for_stylepack()`. ≤3 files per PR. Witness `procedural_assembly_live.json`.

### @designer

> Wave 2 **MCP-DUX-PG2-001** — procedural assembly player read at lod0. Input: `design_procedural_module_kit_v1.md` + 10 lod0 modules in index. Output: design doc only.

---

## Verification

```powershell
# After each art batch
python -m rust_engine_mcp.cli write-witness kit_lod0_003
python -m pytest tools/mcp/python/tests/ -q

# After PG-2
cargo test -p proc_A_dine01 --lib procedural construction
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Wave 2 — 40 modules + PG-1/2 + style packs + tile schema |
| v1.0.1 | 2026-06-02 | PLN-SP-001 + PLN-PG2-001 + T0-001 done; gate checklist witness |
