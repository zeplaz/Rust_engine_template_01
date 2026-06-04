# PLAN-TILE-FIX-AUTO-BUILD-001 — Procedural tile pipeline correction `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-TILE-FIX-AUTO-BUILD-001** |
| **Source** | [`prompts/planner_fix_auto_build.md`](../../prompts/planner_fix_auto_build.md) |
| **Status** | **FOUNDATION DONE** — active work → [`plan_building_tile_spine_001_v1.md`](plan_building_tile_spine_001_v1.md) |
| **Date** | 2026-06-02 |
| **Supersedes (art)** | Lod0/greybox `buildings_iso/production/*` atlases — **freeze, do not ship** |
| **Parents** | [`design_tile_bake_spine_convergence_v1.md`](design_tile_bake_spine_convergence_v1.md) · [`plan_procedural_building_tiles_production_v1.md`](plan_procedural_building_tiles_production_v1.md) |
| **Successor** | **BUILDING-TILE-SPINE-001** — planner post–line 400 in [`prompts/planner_fix_auto_build.md`](../../prompts/planner_fix_auto_build.md) |

---

## North star

**Procedural system generates the full asset graph (geometry + materials + variant recipes); rendering is the final compile step.** Variants exist **before** render, not as post-hoc color dimming.

```text
BuildingDefinition → Procedural Assembly → VariantRecipe[] → Material resolve
  → assembled.blend → (variant × facing × frame) keyframe_render
  → tilemapgen pack → atlas + visual_config.ron → Registry
  → Runtime VisualState(variant, facing, frame)
```

**Wrong spine (retire):** PG assembly → greybox GLB → single ortho snapshot → pack → registry.

---

## Execution todos (roadmap order)

| ID | Owner | Task | Status |
|:---|:---|:---|:---|
| **TILE-FIX-01** | @planner | Freeze/de-index bogus `buildings_iso/production` atlases; witnesses must not green on PNG-exists alone | **done** |
| **TILE-FIX-02** | @planner-mcp | Atlas schema v2: `variant × facing × frame` in `atlas_meta.json` + `visual_config.ron` | **done** |
| **TILE-FIX-03** | @coder | `VisualState` resolver + map stamp uses `rotation_quarter_turns` / facing index | **done** |
| **TILE-FIX-04** | @coder-mcp | Production module GLB library; MCP assembles `assembled.blend` from real modules | **done** |
| **TILE-FIX-05** | @coder-mcp | Material validator: albedo+normal+roughness or explicit fallback — **FAIL** build, no greybox auto | **done** |
| **TILE-FIX-06** | @coder | `VariantRecipe` / `VariantLayer` generator — combinations before render | **done** |
| **TILE-FIX-07** | @coder-mcp | `BuildingDefinition` schema (modules + variant enum) drives bake matrix | **done** |
| **TILE-FIX-08** | @coder-mcp | Blender compile: apply materials → variant → facing → render; pack via keyframe_render + tilemapgen | **done** |
| **TILE-FIX-09** | @designer-mcp | Pilot: warehouse industrial — minimum state×facing matrix, G4 on real assembly | **done** (Phase C: 3×8, `proceed_ship: yes`) |
| **TILE-FIX-10** | @coder-mcp | Promotion gates: geometry, materials 100%, variants, all facings, lookup test, runtime spawn | **done** (schema only; headless not ship) |

**Next program (planner backlog):** ARCH-001 → PILOT-001 in [`plan_building_tile_spine_001_v1.md`](plan_building_tile_spine_001_v1.md).  
**Recovery only:** [`warehouse_tile_ship_workflow_v1.md`](warehouse_tile_ship_workflow_v1.md) (debug — not primary).

---

## Planner phases (reference)

| Phase | Topic | Key deliverable |
|:---|:---|:---|
| 1 | Asset architecture | Variants **before** render; `BuildingDefinition` + variant list |
| 2 | Real modules | Named production GLBs, not cube-only lod0 for ship art |
| 3 | Material resolution | Hard fail if textures missing |
| 4 | Procedural variant generation | `VariantLayer` → `VariantRecipe`, not manual PNG keys only |
| 5 | Rotation | `RenderContract { facings: 4\|8 }`; matrix = **State × Facing** (+ animation) |
| 6 | Unified atlas schema | `AtlasLookup { variant, facing, frame }` → UV |
| 7 | Runtime resolver | Same lookup for buildings, vehicles, power, props |
| 8 | Blender integration | MCP never ships greybox ortho as production |
| 9 | Output contract | `atlas.png`, `atlas_meta.json`, `visual_config.ron`, `assembly.blend`, `manifest.ron` |
| 10 | Promotion gates | Replace “PNG exists” with full validation checklist |
| 11+ | Authority + graphs | Single SoT chain; Assembly Graph + Variant Graph; APS → MCP → headless worker — see **BUILDING-TILE-SPINE-001** |

---

## Render matrix example (warehouse pilot)

| Axis | Count | Notes |
|:---|:---|:---|
| Facings | 8 | `keyframe_render` + `tilemapgen -pk` |
| States | 6+ | clean_day, clean_night, damage tiers, fire, construction, abandoned |
| Fire animation | 8 frames | × facings = 64 cells for fire strip (or separate atlas per policy) |

**Minimum ship bar:** all required facings × `ship_minimum_keys` from [`_variant_catalog.ron`](../assets/configs/buildings/_variant_catalog.ron), validated lookup + runtime stamp test.

---

## Do not

- Promote current `warehouse_*_production_v1_atlas.png` and siblings.
- Treat `bake_source: keyframe_pack` as green when stills are headless greybox ortho.
- Author “damage/night” by dimming Principled BSDF after a single render.

---

## Agents

| Step | Agent |
|:---|:---|
| Architecture + freeze policy | @planner |
| Schemas + visual contract | @planner-mcp |
| ECS resolver + stamp | @coder |
| MCP modules, validator, bake loop | @coder-mcp |
| G4 pilot sign-off | @designer-mcp |
