# PLAN-WATER-DEPTH-TRUTH-v1 — water is water

| Field | Value |
|:---|:---|
| **Id** | **PLAN-WATER-DEPTH-TRUTH-v1** |
| **Status** | **ACTIVE** — W0–W4 landed. **W5 salinity DEFER** (stub [`WorldWaterSalinity`](../systems/water/salinity.rs)). |
| **Date** | 2026-11-16 |
| **Purpose** | One physical water truth: bed + depth (+ flux in ROI). River/lake/ocean are look/regime labels only. Gameplay never gates on `WaterClass` or the render catalog. |
| **Related** | [`plan_dam_flood_benchmark_v1.md`](plan_dam_flood_benchmark_v1.md) DF-2/DF-3 presentation debt lands here. |

## Model

```text
⊚bed_elevation + depth_m (+ qx/qy in ROI) ═▶ sole water physics
   ▸ wet := depth_m >= WET_DEPTH_M
   ▸ look := f(depth, speed, coast, salinity?)   [presentation only]
⛔ WaterClass / WaterSurfaceVisualCatalog as gameplay wet authority
```

Dynamic solve stays **ROI** (DamFlood D1). Outside ROI: [`WorldStaticWaterDepth`](../systems/water/static_depth.rs) from world-gen bootstrap.

## W0 — Consumer audit (gameplay wet readers)

| Site | Was | Target |
|:---|:---|:---|
| [`systems/water/probe.rs`](../systems/water/probe.rs) `WaterTileProbe` | runtime → catalog | field → runtime → static depth → catalog last |
| [`construction/build_interaction.rs`](../construction/build_interaction.rs) | runtime → catalog `tile_kind` | `tile_is_wet` |
| [`construction/dam_barrier.rs`](../construction/dam_barrier.rs) | `tile_is_water_prefer_runtime` | `tile_is_wet` |
| [`strategic/settlement/town.rs`](../strategic/settlement/town.rs) | runtime → catalog | `tile_is_wet` / static depth |
| [`strategic/settlement/faction_step.rs`](../strategic/settlement/faction_step.rs) | same | same |
| World-gen town place ([`world_generator_enhanced.rs`](../terrain/generation/world_generator_enhanced.rs)) | catalog `tile_kind` | static depth wet |
| Catalog / overlay / PixelWorld | look + accidental truth | **look only** |

## Slices

| Id | Work | Exit |
|:---|:---|:---|
| **W0** | This charter + audit | **ADOPTED** |
| **W1** | `tile_is_wet` + migrate probes | **ADOPTED** — catalog last resort only |
| **W2** | `WorldStaticWaterDepth` at world-gen | **ADOPTED** |
| **W3** | Overlay/PixelWorld look from depth | **ADOPTED** — exclusive look; PixelWorld `ground+depth` |
| **W4** | DamFlood author bed+depth | **ADOPTED** — corridor depth, dry banks depth=0 |
| **W5** | Salinity scalar | **DEFER** — stub resource only; no enum fork · **DR-WATER-SALINITY** |

**Deferrals:** [`plan_deferral_registry_v1.md`](plan_deferral_registry_v1.md) — **DR-WATER-SALINITY**.

## Success

- Bed + depth (+ flux) answer wet/flood/block for every gameplay system.
- Removing catalog from a probe does not change build/unit wet answers when static/runtime exist.
- DamFlood: one depth corridor, not a class-painted bay.
