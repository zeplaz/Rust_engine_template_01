# PLAN-TERRITORY-MATRIX-002 — file prefix → owning program `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **PLAN-TERRITORY-MATRIX-002** |
| **Parent** | $ref:docs/archive/2026-06-src-dev/plans/planner_improvement_analysis_20260603_v1.md§D |
| **Backlog** | $ref:docs/archive/2026-06-src-dev/plans/coder_unified_backlog_v1.md§Owner-split |
| **Board** | $ref:src/dev/master_chain_board_4d_v1.md |
| **Lang** | $ref:src/dev/agent_lang_v1.md |
| **Planner** | **SIGNED** |
| **Date** | 2026-06-07 |

**Rule:** One **primary writer** per file prefix per merge window. **≤3 files per PR.** Cross-prefix edits need `⟨BP:SHARE⟩` marker with `joint:` naming the other agent.

---

## Summary

Six parallel lanes (construction, infra A/B, MCP, HUD, weather) collide on **11 hotspot files**. This matrix is the single authority for **prefix → program chain → agent** before the next multi-coder week.

---

## 1. Agent × prefix (Rust `src/`)

| Prefix / path | Program chain | Primary agent | Secondary (read-only) |
|:---|:---|:---|:---|
| `src/construction/` | H · CON-P* | **@coder A** (execute funnel) | @coder B — `site_stage_tick` only |
| `src/construction/procedural/` | E · GRAMMAR / PROC-PG | **@coder A** | @coder-mcp — Python parity witness |
| `src/infrastructure/profiles/` | H · INFRA-E0/E6 | **@coder A** | — |
| `src/infrastructure/transport/graph.rs` · `spline.rs` · `authoring/` | H · INFRA-E1/E2 | **@coder A** | — |
| `src/infrastructure/transport/junction.rs` · `snapshot_bridge.rs` | H · INFRA-E1/E3 | **@coder B** | — |
| `src/infrastructure/utility/` | H · INFRA-E4 | **@coder B** | @coder A — E4-002 flow hook only |
| `src/infrastructure/settlement/` | H · INFRA-E5 | **@coder B** | @coder A — E5-002 logistics read |
| `src/systems/transport/` | H · INFRA-E1/E3 | **@coder B** (hydrate/snapshot) | @coder A — topology consumers |
| `src/systems/navigation/` | H · INFRA-E6 | **@coder A** | — |
| `src/economy/logistics/` | H · INFRA-E5 · CON-P7 | **@coder A** (E5-002) | @coder B — historical B owner; **A leads** per $ref:docs/archive/2026-06-src-dev/plans/infra_agent_orders_v1.md |
| `src/economy/activation/` | H · IND-E02 · PROC-OG | **@coder B** | @coder A — concrete chain e2e witness only |
| `src/economy/resource_flow.rs` | H · mixed | **coordinate** — one PR owner via marker | — |
| `src/strategic/site/` · `construction_book.rs` | H · CON-P2/P5 | **@coder B** (tick/commit) | @coder A — components/progress |
| `src/strategic/spatial_network.rs` · `network_flow.rs` | H · INFRA-E4 | **@coder A** (E4-002) | @coder B — utility types |
| `src/strategic/logistics_graph.rs` | H · INFRA-E5 | **@coder A** | read-only for B |
| `src/systems/weather/` | I · WEATHER-* | **@coder C** | max **1** consumer file/PR outside weather |
| `src/engine/play_scenario.rs` | H · PLAY-TRUTH | **@coder B** (seeds) | @coder A — E5-002 routes; **never same PR** |
| `src/engine/test_harness.rs` | G · visual proof | **@coder** (harness maintainer) | A/B — seed hooks only with marker |
| `src/render/` · `infrastructure_overlay.rs` | H · INFRA-E6 | **@coder B** (E6-003) | @coder A — E6-004 debug extend |
| `src/gui/editor/` · map_editor | H · INFRA-E2 | **@coder A** | @designer — wireframe only |
| `src/gui/hud/` · `simulation_session.rs` | G · SIM-HUD | **@coder** | @designer — copy/layout |
| `src/gui/diagnostics_ui.rs` | G · infra | **@coder** | — |
| `src/terrain/` | H · INFRA-E0/E6 | **split** — registry A · deprecation B | marker required |
| `src/io/save/` · `src/io/snapshot/` | H · INFRA-E3 | **@coder B** | — |
| `src/dev/*_live_proof.rs` · `runtime_witness/` | all chains | **slice owner** writes witness | — |

---

## 2. Non-Rust territories

| Prefix | Program | Agent | Do not |
|:---|:---|:---|:---|
| `tools/mcp/python/` · `tools/mcp/art_pipeline_suite/` | B · F · D | **@coder-mcp** | @coder edits without handoff |
| `tools/mcp/schemas/` | B · C · D | **@planner-mcp** spec · **@coder-mcp** impl | planner writes Python |
| `tools/mcp/blender/` | D · B | **@coder-mcp** | chat-only bpy |
| `tools/orchestrator/queues/*.json` | L1 | **@planner** rows · **@orchestrator** drain | agents rewrite other agent's `ready` rows |
| `tools/orchestrator/queues/HANDOFF.md` | C | **@planner-mcp** overlay · **@orchestrator** paste | full prose rewrites |
| `src/dev/plan_*.md` · `*_exec_*.md` | L2 | **@planner** | @coder-mcp except MCP art exec |
| `src/dev/design_*.md` · `prompts/designer_*` | UX | **@designer** | Tk impl |
| `assets/staging/` · `assets/models/modules/` | B · D | **@coder-mcp** promote path | manual copy without witness |
| `assets/config/infrastructure/` | H | **@coder A** RON | — |
| `debug_runs/*.json` | L6 | **implementer** of slice | Read full file — `BLANG:WIT` |

