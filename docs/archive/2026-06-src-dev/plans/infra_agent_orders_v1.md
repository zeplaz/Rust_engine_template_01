# INFRA tail — Agent orders (orchestrator dispatch) `v1`

| Field | Value |
|:---|:---|
| **Program** | PLAN-INFRA-WORLD-LAYERS · Chain H |
| **Lang** | $ref:src/dev/agent_lang_v1.md — `⟨ID⟩` · `$ref:` · `$sym:` · BLANG |
| **Exec** | $ref:docs/archive/2026-06-src-dev/plans/plan_infra_tail_exec_001_v1.md |
| **E5 exec** | $ref:docs/archive/2026-06-src-dev/plans/plan_construction_p7_logistics_exec_001_v1.md |
| **Parent board** | $ref:docs/archive/2026-06-src-dev/plans/plan_infrastructure_world_layers_exec_001_v1.md |
| **Queue** | $ref:tools/orchestrator/queues/continuation_queue.json |
| **Handoff** | $ref:tools/orchestrator/queues/HANDOFF.md |

---

## Assignment matrix

| Order | ⟨ID⟩ | Agent | Task | COMMIT:WIT |
|:---:|:---|:---|:---|:---|
| **0** | ⟨INFRA-E5-002⟩ | @coder A | Graph-only freight — $ref:docs/archive/2026-06-src-dev/plans/plan_construction_p7_logistics_exec_001_v1.md | $ref:debug_runs/logistics_throughput_live.json |
| **1** | ⟨INFRA-E4-002⟩ | @coder A | Utility graph + $sym:network_flow_chunk_local_solver_system@src/strategic/network_flow.rs | $ref:debug_runs/utility_network_live.json |
| **2** | ⟨INFRA-E6-001⟩ | @coder A | $sym:ProfileRegistry@src/infrastructure/profiles/mod.rs → material tags | $ref:debug_runs/transport_network_live.json |
| **3** | ⟨INFRA-E6-002⟩ | @coder A | Nav on $sym:TransportNavExport@src/systems/transport/types.rs + `allowed_agents` | $ref:debug_runs/nav_agent_routing_live.json |
| **4** | ⟨INFRA-E6-004⟩ | @coder A | Debug overlay on $sym:InfrastructureOverlayDrawRequests@src/render/infrastructure_overlay.rs | visual / lib fixture |
| **—** | ⟨INFRA-E6-003⟩ | @coder B | Base overlay render — **not** coder A | $ref:docs/archive/2026-06-src-dev/plans/design_infra_network_overlay_v1.md |

**Blocked:** none for E5 — E1-004 🟢. **Do not** wait on G-PLAY-01 operator.

---

## Paste — @coder A (primary)

```text
BLANG:PRE → BLANG:Q+("coder")
$ref:docs/archive/2026-06-src-dev/plans/infra_agent_orders_v1.md
$ref:docs/archive/2026-06-src-dev/plans/plan_infra_tail_exec_001_v1.md

ΔWF:
  ⟨INFRA-E5-002⟩ 🟡 pick first — $ref:docs/archive/2026-06-src-dev/plans/plan_construction_p7_logistics_exec_001_v1.md
    $sym:ThroughputSolverState@src/economy/logistics/mod.rs
    $sym:TransportNavExport@src/systems/transport/types.rs
    COMMIT:WIT $ref:debug_runs/logistics_throughput_live.json
    keys: graph_only_paths · routes_blocked → 0

  ⟨INFRA-E4-002⟩ ○ parallel if disjoint
    $sym:NetworkType::Power@src/strategic/spatial_network.rs
    new UtilityGraph · witness utility_network_live.json

  ⟨INFRA-E6-001⟩ ○ → ⟨E6-002⟩ → ⟨E6-004⟩
    $sym:ProfileRegistry@src/infrastructure/profiles/mod.rs
    $sym:NavigationSchedulePlugin@src/systems/navigation/schedule_plugin.rs

Territory: src/infrastructure/ · src/economy/logistics/ · src/systems/navigation/
NO: tools/mcp/ · src/construction/ execute funnel · patch_s7p_* in release

BLANG:S5 infrastructure logistics navigation
BLANG:WIT → BLANG:Q✓
```

---

## Symbol index (resolve via BLANG:DOC)

| ⟨ID⟩ | Primary $sym |
|:---|:---|
| ⟨INFRA-E4-002⟩ | $sym:UtilityNetworkSnapshot@src/infrastructure/utility/mod.rs · $sym:ChunkNetworkDigest@src/strategic/spatial_network.rs |
| ⟨INFRA-E5-002⟩ | $sym:path_open_from_nav@src/economy/logistics/routes.rs · $sym:LogisticsGraph@src/strategic/logistics_graph.rs |
| ⟨INFRA-E6-001⟩ | $sym:RoadProfile.surface_tags@src/infrastructure/profiles/mod.rs |
| ⟨INFRA-E6-002⟩ | $sym:TransportEdge.allowed_agents@src/infrastructure/transport/graph.rs |
| ⟨INFRA-E6-004⟩ | $sym:collect_transport_overlay_edges_system@src/render/infrastructure_overlay.rs |

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | ⟨AGENT-LANG-006-INFRA-REF⟩ — $ref + $sym dispatch |
