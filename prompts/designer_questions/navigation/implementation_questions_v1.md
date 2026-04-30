# Navigation — implementation questions `v1`

**Pair with:** [`spec/README.md`](spec/README.md), `pathfinding_hierarchical_v1.md`, `prompts/designer_questions/terrain_world/chunks_streaming_v1.md` (chunk boundaries).

## Representation

1. **RoadGraph** / **RailGraph**: separate resources or one `TransportGraph` with `EdgeKind`?
2. **Flow field** storage: dense grid per region vs sparse; resolution ladder per LOD tier (explicit table 📎).
3. **Dynamic obstacles**: bitmask layer, graph edge weight only, or both — when to rebuild field?

## LOD & handoff

4. Path request carries **min_LOD** + **max_LOD**? Who upgrades on stuck detection?
5. **Rail block / signal** state: simulated only at LOD ≥ N — document N.

## Multiplayer

6. **Client proposal** payload: polyline + cost estimate + `request_id`? Max proposals per client per tick.
7. Server **reject** reasons → telemetry + player feedback (optional)?
8. **Anti-cheat**: ignore client paths in competitive modes until milestone M?

## Performance

9. **Hierarchical** region abstraction: graph of regions — same as chunk grid or coarser?
10. Path **cache** TTL per entity class (vehicle vs pedestrian)?

## Active modules (`src/systems/navigation/` — code-synced)

11. **`nav`**, **`potental_feild_nav`**, **`road_vehicles_motion`** — document intended split (graph vs field vs kinematics); avoid duplicating vehicle motion with `entities/vehicles/runtime.rs` without a contract.
12. **Typo debt:** `potental_feild_nav.rs` — rename plan vs stable `mod` path for downstream imports.
13. **Queries:** which components mark “navigable” road entities today (mesh vs grid) 📎?

## Repo hygiene

14. Rename `potental_feild_nav.rs` → `potential_field_nav.rs` when implementing; align module paths.
