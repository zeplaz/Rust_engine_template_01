# G4 — Transport **R8** network slice (snapshot / hydrate) `v1`

> **STATUS:** Companion to **G4** stub pack [`g4_serialization_stubs_steps_v1.md`](g4_serialization_stubs_steps_v1.md). Defines the **first concrete domain slice** that should use the hybrid save story once **wave S** touches transport: **`TransportNetworkSnapshot`** ↔ runtime **`hydrate_transport_from_snapshot`**.

**Source of truth:** [`../../serialization/serialization_hybrid_migration_matrix_v1.md`](../../serialization/serialization_hybrid_migration_matrix_v1.md) · **R8** row [`../../transport/road_rail_migration_matrix_v1.md`](../../transport/road_rail_migration_matrix_v1.md) · code `src/systems/transport/snapshot.rs`, `bake.rs`.

**Authoring order:** [`../../transport/runbook/r9_authoring_bake_order_steps_v1.md`](../../transport/runbook/r9_authoring_bake_order_steps_v1.md) — baked **`control_points`** must match designer intent; **never** persist a **lexicographically sorted** vertex list unless the sort is an explicit, documented transform.

---

## 1. Scope

| In scope | Out of scope (separate rows / packs) |
|:---|:---|
| Serialize `TransportNetworkSnapshot` (JSON/RON/section of binary blob TBD by hybrid matrix) | Full **M5** world snapshot orchestration |
| Load → **`hydrate_transport_from_snapshot`** on main thread; **no** gameplay rule invention in loader | Phase II lane combinatorics (**T-LANE-001**) |
| Deterministic round-trip tests for DTO + hydrate | Tilemap-only road visuals without graph |

---

## 2. DTO contract (R8)

- **`schema_version`**: `TRANSPORT_NETWORK_SCHEMA_V1` (see Rust `snapshot.rs`).
- **`nodes`**: stable `key` + `position` (authoritative for hydrate validation).
- **`edges`**: `id`, `head` / `tail` node keys, `successors`, **`control_points`**, **`profile`**, **`allowed_agents`**.
- **Determinism:** hydrate clears topology / field / edge directory then rebuilds; same bytes → same graph.

**Optional future fields (not required for first G4 promotion):**

- `authoring_sessions` / stroke metadata if editor saves **unbaked** markers (`placement_seq` per tile).

---

## 3. Atomic steps (G4-SNN template — fill when executing)

When **BQ-110** + hybrid matrix name this slice:

1. **G4-T01** — Register transport snapshot loader next to owning plugin (`TransportSimulationPlugin` or serialization boundary); version gate on `schema_version`.
2. **G4-T02** — Save path: emit `TransportNetworkSnapshot` from runtime **or** from editor bake only (document which is **source of truth** at save time).
3. **G4-T03** — Test: fixture JSON/RON → hydrate → assert `TransportTopology` / `TransportEdgeDirectory` + cost pipeline (existing transport tests extended).
4. **G4-T04** — Update [`../../serialization/serialization_hybrid_migration_matrix_v1.md`](../../serialization/serialization_hybrid_migration_matrix_v1.md) + **R8** §4 status (`Partial` / `Applied`) with owner.

---

## 4. Acceptance

- [ ] One **load test** proves file → **`hydrate_transport_from_snapshot`** → nonempty graph when fixture expects it.
- [ ] Matrix **R8** blockers list **G4 wave S** completion for this slice explicitly.
- [ ] **R9** bake order runbook cross-linked; no hidden tile-sort in save path.
