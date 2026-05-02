# Lane graph & simulation modules — idea doc (draft)

> **STATUS:** Module sketch. **Stubs** (`LaneEdge`, `LaneNavGraph`, junction refresh) are **not** specification — replace under matrix **Phase II** / **T-LANE-001** in [`transport_sim_runplan_v1.md`](transport_sim_runplan_v1.md).
>
> **Authoritative schedule:** [`rulebook_drafts.md`](rulebook_drafts.md) §0.2 (same dependency order as below).

Version: `v1.1.0`

---

## 1. Lane graph module (structure)

**Role:** Pure topology — adjacency and edge records used by field attachment, pathfinding, and (later) reservations. **No** physics, no rendering.

Illustrative plugin boundary:

```rust
// Illustrative only — not API commitment
pub struct LaneGraphPlugin;

impl Plugin for LaneGraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LaneNavGraph>()
            .add_systems(Update, (rebuild_lane_adjacency, sync_junction_topology));
    }
}
```

**Data (stubs — replace in Phase II):**

- `LaneNavGraph`: adjacency map (e.g. `HashMap<Entity, Vec<Entity>>` or stable IDs TBD).
- `LaneEdge`: today **placeholder** (`length`); Phase II adds lane id, connectivity endpoints, profile ref, block membership, etc.
- `sync_junction_topology`: **must become** a real algorithm + component set — see matrix §8; until then mark **blocked** in implementation, not “magic update.”

---

## 2. Field simulation module (pressure)

**Role:** `EdgeFieldState` (congestion, damage, danger, travel_time, …) per design. Updates **after** topology for the tick is known **or** reads previous tick’s topology with explicit lag rule (pick one in implementation runbook).

---

## 3. Vehicle simulation module (actors)

**Role:** Path following, optional off-road fallback, stress hooks — **after** costs and constraints are defined for that feature tier.

---

## 4. Reservation module (safety, P2+)

**Role:** Hard temporal/spatial slots on edges or blocks. **Field = soft cost; reservation = hard gate** when both exist.

---

## 5. Signal module (intersection / rail, P2+)

**Role:** Block and chain evaluation — depends on **Rulebook C** geometry.

---

## 6. Ghost — two kinds (do not merge)

| Kind | Scope | Matrix |
|:---|:---|:---|
| **Authoring ghost** | Map editor: spline preview, validation, optional cost hint — **no** durable `NetworkEdge` until user confirms **bake** | **R9**; gated by **R8** for “Applied” |
| **Runtime preview** | Optional client-only or non-persistent debug (e.g. planned path ribbon) — **never** substituted for save/authoritative graph | `ASK:` / debug policy |

**Rule:** authoring ghost systems **do not** append to the serialized transport slice; **commit_placement** means “emit bake command,” not “silently spawn sim entities.”

---

## 7. LOD / streaming module (budget)

**Role:** Pick sim **fidelity tier** per chunk from **budget** (CPU time, memory, update rate, streaming radius) — see **T-LOD-001**.  
`SimLOD::Full | Field | Regional | Dormant` (names illustrative) describe **how much** we simulate, not “N vehicles worldwide.”

---

## 8. Logical pipeline (matches rulebook §0.2)

Within simulation (non-editor):

1. **LaneGraph** — structure refresh (`rebuild_lane_adjacency`, `sync_junction_topology` once real).  
2. **FieldSim** — integrate pressure.  
3. **Cost cache** — derive path weights (Rulebook B).  
4. **Planning** — pathfind / reservation requests.  
5. **Reservation / signals** — when P2 exists.  
6. **VehicleSim** — movement.  
7. **LODSim** — tier updates / aggregation.

**Editor authoring** runs on separate UX timing; bake output feeds step **1** on the next authoritative world commit.

---

## 9. Invariants (short)

- Field does not replace graph structure.
- Graph does not run full physics.
- Reservation does not replace field costs (and vice versa).
- **Authoring ghost** does not author sim state by itself.
- LOD does not change **saved** world truth — only runtime evaluation depth.

---

## 10. Traceability

- Matrix **R2/R3/R7** — graph shape and nav export.  
- [`sysem_desitions.md`](sysem_desitions.md) — hybrid tension spec.  
- [`rulebook_drafts.md`](rulebook_drafts.md) — orchestrator + rulebooks A–C.
