# System design tension spec — transport + world sim (draft v1)

> **STATUS:** **Draft.** Tension doc only. Does **not** override [`../../matrix/transport/road_rail_migration_matrix_v1.md`](../../matrix/transport/road_rail_migration_matrix_v1.md) or [`rulebook_drafts.md`](rulebook_drafts.md) until referenced there.
>
> **Schedule:** [`rulebook_drafts.md`](rulebook_drafts.md) §0.2 · **Run plan:** [`transport_sim_runplan_v1.md`](transport_sim_runplan_v1.md)

Version: `v1.1.0` *(renamed file may follow — see README)*

---

## Core architecture identity (hypothesis)

Hybrid **transport + simulation** model:

- **Lane / network graph** — primary movement **structure** (when Phase II lane detail exists; P0 may use coarser edges).
- **Off-road / terrain field** — secondary freedom + higher cost fallback.
- **Field on edges** — continuous pressure (congestion, danger, damage, …).
- **Agents** — sparse, high-impact; they perturb field, not replace baseline flow.
- **Reservations** — hard safety where implemented (**P2+** blocks).
- **Collisions** — optional **rare** failure path; `ASK:` product (**see below**).
- **Ghost** — **authoring** vs **runtime** classes (see §5).
- **LOD** — **budget bands** for streaming + sim depth (**T-LOD-001**), not fixed global population claims.

---

## 1. Movement model (lane graph + off-road hybrid)

**Rule:** Vehicles prefer lane graph routing; they may leave it under design rules (blockage, policy, bugs **TBD**).

Illustrative structs:

```text
VehicleMovement: lane_path, optional offroad target, stuck/recovery state
```

Off-road: lower precision, higher cost, rejoin using field-aware routing when graph connectivity returns.

---

## 2. Road representation (graph + implied lanes)

- **Simulation:** edges, junction nodes, future **block** system for intersections.
- **Visual:** spline mesh, multi-lane extrusion, decals — **does not** define sim connectivity alone.
- **Lanes from profiles** — not hand-painted everywhere (**matrix R4/R9**).

---

## 3. Field system

`EdgeFieldState` (or superset) drives **soft** routing bias and designer-visible pressure.  
Routing weights come from a **read-only** cost step — see Rulebook **B** (`rulebook_drafts.md`).  
Tunable weights via data (RON / config) — **determinism** policy belongs in implementation runbook.

---

## 4. Reservation + collision (product `ASK:`)

Reservations are **hard** where enabled. **Stress → forced entry / collision** is a **strong** gameplay choice:

- Default recommendation for engine docs: **keep collisions rare and opt-in** until a designer brief locks it.
- If adopted, add explicit matrix / BQ row — do not treat as implicit in R1–R10.

Illustrative: `ReservationTable`, `ReservationSlot { start, end, owner }`, `AgentStress` — names not final.

---

## 5. Ghost system — authoring vs runtime

| Class | Purpose | Persistence |
|:---|:---|:---|
| **Authoring ghost** | Editor **R9**: spline/control preview, validation, optional “what-if” cost tint | **None** until user **bake** + **R8** snapshot includes network |
| **Runtime preview** | Debug / client visualization (path, placement helpers) | **Non-authoritative** |

**Practice:** never mix authoring ghost entities into the same “commit” path as simulation spawn without an explicit bake boundary (**T-GHOST-001**).

---

## 6. Intersection model (hybrid block + mesh)

- **Logic (future P2):** block occupancy / signals per Rulebook **C**.
- **Visual:** blended intersection mesh (`intersection_road.md` ideas) — Phase II matrix §8.

---

## 7. LOD simulation + streaming (budget framing)

**Rule:** Tiers describe **what we can afford** in a chunk or region:

- **Near / hot:** full graph + field + (when existed) reservations.
- **Mid:** field- or aggregate-forward (no per-entity detail) within a **ms/memory** envelope.
- **Far / cold:** stored or flow-summary only; streaming policy governs load.

**Do not** document immutable headcounts (e.g. “always 10k vehicles”) as product truth — tie tiers to **perf budgets** and **T-LOD-001**.

---

## 8. Design invariants

- Field does not replace graph; agents do not replace field; mesh does not drive sim connectivity.
- Reservations (when on) do not obsolete stress or field — they answer different questions unless product collapses them (`ASK:`).

---

## Traceability

- [`lane_graph_model_idea.md`](lane_graph_model_idea.md) — module boundaries + stub honesty.  
- [`rulebook_drafts.md`](rulebook_drafts.md) — orchestrator + Rulebooks A–C + P3 outline.
