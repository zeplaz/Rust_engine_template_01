# Transport + simulation — orchestrator & rulebooks (draft)

> **STATUS:** Draft contracts for **later** implementation. **Does not** replace [`../../matrix/transport/road_rail_migration_matrix_v1.md`](../../matrix/transport/road_rail_migration_matrix_v1.md) **R1–R10** or Phase II until those rows say so.
>
> **Run plan & todos:** [`transport_sim_runplan_v1.md`](transport_sim_runplan_v1.md) (**T-SCHED-001**, **T-LANE-001**, **T-GHOST-001**, **T-LOD-001**).
>
> **Code:** [`transport_code_implementation_plan_v1.md`](transport_code_implementation_plan_v1.md) · `crate::systems::transport`.

Version: `v1.1.0` (condensed — duplicate rulebook numbering removed; scope split by phase)

---

## 0. Orchestrator

### 0.1 Phase map

| Phase | What it is | Where it lives |
|:---:|:---|:---|
| **P0** | Matrix **R1–R10**: network graph, profiles, **R8** save, **R9** spline authoring + bake, **R10** visuals, **R7** coarse nav export | Matrix + map editor + G4/G5 |
| **P1** | Rulebooks **A–C** — field lifecycle, read-only pathfinding weights, junction-block *rules* (implementation follows P0 graph) | This doc §A–§C |
| **P2** | Matrix **§8 Phase II** — junction / lane topology, replace lane-graph **stubs** — **no hand-waving** (**T-LANE-001**) | Matrix §8 + designer specs |
| **P3** | Trains / logistics, chunk streaming, sim LOD as **budget tiers** — separate orchestrator when P0 stable | §D outline only |

### 0.2 Logical simulation schedule (dependency order)

Define **this** order in design docs and map it to the engine scheduler when code exists (**T-SCHED-001**). This is **not** a Bevy tutorial; framework-specific labels are out of scope here.

| Step | Layer | Responsibility |
|:---:|:---|:---|
| **1** | **Topology** | Build or refresh **structure**: nodes, edges, lane adjacency. Downstream systems must not read stale graph pointers. *Includes lane-graph plugin work once it exists.* |
| **2** | **Field — input** | Apply sparse deltas (agents, events, environment) into field scratch buffers. |
| **3** | **Field — integrate** | Propagate, decay, commit **EdgeFieldState** / optional region aggregates. |
| **4** | **Cost cache** | Compute **read-only** edge weights for pathfinding from field + terrain + static geometry. **Path search reads cache only** — it does not write field or move agents (see rulebook **B** wording). |
| **5** | **Planning** | Path requests, reservation *requests* (when implemented). |
| **6** | **Constraints** | Reservations, signals, block gates (**P2+** when junction blocks exist). |
| **7** | **Movement** | Commit motion along lane graph / valid off-graph fallback per design. |
| **8** | **Authoring (editor)** | **Authoring ghost** — preview and validate only; **no** durable transport graph mutation until user **bake** and **R8** can persist (**matrix R9**). Runs on editor clock, not sim ordering authority. |
| **9** | **LOD / streaming** | Choose **budget tier** per chunk (CPU/ms, memory, active radius). Tiers describe **what we can afford**, not marketing headcounts (**T-LOD-001**). |

**Data flow (summary):** topology → field → weights → search → (reservations) → movement → field feedback.

---

## Rulebook A — Field system (lifecycle)

### A.1 Purpose

Continuous **pressure** on edges (and optional regions): congestion, damage, danger, usage heat, recovery. **Not** navigation, not full physics, not per-vehicle trajectory storage.

### A.2 Core data (illustrative)

```text
EdgeFieldState: congestion, damage, danger, heat, decay_rate, ...
RegionFieldState (optional): aggregated values per chunk/district
```

Layers may stack: **edge** (fine) → **region** (coarse) → **global** macro signals — each phase adds fidelity; P0 may ship edge-only.

### A.3 Update sequence (within field step)

1. Reset or accumulate transient deltas.  
2. Apply agent / environment contributions.  
3. Propagate along adjacency (optional diffusion — deterministic rules TBD in implementation).  
4. Decay toward baseline.  
5. Commit values visible to **Rulebook B**.

