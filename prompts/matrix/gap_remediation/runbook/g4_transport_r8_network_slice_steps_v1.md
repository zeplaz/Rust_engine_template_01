# G4 — Transport **R8** network slice (snapshot / hydrate) `v1`

> **STATUS:** **Executed (JSON slice)** — hybrid **binary** world body + **M5** aggregation remain **open** per [`../../serialization/serialization_hybrid_migration_matrix_v1.md`](../../serialization/serialization_hybrid_migration_matrix_v1.md).

**Source of truth:** [`../../serialization/serialization_hybrid_migration_matrix_v1.md`](../../serialization/serialization_hybrid_migration_matrix_v1.md) · **R8** row [`../../transport/road_rail_migration_matrix_v1.md`](../../transport/road_rail_migration_matrix_v1.md) · code `src/systems/transport/snapshot.rs`, `persistence.rs`.

**Authoring order:** [`../../transport/runbook/r9_authoring_bake_order_steps_v1.md`](../../transport/runbook/r9_authoring_bake_order_steps_v1.md) — baked **`control_points`** must match designer intent; **never** persist a **lexicographically sorted** vertex list unless the sort is an explicit, documented transform.

---

## 1. Scope

| In scope | Out of scope (separate rows / packs) |
|:---|:---|
| Serialize `TransportNetworkSnapshot` (JSON dev slice; binary TBD) | Full **M5** world snapshot orchestration |
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

## 3. Atomic steps (**Applied** for JSON slice)

| Step | Result |
|:---|:---|
| **G4-T01** | `persistence.rs`: JSON parse + schema gate; `TransportNetworkPersistencePlugin` + `LoadTransportNetworkSnapshotFromDisk` hydrates world + updates `TransportLastHydratedSnapshot`. |
| **G4-T02** | ** Save source of truth:** editor bake sets `TransportLastHydratedSnapshot`; `transport_network_snapshot_from_world` rebuilds DTO from runtime (edge meta stores head/tail/CP). `transport_network_snapshot_save_json_path` for tools. |
| **G4-T03** | Tests: `g4_fixture_load_hydrate_nonempty_topology`, `g4_round_trip_json_file_from_manifest`, `hydrate_then_snapshot_from_world_round_trip`; fixture `assets/test_fixtures/transport/network_chain_v1.json`. |
| **G4-T04** | Matrix: **R8** → **partial**; hybrid matrix + this file + transport cross-links updated. Full **Applied** when wave **S** owns transport bytes inside world save. |

---

## 4. Acceptance

- [x] **Load test:** file → **`hydrate_transport_from_snapshot`** → nonempty topology (`g4_*` tests).
- [x] Matrix **R8** documents **partial** + remaining **G4 wave S / M5** blockers.
- [x] **R9** bake order runbook cross-linked; no hidden tile-sort in save path.
