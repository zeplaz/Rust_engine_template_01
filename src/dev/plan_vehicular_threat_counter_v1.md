# Vehicular threat counter — v1

Counter to the pasted vehicular-threat design. This game is an industrial multi-state war sim (logistics, power, industrial output, conflict, settlements). Vehicles, turrets, ramming, tires, armor plates, and Fleeing / Intercepting / Ramming are out of scope.

## Summary

Decay and route disruption are already owned. `transport_field_integrate` is the sole danger writer and already decays linearly (`danger -= 0.2*dt`). `ChunkStrategicOverlay` already stores per-cell `threat` and `artillery_danger`, written by zone fire and site emitters. ATR-2 (not shipped; Wave B of `plan_actualize_v1.md`) already copies overlay threat into `EdgeFieldState.danger` and closes a route at `disruption >= 1`. The paste adds a third threat array, a second decay rule, a Transform writer, a second screen projection, and a destruction spike. All of that is CUT. No new resource is FIT.

```text
⊚transport_field_integrate ⊨ danger write + linear decay
⊚ChunkStrategicOverlay     ⊨ per-cell threat ∧ artillery_danger
⊚ATR-2 ⊰ overlay threat → EdgeFieldState.danger (route close at disruption >= 1)
FIT resources: none
NARROW: none
```

## Pasted rows

| Pasted row | Mark | Id |
|---|---|---|
| `I_d = I_0 / (1+d)^2` BFS-flooded into an `InfluenceGrid` cleared every tick | CUT | VTH-CUT-INFLUENCE |
| `IsometricGrid`, `InfluenceGrid`, `GridTransform`, `ThreatEmitter` | CUT | VTH-CUT-INFLUENCE |
| `InfluencePropagationSystem` | CUT | VTH-CUT-INFLUENCE |
| Frag decay `V_t = V_start * e^(-λt)` or `V_new = V_old * (1-0.05)` on a whole `FragGrid` | CUT | VTH-CUT-FRAG |
| `FragGrid`, `FragMapLoggingSystem`, `FragMapDecaySystem` | CUT | VTH-CUT-FRAG |
| `TacticalBrain`, `VehicleSteeringSystem` | CUT | VTH-CUT-STEER |
| Iso heatmap `Screen_X = (x-y)*tile_width/2`, `Screen_Y = (x+y)*tile_height/2`, colored iso quads, `TacticalDebugRenderSystem` | CUT | VTH-CUT-DEBUG |
| `ModuleDestructionEvent` as a danger / frag spike | CUT | VTH-CUT-MODULES |

### VTH-CUT-INFLUENCE

A tick-cleared `InfluenceGrid` is a third threat store beside overlay `threat` and `artillery_danger`. MAI-CUT-INFLUENCE already rejected a stored suitability heatmap. ATR-2 already moves overlay threat onto edges; a parallel BFS flood duplicates that copy. Inverse-square falloff is not given a new grid. `shot_danger_at` / `shot_impact_at` stay pure and stay without distance falloff: `src/strategic/shot_pulse.rs` is a finished slice and is not reopened here.

### VTH-CUT-FRAG

Exponential decay and `(1-0.05)` multiply are a second decay rule on a second array. Linear decay in `transport_field_integrate` already covers the tick. `FragGrid` and its log/decay systems are duplicate stores.

### VTH-CUT-STEER

No AI writes `Transform`. Movement is `bfs_path_to_goals` on graph edges. `VehicleSteeringSystem` and `TacticalBrain` do not attach to that graph.

### VTH-CUT-DEBUG

Play view is one terrain sprite plus `WorldMapZoneTint`. The isometric screen formula is a second projection. CUT the debug batcher.

### VTH-CUT-MODULES

Power-line damage does not write danger. Module destruction does not spike a frag map.

## NARROW

NARROW: none. Existing linear decay and ATR-2 already cover the math. A NARROW row would be legal only with no new resource and no edit to the files below. Nothing in the paste meets that bar.

Do not assign edits to queued files: `shot_pulse.rs`, fire `emit.rs`, `tile_world_fallback.rs`, `states.rs`, power `systems.rs`, `faction_step.rs`, `transport/mod.rs`, `transport_bridge.rs`, `routes.rs`, `solver.rs`, `construction/mod.rs`.

plan_actualize_v1.md should not gain a slice, because NARROW is none and no file outside the execution queue is named.
