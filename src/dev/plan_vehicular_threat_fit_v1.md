# PLAN-VEHICULAR-THREAT-FIT-v1 — conflict reach already on the overlay

| Field | Value |
|:---|:---|
| **Id** | **PLAN-VEHICULAR-THREAT-FIT-v1** |
| **Status** | **CLOSED as DEFER** — no `VTH-*` slices |
| **Date** | 2026-09-27 |
| **Owner** | `@planner` (this file only). `@coder` does not implement from it. |
| **Obeys** | Wave B **ATR-2** in `src/dev/plan_actualize_v1.md` (queued, not shipped). **MAI-CUT-INFLUENCE**. |
| **Out of scope** | Second influence grid, frag grid, steering vector, tactical brain, `Transform` write, third artillery-danger writer, second camera, screen-space tile batch. Turrets, ram plates, tires, armor, flee / intercept / ram. |

Nothing new survives. Conflict reach already lives on `ChunkStrategicOverlay` (`threat`, `artillery_danger`). Two writers stamp it: `apply_zones_to_strategic_overlays_system` (`ZoneKind::Fire`, weight from `zone_falloff`) and `apply_site_zone_emitters_to_overlays_system` (`ZoneEmitter.fire_control_strength` on the site footprint). A BFS that clears a grid every AI tick and adds `I_0 / (1+d)^2` is a third writer and a second store. **MAI-CUT-INFLUENCE** already rejects that store; `site_score` samples one overlay cell. Edge residual is `EdgeFieldState.danger`, sole writer `transport_field_integrate` (`danger -= 0.2 * dt`, clamp 0, beside congestion, damage, heat). ATR-2 is the only next write of that field (sample overlay threat at the edge anchors, `danger = max(sampled, decayed)`, rebuild folds through `disruption_after_cell_heat`). This file does not re-specify ATR-2. Power hits are `damage_power_line_segment` on `PowerLineSegmentHealth` and do not write `danger`. Paths are `bfs_path_to_goals` on transport edges; `logistics_path_dijkstra` is a separate cell search. AI does not write `Transform`. Map shade is `ZoneShadeToggle` plus `sync_world_map_town_names_and_zone_shade` (`WorldMapZoneTint` over `DistrictBook`). The play map is one terrain sprite. `zone_falloff` is already `1 - dist/radius` inside the disk. A narrow swap of that formula, or a per-tick overlay clear, would reweight supply, sensor, and control stamps in the same pass. A narrow recolor of district rings would paint districts, and a per-cell quad batch would be a second map. No row is **FIT** or **NARROW**. No function changes. No test filter.

```text
⊚ChunkStrategicOverlay ═▶ ⊨ per-cell threat, artillery_danger
   ⊚apply_zones_to_strategic_overlays_system ═▶ ⊨ ZoneKind::Fire stamp
   ⊚apply_site_zone_emitters_to_overlays_system ═▶ ⊨ site footprint stamp
   ⊚set_faction_threat   tests only
⊚transport_field_integrate ═▶ ⊨ EdgeFieldState.danger
   decay only today. ATR-2 is the only allowed threat copy. This program writes nothing.
⊚shot_danger_at / shot_impact_at   pure. Trace tile 0.75, else 0. They do not write the overlay.
⊚damage_power_line_segment ═▶ ⊨ PowerLineSegmentHealth
⊚bfs_path_to_goals ═▶ transport-edge path
⊚logistics_path_dijkstra ═▶ cell search only
⊚sync_world_map_town_names_and_zone_shade ═▶ ⊨ WorldMapZoneTint  ⊰ DistrictBook
   ⊚ZoneShadeToggle ═▶ show or hide that tint
```

## Verdict

