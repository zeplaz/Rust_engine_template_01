---
name: tile-generation
description: Defines isometric tile state machines, variant specs, keyframe bake workflow (civ_truck spine), and atlas batch output for Rust_engine_template_01. Use when authoring tile MCP requests, Republic-style floor/state variants, or planning tile_batch/atlas pipelines — tiles are state machines, not one-off textures.
disable-model-invocation: true
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# Tile Generation Skill

Tiles are **state machines** — not standalone textures. Deterministic stills from structured specs + **proven utils spine**.

## When to use

- Authoring tile variant specs (base, damage, power, fill, lighting)
- Planning or implementing `tile_batch` / `tile-atlas-pack` MCP tools
- Batch atlas packing for Bevy `TileAtlasRegistry` / map stamp
- Mapping sim state → `variant_key` (see `_variant_catalog.ron`)

## Primary rule

> STATE → KEYFRAME STILLS → TILEMAPGEN → ATLAS → ENGINE. LLM sets parameters; Blender + tilemapgen render.

**Ship art forbids** using headless `tile_ortho_bake` alone. See [`design_tile_bake_spine_convergence_v1.md`](../../docs/archive/2026-06-src-dev/plans/design_tile_bake_spine_convergence_v1.md).

## AGENT-LANG ritual (attach [agent-lang](../agent-lang/SKILL.md))

**DSM node WRK→ATL** · production = `bake_source: keyframe_pack` when `ship: true`

```text
BLANG:PRE → keyframe PNG folder → tile-atlas-pack → BLANG:WIT → register $ref:_tile_atlas_index.ron
```

| Status | Sym |
|:---|:---|
| Production spine | 🟢 `keyframe_pack` |
| CI smoke only | 🧊 `smoke_ortho_headless` |
| Ship blocked | 🔴 ortho-only without keyframes |

**Refs:** `$ref:docs/archive/2026-06-src-dev/plans/design_tile_bake_spine_convergence_v1.md`

## Quick workflow (production)

1. Attach **`mcp-production-rules`** — batch + grid + deterministic required.
2. Read [reference.md](reference.md) for enums and engine swap contract.
3. Assembly in Blender with **`utils/Light_keysshotsetup.blend`**.
4. Export variants via **`utils/keyframe_render.py`** → PNG folder.
5. **`tile-atlas-pack <folder> [-pk]`** (wraps `utils/tilemapgen`).
6. Register `_tile_atlas_index.ron` → engine `TileAtlasRegistry`.

## `bake_source` (tile_batch_v1)

| Value | When |
|-------|------|
| **`keyframe_pack`** | **`ship: true`** — MCP packs existing PNGs only |
| **`smoke_ortho_headless`** | CI / pytest (`RUST_ENGINE_TILE_DRY_RUN=1`) |

## Tile variant axes

| Axis | Examples |
|------|----------|
| `variant_key` | `clean_day`, `clean_night_on`, `damaged_night_on`, `burning_00`…`07` |
| `base` | wood, stone, concrete (terrain batches) |
| `state` / `damage` / `power` / `fill` / `lighting` | See variant_set_v1 |

## Vehicles vs buildings

| Asset | Rotation | States |
|-------|----------|--------|
| Vehicles (`civ_truck_01`) | **8 facings** in one sheet | empty/full, day/night |
| Buildings (production) | **One iso** + state rows | day/night, damage, fire frames |

## BUILD-GRAMMAR◈ → WEATHERING◈ (v0)

Building grammar `age` bands and `ARCH-DNA.A` map to tile state-machine rows:

| Grammar | Tile `variant_key` / axis |
|:---|:---|
| `age.bands[].variant_tags` | `clean` · `weathered` · `abandoned` · `damaged` |
| `arch_dna.A = weathered` | damage row + `clean_night_on` siblings |
| `district_styles.style_tags` | base material axis (steel · brick · …) |

```text
SNAP★ → grammar age pick → variant_set_v1 → keyframe bake → atlas
```

Refs: [`arch_build_grammar_v0_baseline_v1.md`](src/dev/arch_build_grammar_v0_baseline_v1.md) · [`industrial_warehouse_v1.ron`](assets/configs/buildings/grammars/industrial_warehouse_v1.ron) `age` section.

## MCP tools (shipped)

| Tool | Role |
|------|------|
| `tile_atlas_pack_tool` | **Production pack** — tilemapgen |
| `tile_batch_run` | **CI/smoke** ortho OR **keyframe_pack** register-only |
| `tile_batch_validate` | Enforces `ship` → `keyframe_pack` |

## Repo status

| Item | Status |
|------|--------|
| keyframe → tilemapgen spine | **Authoritative** — [`utils/LEGACY_ART_PIPELINE_README.md`](../../utils/LEGACY_ART_PIPELINE_README.md) |
| `tile_ortho_bake` headless | **Smoke/CI** until civ-truck parity |
| Engine map stamp | **Shipped** — `TileAtlasRegistry` + PT-4 resolver (planned) |

## Additional resources

- Convergence: [`docs/archive/2026-06-src-dev/plans/design_tile_bake_spine_convergence_v1.md`](../../docs/archive/2026-06-src-dev/plans/design_tile_bake_spine_convergence_v1.md)
- Production program: [`docs/archive/2026-06-src-dev/plans/plan_procedural_building_tiles_production_v1.md`](../../docs/archive/2026-06-src-dev/plans/plan_procedural_building_tiles_production_v1.md)
- Pipeline: [mcp-asset-pipeline](mcp-asset-pipeline/SKILL.md)
- Full enums: [reference.md](reference.md)
