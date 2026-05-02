# Transport — code implementation plan `v1`

> **Design:** [`rulebook_drafts.md`](rulebook_drafts.md) §0.2 · **ECS schedule gate:** [`../../guides/ecs_systems_schedule_runbook_v1.md`](../../guides/ecs_systems_schedule_runbook_v1.md) (**S0–S1** before **W1**).

Version: `v1.0.2`

---

## 0. Prerequisite: ECS schedule baseline

**S0 + S1** are implemented in-repo ([`ecs_systems_schedule_runbook_v1.md`](../../guides/ecs_systems_schedule_runbook_v1.md)): plugin inventory packs, **`SimControlSystemSet`**, transport **`TransportSchedule::Topology.after(AdvanceSimTick)`**, **`NavSets`** rename. Treat as **done** before starting **W1**; extend **S2** for navigation motion sets when needed.

---

## 1. What landed in code (this pass)

| Piece | Path | Purpose |
|:---|:---|:---|
| **`TransportSimulationPlugin`** | `src/systems/transport/mod.rs` | Registers **logical schedule** as Bevy **`SystemSet`** order (**T-SCHED-001** first mapping). |
| **Topology resource** | `TransportTopology` | Empty adjacency until **R1–R3** bake / editor feeds it. |
| **Field store** | `TransportFieldStore` + `EdgeFieldState` | Rulebook **A** minimal: decay-only integrate when sim ticks (**`SimControlState::dt_scale()`**). |
| **Cost cache** | `TransportCostCache` + `edge_traversal_cost` | Rulebook **B**: read-only weights from field; **no** pathsearch here yet. |
| **`TransportNetworkSnapshot`** | `src/systems/transport/snapshot.rs` | **R8** DTO + **`hydrate_transport_from_snapshot`** (nodes, edges, `control_points`, `profile`, `allowed_agents`). |
| **Bake** | `src/systems/transport/bake.rs` | **`bake_snapshot_from_ordered_tile_markers`**: markers in **`placement_seq`** order (see **R9** [`../../matrix/transport/runbook/r9_authoring_bake_order_steps_v1.md`](../../matrix/transport/runbook/r9_authoring_bake_order_steps_v1.md)). **No** lexicographic sort. |
| **Edge directory** | `TransportEdgeDirectory` | Stable **`profile`** + **`allowed_agents`** per edge (for **R7** export). |
| **Nav export** | `TransportNavExport` + `refresh_transport_nav_export` | **W3** / **R7**: costs, successors, masks — refreshed after cost cache each frame. |
| **Map editor** | `map_editor/mod.rs` | Road tool: **Bake roads → transport graph** (**Message** → hydrate). |

This does **not** replace the full **G4** hybrid save pipeline or spline ECS; it implements **W1–W3** code paths, editor hook, and unit tests. Full world snapshot ownership stays matrix **M5** / **R8**.

---

## 1b. Waves **W1–W3** (applied)

| Wave | In code |
|:---|:---|
| **W1** | Editor bake → `bake_snapshot_from_ordered_tile_markers` (ordered by `MapEditorRoadMarkerV1::placement_seq`) → `hydrate_transport_from_snapshot` |
| **W2** | `TransportNetworkSnapshot` serde + hydrate validation + JSON round-trip test |
| **W3** | `transport_nav_export_refresh` chained after `transport_cost_cache_refresh`; motion stage reads `TransportNavExport` |

---

## 2. Next implementation waves (ordered)

| Wave | Goal | Matrix / gates |
|:---:|:---|:---|
| **W1** | Editor bake writes into `TransportTopology` + optional `TransportFieldStore` keys | **R1–R3**, **R9** |
| **W2** | G4 snapshot round-trip for network slice → hydrate topology | **R8** |
| **W3** | `R7` nav adapter reads `TransportCostCache` or topology for coarse export | **G5** |
| **W4** | Phase II junctions: replace stub IDs with real lane connectivity | **T-LANE-001**, §8 |
| **W5** | Reservations + signals | Rulebook **C**, P2 |

---

## 3. Schedule mapping (docs ↔ Rust)

[`rulebook_drafts.md`](rulebook_drafts.md) §0.2 step → current code. **S1+S2:** [`ecs_systems_schedule_runbook_v1.md`](../../guides/ecs_systems_schedule_runbook_v1.md) · [`s2_schedule_navigation_steps_v1.md`](../../matrix/engine_bevy/runbook/s2_schedule_navigation_steps_v1.md).

| Step | Rust |
|:---:|:---|
| 1 Topology | `TransportSchedule::Topology` → `transport_topology_tick` (no-op until W1) |
| 2–3 Field | `TransportSchedule::FieldIntegrate` → `transport_field_integrate` |
| 4 Cost cache + R7 export | `TransportSchedule::CostCache` → (`transport_cost_cache_refresh`, `transport_nav_export_refresh`).chain() |
| 5–6 Nav | `NavSets::DamageSpeedAdjustment.after(CostCache)`; `NavSets::MotionCalculation.after(DamageSpeedAdjustment)` — [`NavigationSchedulePlugin`](../../src/systems/navigation/schedule_plugin.rs); **R7** reads **`TransportNavExport`**; motion body still **placeholder** |
| 7 Damage | `DamageSystem`: `apply_road_damage` in **`NavSets::DamageSpeedAdjustment`** |
| 8 Authoring | *map editor / R9 — separate plugin* |
| 9 LOD | *P3 / T-LOD-001* |

**Extend sets** when adding planning/movement (e.g. `.after(CostCache)`).

---

## 4. Verify

```bash
cargo check
cargo test  # if transport tests added later
```

---

## 5. Traceability

- Renamed designer spec: [`system_decisions_v1.md`](system_decisions_v1.md)  
- UX: [`transport_editor_ux_risk_v1.md`](transport_editor_ux_risk_v1.md) (**T-GHOST-001**)
