# Construction ownership & frame order

**Round 3:** `CONSTRUCTION-R3-C03`, `CONSTRUCTION-R3-H02`  
**Invariants:** [`construction_invariants.md`](construction_invariants.md)

## Who may mutate what

| State | Owner | Mutators |
|-------|--------|----------|
| Active tool | `ActiveBuildTool` | Toolbox egui, cycle key, escape (session-aware) |
| Road/rail path | `ActiveRoadPlacement` / `ActiveRailPlacement` | Road/rail input systems only |
| Zone paint | `ActiveZonePaint` | `zone_paint_input_system` |
| Building ghost | `BuildGhostState` | Building pick/refresh (not zone/road/demolish) |
| Pending queue | `PendingConstructionQueue` | Queue systems, confirm drain |
| Plan queue | `ConstructionPlanQueue` | Road/rail commit enqueue; validate/execute systems |
| Transport / roads executed | `ExecutedRoadNetwork` + transport resources | **`execute_construction_plans_system` only** (undo via `history`) |
| Sites | `ConstructionSite` entities | **`CommitConstructionSiteEvent`** → strategic commit system |
| Zones | `Zone` components | Zone confirm (`spawn_zone_at_tile`) |
| Intersections | `IntersectionRegistry` | Road/rail execute (Round 3) |

## `BuildPlanningPlugin` order (Update)

1. Tool ↔ strip sync  
2. Build mode / escape  
3. Zone tool sync  
4. Witness boards (live, finish, phase2, round2, round3, operational)  
5. Road: width → preview → build preview → input  
6. Rail: sync → preview → build preview → input  
7. Zone paint input  
8. Undo input, early phase tick  
9. Queue intents, building pick, demolish pick, validation, queue, confirm, ghost entity sync  
10. Plan validate → execute  

**PostUpdate:** finalize site history records (undo resolution).

## Egui pass

Toolbox, tool hints, road popup, road ghost, rail ghost, zone ghost, footprint overlay, phase labels — **draw only**; no commit in egui except explicit buttons calling commit helpers.

## Viewport boundary (Round 3 target)

Construction writes **`ConstructionVisualRequest`** (planned); viewport owns camera, hole latch, and `RepresentationResult` extraction. Construction must not set map viewport authority flags.
