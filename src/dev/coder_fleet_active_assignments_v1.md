# Coder fleet — active assignments `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Machine queue** | [`tools/orchestrator/queues/coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) |
| **Rule** | One **primary** per coder per session; parallel only when **file domains disjoint** |

**Gates closed:** WSS-DESIGN-GATE-001 **PASS (qualified)** · CONSTRUCTION-PARAM-DESIGN-001 **PASS (qualified)** · PLAN-CONSTRUCTION-PARAM **SIGNED**

---

## Parallel fleet (start today)

| Coder | P1 (now) | P2 (next) | Do not touch |
|:---|:---|:---|:---|
| **A** | **WSS-CHUNK-SLAB-001** | **FIRE-F2-EXTRACT-001** | `src/construction/*` |
| **B** | **CONSTRUCTION-PARAM-CODER-002** (P2-A input) | **CONSTRUCTION-PARAM-CODER-003** (P1-B `TileOccupationBook`) | `src/substrate/*` |
| **B** (alt) | **M3-UNITS-DEPTH-001** | **REPLAY-RING-LIVE-001** | if construction saturated |

**Defer:** **R4-MV-GHOST-001** (fights parametric `visual_authority.rs` Phase 2-B).

---

## @coder A — WSS-CHUNK-SLAB-001

| Field | Value |
|:---|:---|
| **Plan** | [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) |
| **Design** | [`wssr_design_signoff_v1.md`](wssr_design_signoff_v1.md) |
| **Orders** | [`wssr_coder_hybrid_orders_v1.md`](wssr_coder_hybrid_orders_v1.md) |
| **Witness** | `debug_runs/wss_substrate_live.json` |
| **Module** | `src/substrate/` (new) |

**Exit this session:** CS-001..CS-004 — types, registry, hydrate stub, witness writer (all flags false, `green: false`).

**Hybrid default:** `ChunkWeather` / `ChunkSurfaceFire` **unchanged** — no sim tick writes to slab.

```powershell
cargo test -p proc_A_dine01 --lib wss_substrate
cargo test -p proc_A_dine01 --lib stage5 fire_streaming
```

---

## @coder A — FIRE-F2-EXTRACT-001 (after or second session)

| Field | Value |
|:---|:---|
| **Plan** | [`fire_ecology_f1_todos.md`](fire_ecology_f1_todos.md) F2-01..04 · charter [`planner_elemental_vfx_domain_charter_v1.md`](planner_elemental_vfx_domain_charter_v1.md) |
| **Witness** | `fire_ecology_live.json`, `stage5_full_app_live.json` (`fire_instance_buffer_rows`) |
| **Goal** | Projection-graph fire instances — close **VX-P2-01** |

**Files:** `fire_view_extract.rs`, `render_projection_graph`, extraction — **no substrate**.

---

## @coder B — CONSTRUCTION-PARAM-CODER-002 (Phase 2)

| Field | Value |
|:---|:---|
| **Plan** | [`plan_construction_param_exec_phases_v1.md`](plan_construction_param_exec_phases_v1.md) **P2-A** |
| **Design** | [`construction_parametric_design_signoff_v1.md`](construction_parametric_design_signoff_v1.md) |
| **Prereq** | **CODER-001** ☑ `weighted_footprint.rs` |
| **Witness flags** | `shift_queue_building_removed`, `enter_commits_single_ghost` |

**Files (≤3):** `build_state.rs`, `build_interaction.rs`, `build_tool_authority.rs`

**Rules:** Buildings only — roads/rail/zone unchanged. Enter commits ghost; remove Shift+LMB building queue.

```powershell
cargo test -p proc_A_dine01 --lib construction
```

---

## @coder B — CONSTRUCTION-PARAM-CODER-003 (Phase 1 tail)

| Field | Value |
|:---|:---|
| **Plan** | **P1-B** in [`plan_construction_param_exec_phases_v1.md`](plan_construction_param_exec_phases_v1.md) |
| **Files (≤3):** `src/strategic/site/tile_occupation.rs`, `components.rs`, `mod.rs` |
| **Witness** | `overlap_blocks_commit`, `commit_carries_scale_and_weights` |

**Can run after P2-A** or **parallel** if different coder — same owner B sequential recommended.

---

## @coder B — M3-UNITS-DEPTH-001 (disjoint lane)

| Field | Value |
|:---|:---|
| **Plan** | [`plan_m3_depth_exec_001_v1.md`](plan_m3_depth_exec_001_v1.md) |
| **Design** | [`minimap_m3_unit_aggregation_visual_v1.md`](minimap_m3_unit_aggregation_visual_v1.md) |
| **Witness** | `minimap_compositor_live.json` |

**No** `src/construction/` except if minimap reader only touches gui/compositor.

---

## Done — do not re-open

F7-STREAM-DEEP · R4-CORRIDOR · CONSTRUCTION-PARAM-CODER-000/001 · wave 3 closure bundles

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Fleet reopened after planner + design gates |
