# R9 — Authoring bake order & polyline semantics `v1`

> **STATUS:** Step pack for **map editor → transport graph** fidelity. **Does not** replace [`../road_rail_migration_matrix_v1.md`](../road_rail_migration_matrix_v1.md) **R9** row; use to promote **Partial** with traceable DoD.

**Pair:** [`../../../designer_questions/transport/transport_code_implementation_plan_v1.md`](../../../designer_questions/transport/transport_code_implementation_plan_v1.md) · [`../../../designer_questions/transport/transport_editor_ux_risk_v1.md`](../../../designer_questions/transport/transport_editor_ux_risk_v1.md) (**T-GHOST-001**) · G4 slice [`../../gap_remediation/runbook/g4_transport_r8_network_slice_steps_v1.md`](../../gap_remediation/runbook/g4_transport_r8_network_slice_steps_v1.md).

---

## 1. Problem statement (why lexicographic bake is wrong)

Sorting markers by `(tile_x, tile_z)` before chaining edges **destroys designer intent**:

- A road drawn **east → west** becomes **west → east** if tiles sort the other way.
- **Branches** and **self-crossing** polylines cannot be represented as a single sorted chain; you need **explicit topology** (multiple edges / nodes), not a forced total order on tiles.
- **R8** expects meaningful **`control_points`**. Lexicographic order can invert geometry and break **deterministic hydrate → nav export** assumptions.

**Repo fix (executed):** `MapEditorRoadMarkerV1::placement_seq` + `bake_snapshot_from_ordered_tile_markers` (authoring order; only **consecutive** duplicate tiles collapsed).

---

## 2. Target semantics (R9 “good”)

| Topic | Rule |
|:---|:---|
| **Order** | Polyline vertex order = **placement order** until splines land; later = spline **control-point** order. |
| **Duplicate tile** | Consecutive duplicate clicks **collapse** to one vertex; non-consecutive revisits = **valid** (loop) if hydrate/topology allows — today chain edges only; loops need **explicit graph** (Phase II / multi-edge). |
| **Ghost vs bake** | Per matrix **T-GHOST-001**: preview does not write **R8** slice until **Confirm bake**. |
| **Splines (future)** | Replace “click tile sequence” with **control point list** + snap; bake samples polyline into **`control_points`** for **R8** (matrix: persistence stores CPs). |
| **Undo/redo** | **R9 Applied** blocker per matrix — `placement_seq` must be restorable or replaced by stable **command stack**; document in editor matrix. |

---

## 3. Engineering checklist (promotion helpers)

- [x] **placement_seq** monotonic per editor session; reset on **Enter editor** (`MapEditorRoadPlacementSeq`).
- [x] **Bake** sorts markers by **`placement_seq`**, not by tile.
- [x] Unit test: authoring order **differs** from lexicographic order (`bake_preserves_authoring_order_not_lexicographic`).
- [ ] **Spline tool** + sampling → dense `control_points` (R9 proper).
- [ ] **Branches**: multiple outgoing edges at a node — data model + bake (aligned with **R2/R3** / Phase II).
- [ ] **Undo** stack spec + integration tests before claiming **R9 Applied**.

---

## 4. DoD (this pack, code spine)

1. Bake builds the same polyline the designer “drew” in click order (regression test locked).
2. Docs point here from map editor module header + transport implementation plan.
3. G4 slice runbook defines how **`TransportNetworkSnapshot`** is written/read without re-sorting vertices.