---

## 3. Hotspot files — single writer per week

| File | Writers (conflict) | Primary this week | Merge rule |
|:---|:---|:---|:---|
| $sym:play_scenario@src/engine/play_scenario.rs | A E5-002 · B PLAY-TRUTH | **A** if INFRA-E5-002 open | B waits or read-only review |
| $sym:routes@src/economy/logistics/routes.rs | A E5 · B legacy | **@coder A** | B no edits until E5-002 witness green |
| $sym:ConcreteChainE2eWitness@src/economy/activation/ | A visual seed · B activation | **@coder B** | A touches witness keys only |
| `src/engine/test_harness.rs` | visual · A logistics seed · B industrial | **one PR/week** — owner via HANDOFF | others: lib test only |
| `src/strategic/construction_book.rs` | CON execute · INFRA E2 corridor | **@coder B** execute | A: read TransportEdgeRecord shape |
| `src/systems/transport/snapshot.rs` | B hydrate · A graph types | **@coder B** | A: no snapshot schema edits |
| `src/render/infrastructure_overlay.rs` | B E6-003 · A E6-004 | **@coder B** base overlay | A: extend `InfrastructureOverlayDrawRequests` only |
| `src/gui/hud/simulation_session.rs` | G HUD · harness chrome | **@coder** SIM-HUD | test harness: env flag only |
| `src/economy/resource_flow.rs` | infra utility · economy | **coordinate** | split PRs by system half |
| `Cargo.toml` / `src/lib.rs` plugin registration | all | **slice owner** adds plugin | announce in marker |
| `tools/orchestrator/queues/coder_active_queue.json` | planner sync | **@planner** | coders: `BLANG:Q✓` note only |

---

## 4. Program chain → default agent

| Chain | Domain | Default agent | Queue |
|:---|:---|:---|:---|
| **A** | DSM WRK/ATL | @coder-mcp | grammar |
| **B** | MCP productivity | @coder-mcp | grammar |
| **C** | AGENT-LANG | @planner-mcp | grammar (maintain) |
| **D** | Rowhouse prod | @coder-mcp · @designer-mcp | grammar |
| **E** | Grammar iter | maintain | — |
| **F** | APS UX | @coder-mcp | grammar |
| **G** | Bevy HUD | @coder | continuation |
| **H** | Con/Infra | **@coder A** tail · **@coder B** organic | continuation |
| **I** | Weather | @coder C | continuation |
| **J** | Defer | — | $ref:tools/orchestrator/queues/defer_registry.json |

---

## 5. Safe parallel pairs (same week)

From $ref:docs/archive/2026-06-src-dev/plans/coder_unified_backlog_v1.md§Parallel-work-safe-pairs — **still valid**:

| Lane A | Lane B | Disjoint prefix |
|:---|:---|:---|
| @coder A INFRA-E5-002 | @coder B PROC-OG-* | `economy/logistics/` vs `economy/activation/` |
| @coder A INFRA-E4/E6 | @coder C WEATHER-* | full disjoint |
| @coder-mcp MCP-SPINE | @coder A INFRA | `tools/mcp/` vs `src/infrastructure/` |
| @coder SIM-HUD | @coder A INFRA | `src/gui/hud/` vs `src/infrastructure/` |
| @designer on-call | any implementer | no `src/` edits |

**Unsafe without marker:** A + B both on `play_scenario.rs` · `test_harness.rs` · `construction_book.rs`.

---

## 6. Handoff protocol (cross-territory)

```text
⟨BP:SHARE⟩ required when:
  - touching hotspot file (§3)
  - adding plugin in lib.rs / mod.rs
  - consumer file outside primary prefix (weather rule)

Marker fields:
  mirror: what primary agent last shipped
  joint: "@other-agent — review question?"
  delta_wf: ΔWF→@owner
  territory: prefix from §1
```

**DSM boundary:** @coder-mcp → @coder handoff = **witness path + prefix**, not chat summary ($ref:docs/archive/2026-06-src-dev/plans/user_feedback_orchestration_layer_v1.md).

---

## 7. Orchestrator paste

```text
Territory authority: $ref:src/dev/plan_territory_matrix_002_v1.md

This week hotspots:
  play_scenario.rs     → @coder A if INFRA-E5-002 active
  economy/logistics/   → @coder A only
  economy/activation/  → @coder B only
  test_harness.rs      → one PR owner — check HANDOFF

Before assigning parallel coders: verify §5 pair or require ⟨BP:SHARE⟩.
```

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-07 | PLAN-TERRITORY-MATRIX-002 signed |
