# Dual-view art home — APS / Blender → PixelWorld `v1`

| Field | Value |
|:---|:---|
| **Program** | **DESIGN-DUAL-VIEW-ART-HOME-001** |
| **Date** | 2026-10-17 |
| **Owner** | `@designer-mcp` · `@coder-mcp` · `@orchestrator-mcp` |
| **Status** | **SIGNED** |
| **Parent** | [`plan_dual_view_pixel_world_v1.md`](plan_dual_view_pixel_world_v1.md) |

## Rule

**All ship art (`ship: true`) targets PixelWorld3D** unless the asset is explicitly tagged `presentation: tactical_icon`.

| Lane | Consumer | Toolchain |
|:---|:---|:---|
| Landscape / environment (v1 first) | PixelWorld | Landscape grammar · keyframe bake · atlas pack · `terrain_honesty_lint` |
| Building / module kits (v2) | PixelWorld | Geometry jobs · GLB promote · registry |
| Tactical icons / symbols | TacticalFlat2D | Thin symbol atlas only — no module kits |

## Bake spine

```text
AssetSpec → MCP/CLI (seeded) → validate → promote → Bevy registry
  → RepresentationResult band → PixelWorld consumer
```

`bake_source: keyframe_pack` when `ship: true`. No AI-generated final art.

## APS / MCP checklist (landscape/env first)

| # | Gate | Tool / witness | Status |
|:---|:---|:---|:---|
| 1 | Landscape grammar presets validate | MCP `landscape_grammar_presets_witness` → `debug_runs/mcp_landscape_grammar_preset_batch_live.json` | **GREEN** (2026-10-17) |
| 2 | Sign witness refreshed | `debug_runs/mcp_landscape_grammar_sign_live.json` | **GREEN** |
| 3 | Ground cover / biome / veg packs named for PixelWorld | APS tags `presentation: pixel_world` + LG-5 veg-modulated stamps | **WIRED** (v5) |
| 4 | Promote path documents `presentation: pixel_world` vs `tactical_icon` | this charter + promote notes | **SIGNED** |
| 5 | Runtime v1 landscape relief in play hole | `PixelWorldLandscapeIntent` → tile_world_fallback height/slope | **WIRED** |
| 6 | Building / module kits | ship iso atlases registered + stamped PixelWorld-only; 3D mesh wave still deferred | **WIRED** iso / mesh deferred |
| 7 | `terrain_honesty_lint` on GPU terrain path | MCP `terrain_honesty_lint_tool` → `debug_runs/terrain_honesty_lint_live.json` | **RUN** (0 err / 1 warn GpuTilemap) |

## Reject

- Shipping full APS meshes onto the tactical map
- Using WorldPreview as play art preview authority
- AI-generated final art

## Runtime consumer (engine)

```text
WorldPresentationMode::PixelWorld3D
  → PixelWorldLandscapeIntent { landform_height_slope, relief_emphasis }
  → effective_play_map_pixel_mode (OR with ops-strip PlayMapPixelMode)
  → tile_world_fallback raster into SimulationMapTexture (same RTT)

ArtPresentationTarget (promote tag)
  presentation: pixel_world   → PixelWorldLandscapePackIndex + LG-5 atlas stamps
  presentation: tactical_icon → strategic_icon / symbol lane only

PixelWorld3D RTT paint
  landform relief (height/slope) + LG-5 topology atlas stamps (real PNG)
  + veg/burn/scar/regrowth row preference from VegetationField / ActiveBurn / SuccessionState
  → SimulationMapTexture (same hole; no WorldPreview; no parallel extract)

VoxelMassing columns → PixelWorld only (tactical keeps flat footprint tokens)
```

## Engine types

| Type | Path |
|:---|:---|
| `ArtPresentationTarget` | `src/gui/art_presentation_target.rs` |
| `PixelWorldLandscapePackIndex` | same — 10 presets + `landscape_lg5_expanded_v1` |
| `PixelWorldLandscapeConsume` | band = `LandscapeAtlasStamp` when stamps queue |
| `landscape_chunk_atlas_stamp` | LG-5 UV → tile stamps |
| `queue_pixel_world_landscape_atlas_stamps` | `tile_world_fallback.rs` |
| `WorldPresentationMode` | `src/gui/world_presentation_mode.rs` |

## Promote / register (MCP)

When promoting landscape/env art for Simulation:

1. Set `_meta.presentation` = `"pixel_world"` (schema: `landscape_grammar_v0`).
2. Ship presets live under `assets/configs/landscape/presets/` + `_preset_index.json` `ship_presets`.
3. Engine seeds [`PIXEL_WORLD_LANDSCAPE_SHIP_PRESETS`](../gui/art_presentation_target.rs) into `PixelWorldLandscapePackIndex` at startup.
4. Extra packs: call `PixelWorldLandscapePackIndex::register_pack` (or extend the const list + index together).
5. Building/module kits: still **DR-PIXEL-WORLD-APS-MESH** mesh band — do not dump onto Tactical.

`bake_source: keyframe_pack` when `ship: true`. No AI final art.
