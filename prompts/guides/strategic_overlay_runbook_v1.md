# Strategic overlay runbook `v1`

**Purpose:** Define the **dynamic operational overlay** system — soft regions, influence fields, pressure gradients, network stress, and operational geography — as **fields** over the map, not static polygons.

**Parent:** [`strategic_fields_and_ai_orchestrator_v1.md`](strategic_fields_and_ai_orchestrator_v1.md) → [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

**Related:** [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md), [`weather_simulation_runbook_v1.md`](weather_simulation_runbook_v1.md), [`fire_ecology_simulation_runbook_v1.md`](fire_ecology_simulation_runbook_v1.md), [`chunk_scheduler_runbook_v1.md`](chunk_scheduler_runbook_v1.md), [`ui_operational_direction_runbook_v1.md`](ui_operational_direction_runbook_v1.md)

**Source draft:** [`base_ai_runbook_draft.md`](base_ai_runbook_draft.md) (archived bundle)

Version: `v1.0.0`

---

## Execution rounds (run tests after each round)

Sequential delivery: finish round *N* before starting *N+1* for this runbook.

| Round | Goal | Verify |
|:---:|:---|:---|
| **R1** | Chunk-aligned overlay buffers spawn with [`ChunkCellMatrix`](../../src/terrain/generation/cell_matrix.rs) | `cargo test -p proc_A_dine01 strategic::plugin::tests::strategic_overlay_spawns_with_chunk_matrix` |
| **R2** | `LogisticsGraph` → per-cell `logistics_throughput` / `logistics_strength` inject | `cargo test -p proc_A_dine01 strategic::logistics_net::tests::logistics_edge_injects_throughput_at_anchors` |
| **R3** | Disruption drives effective flow to zero (coupling contract) | `cargo test -p proc_A_dine01 strategic::logistics_net::tests::logistics_full_disruption_zeroes_injection` |
| **R4+** | Faction slots, recon/EW/artillery writers, overlay UX toggles | Add tests per field owner; filter `cargo test -p proc_A_dine01 strategic::` |

**Code:** [`ChunkStrategicOverlay`](../../src/strategic/mod.rs), [`StrategicFieldsPlugin`](../../src/strategic/plugin.rs), [`logistics_net_inject_into_overlays`](../../src/strategic/logistics_net.rs).

---

## 1. Purpose

Overlays represent:

- Soft regions and influence fields  
- Pressure gradients and network stress  
- Operational geography (threat, recon confidence, congestion, etc.)

**Philosophy:** Strategic regions are **dynamic probabilistic fields**, not static polygons.

---

## 2. Overlay categories

| Overlay | Purpose |
|:---|:---|
| Logistics | Throughput, bottlenecks |
| Recon | Visibility / confidence |
| Artillery | Danger / fire effects |
| EW | Signal denial, jamming intensity |
| Fire | Hazard / spread coupling |
| Weather | Operational penalty (mobility, sensors) |
| Morale / instability | Coarse social pressure *(optional domain)* |
| Congestion | Routing stress |

---

## 3. Representation

Overlays may use:

- Chunk grids  
- GPU textures  
- Sparse fields  
- Graph weights (line haul stress alongside fields)

---

## 4. Multi-faction support

Each faction may maintain separate:

- Visibility  
- Threat assessment  
- Confidence  
- Logistics valuation (cost / risk weighting)

---

## 5. Blob / region logic

Operational regions emerge from:

- Diffusion and connectivity  
- Pressure accumulation  
- Path costs and corridor usage

---

## 6. Composition rule

Conceptual composition:

`terrain + weather + recon + logistics + fire + EW → operational viability`

Implementations must name a **single owning system** per composed metric (see expansion orchestrator invariants).

---

## 7. GPU suitability (candidates)

| Overlay | GPU suitability |
|:---|:---|
| Artillery danger | High |
| Recon visibility | High |
| Fire spread | High |
| Weather | High |
| Congestion | High |

Authoritative **ownership, routing decisions, and saves** stay on CPU/ECS where the meta-runbook requires it.

---

## 8. Overlay UX

Visualization:

- Contours, heatmaps  
- Animated flows  
- Pressure gradients  
- Vector arrows  

Prefer **map-attached** presentation (see [`ui_boundary_guide_v1.md`](ui_boundary_guide_v1.md)).

---

## 9. Acceptance (v1 drafting)

- [ ] Each overlay category has a **named owner system** and update cadence (per chunk/LOD).  
- [ ] Faction-specific views do not duplicate unrelated GPU paths without a documented reason.  
- [ ] At least one **debug toggle** route exists for overlay visibility (dev or gameplay inspector).  
- [ ] Composition rules avoid **weather → terrain ontology** mutation (expansion orchestrator path).

---

## 10. Long-term goal

The world map becomes a **living operational analysis surface**, not merely a terrain renderer.
