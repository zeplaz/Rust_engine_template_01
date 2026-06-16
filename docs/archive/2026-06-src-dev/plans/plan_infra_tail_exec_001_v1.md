# PLAN-INFRA-TAIL-001 — E4/E6 tail slices (symbolic exec) `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **PLAN-INFRA-TAIL-001** |
| **Parent** | $ref:docs/archive/2026-06-src-dev/plans/plan_infrastructure_world_layers_exec_001_v1.md§8 · §10 |
| **E5 exec** | $ref:docs/archive/2026-06-src-dev/plans/plan_construction_p7_logistics_exec_001_v1.md |
| **Alignment** | $ref:src/dev/planner_program_alignment_v1.md§G-TOWN-ONE |
| **Overlay design** | $ref:docs/archive/2026-06-src-dev/plans/design_infra_network_overlay_v1.md |
| **Lang** | $ref:src/dev/agent_lang_v1.md |
| **Owner** | @coder A |
| **Planner** | **SIGNED** |
| **Date** | 2026-06-03 |

**Rule:** One slice per PR where possible. Resolve symbols via `BLANG:DOC` — not full module Read.

---

## Pick order (coder A — after ⟨INFRA-E5-002⟩ or parallel if disjoint)

```text
1. ⟨INFRA-E4-002⟩  utility flow — may parallel E5 if files disjoint
2. ⟨INFRA-E5-002⟩  $ref:docs/archive/2026-06-src-dev/plans/plan_construction_p7_logistics_exec_001_v1.md
3. ⟨INFRA-E6-001⟩  profile → material tags
4. ⟨INFRA-E6-002⟩  nav on TransportTopology
5. ⟨INFRA-E6-004⟩  debug overlays (E6-003 on B — do not duplicate)
```

---

## Authority map ($sym)

| Domain | Authority | Consumers |
|:---|:---|:---|
| Utility topology | $sym:UtilityNetworkSnapshot@src/infrastructure/utility/mod.rs | $sym:network_flow_chunk_local_solver_system@src/strategic/network_flow.rs |
| Network typing | $sym:NetworkType@src/strategic/spatial_network.rs | $sym:ChunkNetworkDigest@src/strategic/spatial_network.rs |
| Transport nav | $sym:TransportNavExport@src/systems/transport/types.rs | $sym:ThroughputSolverState@src/economy/logistics/mod.rs |
| Profile → surface | $sym:ProfileRegistry@src/infrastructure/profiles/mod.rs | $sym:RoadProfile.surface_tags@src/infrastructure/profiles/mod.rs |
| Overlay draw | $sym:InfrastructureOverlayDrawRequests@src/render/infrastructure_overlay.rs | $sym:collect_transport_overlay_edges_system@src/render/infrastructure_overlay.rs |
| Nav schedule | $sym:NavigationSchedulePlugin@src/systems/navigation/schedule_plugin.rs | $sym:NavSets@src/systems/navigation/schedule_plugin.rs |

**Single writer:** transport hydrate ($sym:hydrate_transport_from_snapshot@src/systems/transport/snapshot.rs — E1-004 🟢). Logistics **reads** nav — does not paint tiles.

---

## ⟨INFRA-E4-002⟩ — utility graph + solver hook

| # | Task | $sym / $ref | Exit |
|:---:|:---|:---|:---|
| 1 | **UtilityGraph resource** | new `src/infrastructure/utility/graph.rs` · wire in $sym:InfrastructureProfilesPlugin@src/infrastructure/profiles/mod.rs | Hydrate from `UtilityNetworkSnapshot` |
| 2 | **Power from graph** | $sym:NetworkType::Power@src/strategic/spatial_network.rs | No inferred power from transport profile strings |
| 3 | **Flow hook** | $sym:network_flow_chunk_local_solver_system@src/strategic/network_flow.rs | `NetworkType::Power` edges sourced from utility graph |
| 4 | **Witness** | $ref:debug_runs/utility_network_live.json | `power_edges_from_graph: true` |

