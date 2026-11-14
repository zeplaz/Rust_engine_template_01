# PT-5-A follow-on — burning atlas bake (non-blocking)

| Field | Value |
|:---|:---|
| **Id** | **PT-5-A-FOLLOWON** |
| **Parent** | [`plan_procedural_building_tiles_production_v1.md`](plan_procedural_building_tiles_production_v1.md) PT-5 · FIRE-ENTITY-1 |
| **Status** | **CHARTERED / OPEN** (MCP art lane) |
| **Blocks play UX?** | **No** — Option B footprint wash + resolver keys already ship |

## Goal

Bake `burning_00`…`07` into production building atlases and let `TileVariantResolver` advance frames from `SiteFireState` / heat. Engine already lists keys in [`_variant_catalog.ron`](../../assets/configs/buildings/_variant_catalog.ron) and stamps wash from `SiteFireState`.

## Exit

1. Four production atlases include burning UV rows (`tile_batch` / keyframe bake).
2. Resolver picks `burning_*` when `fire_heat >= heat_threshold` and atlas has the key; else wash fallback.
3. MCP validate + registry promote green.

## Owners

`@designer-mcp` matrix → `@coder-mcp` bake → `@coder` resolver frame tick if needed.

## Rejected

- Blocking BUILD-UX / CLICK-UX on atlas bakes
- 3D particle fire as primary building read