| # | Proposal | Verdict | Existing owner |
|:---|:---|:---|:---|
| 1 | `I_d = I_0 / (1+d)^2`, plateau then drop, BFS from a vehicle cell into a grid cleared each AI tick, `Grid[x,y] += I`. | **DEFER** | `zone_falloff` and `apply_zones_to_strategic_overlays_system` (`src/strategic/zones.rs`). Footprint stamp: `apply_site_zone_emitters_to_overlays_system` (`src/strategic/site/overlays.rs`). |
| 2 | Frag decay `V_t = V_start * e^(-λt)`, or `V_new = V_old * (1-δ)` with `δ=0.05`, one pass over a whole grid. | **DEFER** | `transport_field_integrate` (`src/systems/transport/mod.rs`). ATR-2 owns the next edit. |
| 3 | Resource `IsometricGrid`. | **DEFER** | One terrain sprite. District shade is `WorldMapZoneTint`. |
| 4 | Resource `InfluenceGrid`. | **DEFER** | `ChunkStrategicOverlay.threat` and `artillery_danger`. Second heatmap: **MAI-CUT-INFLUENCE**. |
| 5 | Resource `FragGrid`. | **DEFER** | `EdgeFieldState.danger` on `TransportFieldStore`. |
| 6 | Component `GridTransform`. | **DEFER** | AI does not write `Transform`. Occupancy is the overlay and the transport graph. |
| 7 | Component `ThreatEmitter`. | **DEFER** | `Zone` and `ZoneEmitter`. |
| 8 | Component `ModuleDestructionEvent`. | **DEFER** | `damage_power_line_segment` (`src/construction/power_lines/damage.rs`) on `PowerLineSegmentHealth`. |
| 9 | Component `TacticalBrain`. | **DEFER** | `bfs_path_to_goals` (`src/economy/logistics/routes.rs`). |
| 10 | `InfluencePropagationSystem`. | **DEFER** | The two overlay writers in row 1. A vehicle flood would be a third artillery-danger writer. |
| 11 | `FragMapLoggingSystem`. | **DEFER** | Overlay threat and `EdgeFieldState.danger` are the live fields. |
| 12 | `FragMapDecaySystem`. | **DEFER** | `transport_field_integrate`. |
| 13 | `VehicleSteeringSystem` (movement vector). | **DEFER** | `bfs_path_to_goals`. Cell search: `logistics_path_dijkstra` (`src/systems/navigation/logistics_floodfill.rs`). |
| 14 | `TacticalDebugRenderSystem`, iso screen formula, colored quads, `DEBUG_MODE_TACTICAL`. | **DEFER** | `ZoneShadeToggle` and `sync_world_map_town_names_and_zone_shade` (`src/render/world_map_network_mesh.rs`). |

No **FIT** row: no function to change, no test filter.

## Must not change

- `InfluenceGrid`, `FragGrid`, and `IsometricGrid` stay absent.
- `TacticalBrain`, `VehicleSteeringSystem`, `GridTransform`, and any AI write of `Transform` stay absent.
- Artillery-danger / threat writers stay the zone pass and the site-emitter pass. `set_faction_threat` stays tests-only.
- This program leaves `transport_field_integrate`, `shot_danger_at`, and `shot_impact_at` untouched. ATR-2 remains the threat copy into `danger`.
- No work lands in `src/strategic/shot_pulse.rs`, `src/render/fire_vfx/emit.rs`, `src/render/tile_world_fallback.rs`, `src/engine/states.rs`, `src/entities/production/power/systems.rs`, `src/strategic/faction_step.rs`, `src/systems/transport/mod.rs`, `src/strategic/transport_bridge.rs`, `src/economy/logistics/routes.rs`, `src/economy/logistics/solver.rs`.
- No second camera, no iso screen batch, no `DEBUG_MODE_TACTICAL`. `WorldMapZoneTint` stays on district rings.
- `site_score` keeps sampling one overlay cell.

## Open questions

None. `zone_falloff` shares its pass with supply, sensor, and control, so a distance-formula edit fails the attach test. District tint geometry is `DistrictBook`, so a threat toggle on that mesh fails it too.
