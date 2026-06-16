# DESIGN-PROC-MODULE-KIT-001 — Procedural module art kit `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-PROC-MODULE-KIT-001** |
| **Parent** | [`construction_procedural_buildings_plan_v1.md`](construction_procedural_buildings_plan_v1.md) |
| **Coder exec** | [`plan_procedural_build_gen_exec_001_v1.md`](plan_procedural_build_gen_exec_001_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | **PROC-PG-2-001** (greybox); **PROC-PG-1-001** parallel (data-only) |
| **Greybox rule** | Textures block **PG-2** only — not **PG-1** |

**Brief:** Artists deliver **modules**, not 200 finished buildings. Engineering assembles variants at runtime from [`StylePack`](construction_procedural_buildings_plan_v1.md) + grammar.

---

## Deliverable counts (Phase 4 minimum)

| Category | Count | Notes |
|:---|---:|:---|
| Wall segments | **10** | straight 1u, 2u; brick / concrete / wood / steel / glass variants |
| Window modules | **10** | single, double, industrial strip, arched |
| Door modules | **10** | residential, shop, warehouse, military |
| Roof modules | **10** | flat, pitched, industrial shed, warehouse sawtooth |
| Corner / prop | **10** | L-corner, parapet, trim, fence, vent, tank, transformer, AC, light |

**Total: 50 modules** (10 × 5 categories) — not 200 finished buildings.

---

## Style packs (same generator, different art)

| StylePackId | Label | Usage bias |
|:---|:---|:---|
| `style_victorian` | Victorian | Residential / commercial |
| `style_modern` | Modern | Office / residential high-rise |
| `style_industrial_west` | Industrial Western | Factory / warehouse |
| `style_industrial_soviet` | Industrial Soviet | Heavy industry |
| `style_military` | Military | Government / military |
| `style_rural` | Rural | Low density residential / ag |
| `style_colonial` | Colonial | Civic / commercial low-rise |
| `style_port` | Port District | Commercial / industrial waterfront (`style_tags: port_district`) |
| `style_railway` | Railway District | Logistics / warehouses near rail (`style_tags: railway_district`) |

Each pack references **subset** of global module IDs (reuse walls across packs where sensible). **Port/Railway** are district **style biases** via tags + [`district_style_rules_v1.schema.json`](../../tools/mcp/schemas/district_style_rules_v1.schema.json) — not separate mesh generators.

---

## Module metadata (per asset)

RON sidecar or JSON next to mesh:

```ron
(
    module_id: "wall_brick_1u",
    category: "wall",
    grid_units: (1, 1),
    snap: "floor_edge",
    style_tags: ["brick", "residential"],
    collision: "solid",
)
```

| Field | Purpose |
|:---|:---|
| `grid_units` | Procedural grid placement |
| `snap` | floor_edge / corner / roof_ridge |
| `style_tags` | StylePack filtering |
| `collision` | nav / logistics overlay |

---

## W / D / C footprint grammar (authoritative)

**W** = **Width** (cells along facade axis) · **D** = **Depth** (cells into lot) · **C** = **Corner** cell (turn or cap).

Procedural request: `width × depth` in **grid units** (1u = one module snap). Corners consume **C** tokens at L/T/O junctions.

### Facade row grammar (per floor)

```text
+----+----+----+----+
| W  | W  | W  | C  |   ← width = 4
+----+----+----+----+
| W  | D  | W  | C  |   ← door bay centered or offset per archetype
+----+----+----+----+
```

| Token | Meaning | Module category |
|:---|:---|:---|
| **W** | Wall bay (window slot optional) | `wall` |
| **D** | Door bay (ground floor default) | `door` |
| **C** | Corner / turn | `corner_prop` |
| **R** | Roof footprint cell (plan view) | `roof` |
| **.** | Yard / setback (no mesh) | — |

### Depth stacking (floors)

```text
Floor 0: W/D/C row (doors allowed)
Floor 1..N-1: W rows only (windows)
Roof plane: R grid matching width × depth outline
```

| Constraint | Rule |
|:---|:---|
| Min footprint | 2×2 cells |
| Max greybox | 12×12 cells (matches parametric scale cap) |
| Door count | ≥1 on floor 0 for `usage != industrial_warehouse` |

Ship reference art: `assets/configs/buildings/_footprint_grammar_reference.png` + token table in this doc.

---

## Development tier (MCP + index — authoritative)

**Policy:** [`plan_module_kit_production_tier_v1.md`](plan_module_kit_production_tier_v1.md)

| Tier | Use | StylePack |
|:---|:---|:---:|
| **`smoke`** | MCP G0–G5 harness (`kit_greybox_*` legacy) | **Ignored** |
| **`lod0`** | PG-2 silhouette assembly (`kit_lod0_*`) | Explicit load only |
| **`production`** | Player-visible modules (`kit_production_*`) | Default |

**MCP smoke cubes are not kit art.** They must not satisfy this validation contract for StylePack.

---

## LOD / placeholder policy

| LOD | Art | When | `development_tier` |
|:---|:---|:---|:---|
| **LOD0** | Archetype-correct silhouette (not cheat cubes) | PG-2 milestone | `lod0` |
| **LOD1** | Textured modules | post organic growth UX | `production` |
| **LOD2** | Hero buildings only | manual catalog overrides | catalog override |

**Do not** block coder PG-1 on textured art.

**Do not** promote MCP pipeline smoke (`kit_greybox_*`) as LOD0 or production — see production tier plan.

---

## Validation contract (per module — inbound Validation MCP)

Registration in `_module_index.ron` for **`lod0` and `production`** only (smoke rows excluded from StylePack):

| Check | Rule | `lod0` | `production` |
|:---|:---|:---:|:---:|
| **Pivot** | Bottom-center unless `snap: roof_ridge` | required | required |
| **Grid** | 1 m module grid; integer × 1m multiples | required | required |
| **Scale** | Real-world meters | required | required |
| **Silhouette** | Archetype profile matches id (no slab-as-sawtooth) | required | required |
| **Poly budget** | ≤ 2k tris LOD0 | required | required |
| **PBR** | Tileable Albedo+Normal+Roughness+AO | `pbr_status: deferred` OK | **required** |
| **UVs** | No overlapping islands | if textured | required |
| **Collision** | Box or convex metadata | recommended | required |
| **Naming** | `module_id` ∈ inventory § below | canonical only | canonical only |

**Tooling:** `validate_asset_report` (tier-aware) + `validate_glb_asset` header/verts — **not** verts-only pass for non-smoke tiers.

---

## Textures (tileable PBR — not unique baked)

| Use | Resolution |
|:---|:---|
| Standard modules | **512×512** or **1024×1024** |
| Landmarks / hero | **2048+** (manual catalog only) |

**Prefer 5 tileable families × engine variation** over 50 unique baked textures:

```ron
// Runtime / material instance — PG-2+ render
MaterialVariation(
    hue_shift: 0.02,
    roughness_shift: 0.05,
    dirt_level: 0.15,
    wear_level: 0.1,
)
```

---

## Style guide (avoid AI-generic look)

**Do not brief with:** beautiful · cinematic · ultra detailed · photoreal · concept art

**Do brief with:** functional · architectural · engineering reference · modular · game-ready · real-world construction methods

**References (real sources — Reference MCP / designer cites):** USGS · Natural Earth · OpenStreetMap · historic building surveys · industrial architecture refs · railway/military engineering manuals — **not** AI art boards as final assets.

---

## Phase 4b extensions (after core 50 — optional)

| Category | Count target | Notes |
|:---|---:|:---|
| Foundation modules | 4–6 | slab, strip footing, basement lip |
| Utility props | 6–8 | transformer, pipe rack, substation pad |
| Road props | 4–6 | curb, lane marking kit, rail buffer stop |

Not required for PG-2 greybox; add when organic growth UX needs street readability.

---

## Module inventory (IDs — greybox manifest)

### Walls (10)

`wall_brick_1u`, `wall_brick_2u`, `wall_concrete_1u`, `wall_concrete_2u`, `wall_wood_1u`, `wall_wood_2u`, `wall_steel_1u`, `wall_glass_curtain_1u`, `wall_industrial_panel_2u`, `wall_military_bunker_1u`

### Windows (10)

`win_single_1u`, `win_double_1u`, `win_strip_2u`, `win_arched_1u`, `win_industrial_3u`, `win_shop_2u`, `win_house_1u`, `win_office_1u`, `win_bunker_slit`, `win_skylight_1u`

### Doors (10)

`door_residential`, `door_shop`, `door_warehouse`, `door_garage`, `door_office`, `door_civic`, `door_military`, `door_factory`, `door_double_shop`, `door_gate_industrial`

### Roofs (10)

`roof_flat`, `roof_pitched_gable`, `roof_pitched_hip`, `roof_shed`, `roof_sawtooth`, `roof_parapet`, `roof_metal_low`, `roof_tile`, `roof_bunker`, `roof_canopy`

### Corner / prop (10)

`corner_L`, `corner_T`, `corner_parapet`, `prop_fence`, `prop_light`, `prop_vent`, `prop_tank`, `prop_transformer`, `prop_ac`, `prop_chimney`

---

## Acceptance (designer sign-off)

| # | Criterion | Status |
|:---:|:---|:---:|
| D1 | Module manifest lists all 50 IDs across 7 style packs | ☑ |
| D2 | Each of 5 categories has exactly 10 entries | ☑ |
| D3 | Victorian, Modern, Industrial Western fully mapped | ☑ |
| D4 | W/D/C footprint grammar documented | ☑ |
| D5 | No 200-building mesh request | ☑ |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-02 |

---

## Handoff to coder

- Mesh paths: `assets/meshes/buildings/modules/<category>/`
- Registry: `assets/configs/buildings/style_packs/*.ron`
- Index: `assets/configs/buildings/_module_index.ron`

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-27 | Initial kit charter |
| v1.1.0 | 2026-06-02 | **PASS** — 50-module inventory, W/D/C grammar, unblocks PROC-PG-2 |
| v1.2.0 | 2026-06-02 | Validation contract, textures, MaterialVariation, style guide, Phase 4b, port/rail tags |
| v1.3.0 | 2026-06-02 | `development_tier` smoke/lod0/production; smoke ≠ StylePack art |