**Tests:** `cargo test -p proc_A_dine01 --lib infrastructure::utility strategic::network_flow`

**Blocked:** none — E4-001 🟢 on disk.

---

## ⟨INFRA-E6-001⟩ — material tags from profiles

| # | Task | $sym / $ref | Exit |
|:---:|:---|:---|:---|
| 1 | **Tag resolver** | $sym:ProfileRegistry.resolve@src/infrastructure/profiles/mod.rs | `surface_tags` → terrain `MaterialId` |
| 2 | **No hot-path enum** | grep `RoadSurfaceType` in sim hot path | Registry lookup only (R4 matrix) |
| 3 | **Witness** | extend $ref:debug_runs/transport_network_live.json | `profile_material_histogram` non-empty |

**Territory:** `src/infrastructure/profiles/`, `src/terrain/registry_serde_path.rs` — read-only elsewhere.

**Parallel:** designer placeholder art — not blocking.

---

## ⟨INFRA-E6-002⟩ — nav agent routing

| # | Task | $sym / $ref | Exit |
|:---:|:---|:---|:---|
| 1 | **Coarse path** | $sym:TransportNavExport@src/systems/transport/types.rs | Dijkstra / A* on topology adjacency |
| 2 | **Agent filter** | $sym:TransportEdge.allowed_agents@src/infrastructure/transport/graph.rs | Train rejected on road-only edge |
| 3 | **Schedule wire** | $sym:NavigationSchedulePlugin@src/systems/navigation/schedule_plugin.rs | Replace placeholder with path follower stub |
| 4 | **Witness** | $ref:debug_runs/nav_agent_routing_live.json | `road_agent_reaches_goal: true`, `train_on_road_only: false` |

**Tests:** `cargo test -p proc_A_dine01 --lib navigation infrastructure::transport`

**Depends:** E1-001 🟢 · E5-002 recommended (nav proofs use same graph).

---

## ⟨INFRA-E6-004⟩ — debug overlays

| # | Task | $sym / $ref | Exit |
|:---:|:---|:---|:---|
| 1 | **Edge debug layer** | $sym:InfrastructureOverlayDrawRequests@src/render/infrastructure_overlay.rs | Toggle congestion / damage / utility load |
| 2 | **Debug plugin** | $sym:DebugViewportOverlayPlugin@src/render/debug_viewport_overlay.rs | Default sim **off** |
| 3 | **Witness** | optional visual capture | Overlay visible when flag set |

**Do not:** duplicate E6-003 base overlay on coder B — extend draw requests only.

---

## Acceptance (planner sign-off)

| # | ⟨ID⟩ | Criterion |
|:---:|:---|:---|
| 1 | E4-002 | `NetworkType::Power` reads utility graph; witness `power_edges_from_graph` |
| 2 | E6-001 | `surface_tags` resolve via registry; no new hardcoded road surface in hot path |
| 3 | E6-002 | Agent path respects `allowed_agents`; witness nav JSON green |
| 4 | E6-004 | Debug overlay toggle; default off in sim session |
| 5 | — | No `patch_s7p_*` in release (inherits E5-002) |

---

## Orchestrator paste

```text
BLANG:Q+("coder") · Chain H
$ref:docs/archive/2026-06-src-dev/plans/infra_agent_orders_v1.md
$ref:docs/archive/2026-06-src-dev/plans/plan_infra_tail_exec_001_v1.md

ΔWF→@coder A:
  ⟨INFRA-E4-002⟩ parallel OK if disjoint from E5
  ⟨INFRA-E5-002⟩ $ref:docs/archive/2026-06-src-dev/plans/plan_construction_p7_logistics_exec_001_v1.md
  ⟨INFRA-E6-001⟩ → ⟨E6-002⟩ → ⟨E6-004⟩

BLANG:S5 infrastructure logistics navigation
BLANG:WIT → BLANG:Q✓
```

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | PLAN-INFRA-TAIL-001 — $ref + $sym delta; unblocks coder A tail |
