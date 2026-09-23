# TRIP-VSS-FIRE-AUTHORITY — Fire / VFX authority trip stub

```text
@orchestrator-status: ACTIVE
@orchestrator-owner: @sim-steward
Program: VSS-001 · Effects: $ref:src/dev/plan_effects_system_architecture_v1.md
```

**You touched fire VFX, overlay buffers, or fire shaders.**

**One visual path per effect class.** Do not add parallel extract or duplicate LOD.

| Layer | Sole authority |
|:---|:---|
| Sim heat | `ChunkSurfaceFire` / `ChunkFireOverlay` |
| Overlay CPU tint | `sync_shared_overlay_from_simulation` → `SharedOverlayFieldBuffers` |
| Particles | `fire_vfx` → `gpu_particle_draw` → `gpu_fire_particle_raster` |
| Artist consumables | `EffectConsumableRegistry` (T4-004) — extend fire_vfx spine, don't fork |

**Before merge:**

- [ ] `@debug-intelligence` routing if dual writer suspected
- [ ] `@cleanup-intelligence` packet before deleting atmosphere stubs
- [ ] Witness: tactical_map_debug or stage5 scoped to **entry path**

**Do not:** Re-enable `WeatherFireFieldDebugOverlay` giant sprite on vfx scenes.

**T3 pick order (witness):** `$ref:debug_runs/sim_effect_fire_consumer_live.json` — **T3-002** overlay VT-4 fix (`simulation_session.rs`) **before** **T3-001** ignition funnel; SERIAL with T2+fire raster.
