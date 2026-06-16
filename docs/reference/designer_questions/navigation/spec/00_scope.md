# Navigation — scope `00`

**Design deep dive:** [`../pathfinding_hierarchical_v1.md`](../pathfinding_hierarchical_v1.md)

## In scope (v1)

- **Surface transport:** roads, rails (block/signal at LOD 📎), optional off-road / coarse pedestrian abstractions.
- **Hierarchical pathfinding:** region/chunk/coarse graph handoff; align with `terrain_world/simulation_lod_v1.md`.
- **Dynamic obstacles:** integration contract with construction, damage, and chunk loads 📎.

## Out of scope / defer

- Full **navmesh** for arbitrary 3D interiors (unless merged with building tiles later 📎).
- **ACT** for every entity class day one — phase with `strategic_platforms/phased_engine_delivery_v1.md`.

## Multiplayer

- Default posture: **client proposes** path polyline + metadata; **server validates** cost, legality, and authority (detail in `03_client_server_pathing.md`).
