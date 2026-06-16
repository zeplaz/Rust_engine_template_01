# Navigation — pathfinding LOD & handoff `02`

**Pair:** [`../pathfinding_hierarchical_v1.md`](../pathfinding_hierarchical_v1.md), `terrain_world/simulation_lod_v1.md`.

## Path request

- Carry **min_LOD / max_LOD** (📎 defaults) and **entity class** (heavy truck vs drone vs train).
- **Stuck detection:** who escalates LOD or re-plans — client hint vs server-only 📎.

## Rail

- **Signal / block** state simulated only at LOD ≥ N — record N in config when known (`ASK:` until set).

## Handoff invariants

- Polyline **segment endpoints** must land on legal graph nodes after chunk boundary correction.
