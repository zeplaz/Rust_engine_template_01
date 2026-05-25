# Construction Round 3 — plan

**Spec:** [`recovery_construction.md`](recovery_construction.md) (from § Critical Observation, line 962+)  
**Invariants:** [`construction_invariants.md`](construction_invariants.md)  
**Board:** [`construction_round3_todos.rs`](construction_round3_todos.rs)  
**Operational gate:** [`construction_operational_todos.rs`](construction_operational_todos.rs) · [`construction_operational_gate.md`](construction_operational_gate.md)

**Not Stage 5.** Construction remains a parallel lane.

---

## Prerequisite (do first)

Close **Phase 2** in order: **P6 → P7 → P8 → P9** ([`construction_phase2_todos.rs`](construction_phase2_todos.rs)).

| Block | Purpose |
|-------|---------|
| **P6** | Remove legacy contamination (shim, fake zone/demolish, tile roads, archetype map) |
| **P7** | Normalize content flow (commercial/industrial/utilities, building-only pipeline) |
| **P8** | Verify invariants (ghost policy, e2e, input conflict matrix) |
| **P9** | Proof / observability (live JSON, spline, snap, upgrade, conform — mostly done) |

Round 3 **must not** add features that bypass Phase 2 cleanup.

---

## Risk context (why Round 3 exists)

The construction core is coherent enough that **expansion can re-fragment authority**. Round 3 prioritizes:

- catalog-driven runtime (scalability)
- transport **graph** topology (intersections)
- visual authority (viewport-aligned overlays)
- performance before scale (pooling, batching)
- strict funnel enforcement (no helper spawn paths)

---

## R3-A — Catalog runtime (highest priority)

| ID | Deliverable |
|----|-------------|
| R3-A01 | `BuildingDefinition` + RON/JSON loader from `assets/configs/buildings/` |
| R3-A02 | `BuildingDefinitionRegistry` resource (id → def) |
| R3-A03 | Residential/commercial/industrial picks resolve catalog `id` |
| R3-A04 | Intent panel + ghost footprint from def (matrix / size_x/y) |
| R3-A05 | Commit uses def `SiteArchetype` + footprint — drop hardcoded duplex defaults |

Target model (from recovery):

```rust
// src/construction/building_definitions.rs (planned)
pub struct BuildingDefinition {
    pub id: String,
    pub display_name: String,
    pub footprint: UVec2,
    pub construction_cost: u32,
    pub workers: u32,
    pub power_usage: f32,
    pub water_usage: f32,
    pub archetype: SiteArchetype,
    pub category: BuildingCategory,
    // residential: unit mix via catalog extensions
}
```

---

## R3-B — Transport topology

| ID | Deliverable |
|----|-------------|
| R3-B01 | `IntersectionId` + `HashMap` registry (replace vec stub) |
| R3-B02 | On road/rail commit: register or merge node at crossing tiles |
| R3-B03 | Segment ↔ intersection linkage (entities or stable ids) |
| R3-B04 | Query API: neighbors / connected segments at tile |

Enables future traffic, signals, pathfinding anchors.

---

## R3-C — Visual authority

| ID | Deliverable |
|----|-------------|
| R3-C01 | `ConstructionVisualRequest` buffer (road/rail/zone/build ghost intents) |
| R3-C02 | Single egui/viewport draw pass consumes requests (no scattered layer ids) |
| R3-C03 | Document + wire boundary to viewport / `RepresentationResult` layer |

Connects to viewport cleanup work — construction must not own hole-latch or camera state.

---

## R3-D — Brush systems

| ID | Deliverable |
|----|-------------|
| R3-D01 | `PlacementBrushMode` (Single, Line, Rectangle, Paint) on session/tool |
| R3-D02 | Building line brush (row housing / poles) |
| R3-D03 | Zone rectangle brush |

Round 2 has alt-drag paint only.

---

## R3-E — Undo / history

| ID | Deliverable |
|----|-------------|
| R3-E01 | Demolish undo (snapshot despawned site or block with message) |
| R3-E02 | Redo stack (optional; mirror undo) |
| R3-E03 | Separate undo labels for road vs rail vs site vs zone |

Round 2: Ctrl+Z for road/site/zone only.

---

## R3-F — Rail expansion

| ID | Deliverable |
|----|-------------|
| R3-F01 | Switch node placement stub |
| R3-F02 | Junction authority resource |
| R3-F03 | Rail topology distinct in proof JSON (not only shared road tiles) |

Round 2: grade + curve + separate pipeline; not switches.

---

## R3-G — Overlay performance

| ID | Deliverable |
|----|-------------|
| R3-G01 | Pooled / reused preview state (no per-frame entity spawn) |
| R3-G02 | Incremental path preview rebuild (append-only control points) |
| R3-G03 | Batched zone paint overlay draw |

---

## R3-H — Governance

| ID | Deliverable |
|----|-------------|
| R3-H01 | [`construction_invariants.md`](construction_invariants.md) linked from AGENTS.md |
| R3-H02 | `construction_ownership.md` — frame order, who mutates what |
| R3-H03 | CI/local audit: no construction placement outside `src/construction/` |

---

## Suggested execution order

1. **Phase 2 P6–P8** green  
2. **CONSTRUCTION_OPERATIONAL_GREEN** ([`construction_operational_todos.rs`](construction_operational_todos.rs))  
3. **R3-H01** invariants + **R3-A** catalog  
4. **R3-B** intersections  
5. **R3-C** + **R3-G** in parallel  
6. **R3-D**, **R3-E**, **R3-F** as gameplay needs  

---

## Exit criteria (Round 3 lane)

- Catalog drives at least residential + one commercial/industrial def end-to-end in app  
- Intersection registry populated on crossing commits  
- Visual requests routed through one authority path  
- Operational green + proof JSON includes `round3` + `operational` boards  
- No invariant violations in audit script / witness