### A.4 Rules

- Agents (or economy hooks) **write** field through defined APIs; they **do not** edit nav topology directly.
- **Navigation does not write** `EdgeFieldState`.
- Field updates **before** pathfinding reads the cost cache in the same logical tick (schedule §0.2).

---

## Rulebook B — Pathfinding cost (read-only integration)

### B.1 Purpose

Bridge **field + terrain + static geometry** → **scalar edge weight** consumed by A* / Dijkstra / hierarchical variants.

### B.2 Cost payload (illustrative)

```text
EdgeCostSnapshot: base, congestion_sample, damage_sample, danger_sample, slope, ...
effective_weight = f(base, samples, CostWeights profile)
```

### B.3 Rules

- **Weights** (`CostWeights` or RON) are designer-tunable.
- **Search algorithm** combines stored `g`-costs / heuristic using these weights — it does **not** run simulation (no field writes, no agent spawn) inside the expand step. **Prepare weights first** (step 4 in §0.2).
- **Determinism:** same field snapshot + same graph + same seed policy → same tie-breaking; document fixed timestep / ordering in implementation runbook.

### B.4 Cache invalidation

When field or topology changes materially, invalidate affected edges’ cached weights before planning waves that depend on them.

---

## Rulebook C — Junction block partitioning (P2+)

### C.1 Purpose

Discretize intersections into **conflict-free blocks** for reservations and signals (road + rail).

### C.2 Model (illustrative)

```text
JunctionBlock: id, member edges, attached signals
```

### C.3 Partition rules

- No two **conflicting** movements occupy the same block simultaneously unless a designed exception says so (`ASK:` in product).
- Blocks are **independently reservable** where the model applies.
- Signals (or chain rules) **gate** entry into downstream blocks.
- **Deadlock:** forward evaluation of block chains; detect cycles; fallback yield — detail in Phase II spec.

### C.4 Scope

Logical spec only until **P2** topology exists. Do not block **P0** spline bake on full block solver.

---

## Phase P3 — Extended scope (outline only)

**Orchestrator:** TBD — own runbook when P0+P2 mature. Content below is **not** P0 deliverable.

### D.1 Train scheduling & logistics

- Entities such as `Train`, `ScheduleStop`, `Station`, demand/supply maps.
- Assignment uses **field-weighted** costs + reservation feasibility when rail blocks exist.
- **Product:** priority formulas and economy coupling are backlog / BQ, not matrix R1–R10.

### D.2 Chunk streaming & sim LOD (budget framing)

- **Chunks** load/unload by spatial policy + streaming cost.
- **Sim LOD tiers** are **budget bands**: e.g. “full fidelity within player budget,” “field-only mid ring,” “aggregated far ring,” “cold storage” — each tied to **frame-time / memory / update frequency** targets, **not** fixed global vehicle counts.
- **Streaming** preserves stable identities and continuity of reservations **or** defines explicit handoff when tiers change (**T-LOD-001**).
- **Rule:** changing LOD **changes resolution of evaluation**, not immutable world truth.

---

## Global invariants (all phases)

- **Field** informs cost; it does **not** replace the **lane/network graph** as structural truth.
- **Blocks** (when present) constrain **what is allowed**, not replace movement policies by themselves.
- **Mesh** is visual unless a future row explicitly couples it to sim (**matrix R10 / Phase II**).
- **Authoring ghost** ≠ runtime entities; **bake** + **R8** gate durable graph (**T-GHOST-001**).
- **Chunks / LOD** affect **where** systems run and **how coarsely**, not arbitrary rewriting of saved world state.

---

## Traceability

| This doc | Matrix |
|:---|:---|
| P0 | **R1–R10** |
| P2 / Rulebook C | **§8 Phase II** |
| P3 | Outside R1–R10 until new rows |
| Schedule §0.2 | Informs **R7**, post-foundation **A/B/C** — see [`../../matrix/transport/reference_post_foundation_track_v1.md`](../../matrix/transport/reference_post_foundation_track_v1.md) |
