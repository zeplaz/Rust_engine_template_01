# Status overview — where we are `v1` (2026-06-13)

```text
⟦SYMLANG⟧⟐v1  ◈STATUS
Authority: witness JSON · machine queues · HANDOFF
Program: POST-DRAIN-PHASE-4-001 → Phase 5 seed
```

---

## Executive summary

| Layer | Verdict | Plain English |
|:---|:---|:---|
| **Stage 5 / FULL_APP spine** | 🟢 | Harness witnesses green; `--test visual` proof path works |
| **Phase 4 coder drain** | 🟢 **DRAINED** | SimEffect · build UX · MAP-PICK · zoom · fire harness — seq 1–14 done |
| **G-PLAY (normal play)** | 🟡 **OPEN** | Lib gates pass; operator acceptance + play scenario witness **red** |
| **Fire (product)** | 🟡 **Split** | GPU sparks green in harness; normal play + ecology JSON still weak |
| **Build readability** | 🟡 **Mid** | Grammar v0 signed; **P0 HUD/construction spine unwired** — see [`build_read_unwired_spine_v1.md`](build_read_unwired_spine_v1.md) |
| **MCP art P2** | 🟡 | Rowhouse pilot green; sim validators plan + kit002 unfreeze block coder-mcp |
| **Compile / tree** | 🟢 lib | `cargo check -p proc_A_dine01 --lib` green — **unwired modules not in binary** |

**Primary lane now:** **G-PLAY closure** + **BUILD-READ tail** + **fire play loop** — not new architecture.

---

## Phase 4 — closed slices (witness-backed)

| ⟨ID⟩ | Status | Witness |
|:---|:---|:---|
| TRIAGE-MAP-PICK-CLOSURE-001 | 🟢 done | `construction_stage_live.json` |
| TRIAGE-MAP-ZOOM-SMOOTH-001 | 🟡 **unwired** | tile/zoom code landed · witness fns missing — [`build_read_unwired_spine_v1.md`](build_read_unwired_spine_v1.md) |
| TRIAGE-BUILD-CLICK-PLACE / CURSOR-UNIFY | 🟢 done | lib + design PASS |
| SIM-EFFECT-QUEUE / TEL | 🟢 done | `sim_effect_spine_live.json` |
| FIRE-IGNITION-P0-001 | 🟢 done (lib) | producers in `src/sim/effects/` |
| SCENARIO-TRIGGER-001 | 🟢 done | `EmitSimEffect` in scenario RON |
| TRIAGE-FIRE-PRODUCT-001 | 🟢 done (harness) | `stage5_full_app_live.json` spark_rows=12 |
| BUILD-READ-SHAPE-002/003, SITE-v0-002, WORLD-002 | 🟢 done | drain queue |
| EVENT-LOG-UI-001 | 🟢 done | design PASS |
| P0-VFX-ZOOM-LOCK / TERRAIN-BLOB | 🟢 done | vfx scroll free |

**Drain authority:** [`coder_drain_queue.json`](../tools/orchestrator/queues/coder_drain_queue.json) — no ready seq rows left.

---

## Open — operator / product (highest priority)

| ⟨ID⟩ | Owner | Gap | Witness |
|:---|:---|:---|:---|
| **G-PLAY-01** | operator | Play scenario acceptance after MAP-PICK + zoom | `play_scenario_live.json` **red** |
| **G-PLAY-FIRE-001** | @coder | Demo scenario ignite → sim heat → sparks at default zoom (Path A) | `play_scenario_live.json` + operator |
| **FIRE-ECOLOGY-REFRESH-001** | @coder | `fire_ecology_live.json` stale (0 heat on disk vs lib green) | `fire_ecology_live.json` |
| **VFX-FIRE-HIGHLIGHT-001** | @coder | Red box marker for `--test vfx` fire region (landed, needs witness) | new — `vfx_fire_test_highlight_live.json` |
| **MINIMAP-WIDGET-IMPL-001** | @coder | **Blocked** — `minimap_bevy_interaction.rs` on disk, **not wired** · MINIMAP-REWIRE-001 | `design_minimap_widget_live.json` (design green) |

---

## Unwired spine — BUILD-READ / HUD (must re-wire before verify)

**Authority:** [`build_read_unwired_spine_v1.md`](build_read_unwired_spine_v1.md) · **Queue lane:** `J_REWIRE` in [`post_drain_phase5_queue.json`](../tools/orchestrator/queues/post_drain_phase5_queue.json)

| Wire ID | Owner | Plain English |
|:---|:---|:---|
| BUILD-READ-REWIRE-001 | @coder | `placement_debug.rs` back into build graph + projection helpers |
| BUILD-READ-REWIRE-002 | @coder | `simulation_pointer_gate.rs` + rail submenu rect in ops shell |
| MINIMAP-REWIRE-001 | @coder | Minimap shell API (`MinimapEdge`, title bar) + wire Bevy interaction |
| BUILD-READ-REWIRE-003 | @coder | `map_zoom_coherence_001` witness module for live proof |
| BUILD-READ-REWIRE-004 | @coder | Pilot catalog authority in commit path (not just RON on disk) |
| APS-QC-REWIRE-001 | @coder | `assembly_snapshot_qc_ui.rs` into `gui/mod.rs` |

---

## Open — BUILD-READ tail (Phase 5)

| ⟨ID⟩ | Owner | Status |
|:---|:---|:---|
| BUILD-READ-P0-002 | @coder | **Blocked** on REWIRE-003 — then refresh `map_zoom_coherence_live.json` |
| BUILD-READ-P0-003 | @coder | **Blocked** on REWIRE-001/002 — pointer gate + placement debug not compiled |
| BUILD-READ-DEBUG-001 | @coder | **Blocked** on REWIRE-001 |
| BUILD-READ-DESIGN-001/002 | @designer | Readability brief + HUD copy — **sign-off pending** |
| BUILD-READ-GRAMMAR-v0-002 | @coder-mcp | APS DNA preset + β sliders |
| BUILD-READ-GRAMMAR-v0-003 | @coder | Evaluator DNA+β → massing pick |
| BUILD-READ-VISUAL-001 | @coder | Post-commit lod0/production mesh visible in sim |
| BUILD-READ-VISUAL-002 | @coder-mcp | Production tile bake for pilot warehouse |
| BUILD-READ-PILOT-001/002 | @coder / @designer-mcp | Pilot catalog — **partial** (RON + `pilot_catalog.rs` wired; commit path still hardcoded) |

Plan: [`plan_operator_build_readability_exec_001_v1.md`](plan_operator_build_readability_exec_001_v1.md) · Unwired: [`build_read_unwired_spine_v1.md`](build_read_unwired_spine_v1.md)

---

## Open — planner / MCP (doc gates)

| ⟨ID⟩ | Owner | Blocks |
|:---|:---|:---|
| MCP-P2-SIM-VALIDATORS-PLAN-001 | @planner-mcp | **SIGNED** — Phase 1+2 coder-mcp ready |
| MCP-P2-KIT002-PLAN | @planner-mcp | kit002+ frozen lane |
| ARCH-002 variant graph schema | @planner-mcp | variant-aware bakes |
| PLAN-AUDIT-020 | @planner | After G-PLAY-01 EXECUTED |
| PLAN-MAP-ZOOM-SMOOTH-001 | @planner | **done in phase4** — sync planner_active_queue |

Backlog: [`planner_backlog_sweep_001_v1.md`](planner_backlog_sweep_001_v1.md)

---

## Fire status (detailed)

```text
Harness (--test visual/vfx)     Normal play (cargo run)
─────────────────────────     ────────────────────────
spark_rows = 12 ✅              overlay OFF by default
operational_spark green ✅      need zoom α≥0.42 for sparks
GPU compute on ✅               G-PLAY scenario witness red
f2_smoke pipeline green ✅      operator: "no fire/sparks"
```

| Component | Code | Witness |
|:---|:---|:---|
| Ember + spread | `ember_spot_ignition.rs` | lib green |
| SimEffect producers | `src/sim/effects/producers.rs` | `sim_effect_spine_live.json` |
| Scenario ignite | `default_industrial_demo_fire.scenario.ron` | not proven in play |
| Play visibility | `play_fire_visibility.rs` | lib green; play red |
| VFX highlight box | `vfx_fire_test_highlight.rs` | **new — unwitnessed** |

---

## Witness red flags (disk truth)

| JSON | green | Note |
|:---|:---|:---|
| `play_scenario_live.json` | false | G-PLAY blocker |
| `fire_ecology_live.json` | false | Stale minimal run |
| `minimap_compositor_live.json` | partial | M4 tails |
| `ui_shell_migration_live.json` | partial | infra not product gate |

Green anchors: `stage5_full_app_live.json`, `sim_effect_spine_live.json`, `construction_stage_live.json`, `sim_effect_spine_live.json`.

---

## Recommended pick order (next 2 weeks)

### Week A — product closure

1. **G-PLAY-01** operator run + sign-off checklist  
2. **G-PLAY-FIRE-001** — scenario → heat → sparks at play zoom  
3. **VFX-FIRE-HIGHLIGHT-001** — witness + `--test vfx` operator verify  
4. **FIRE-ECOLOGY-REFRESH-001** — refresh ecology JSON from F2 proof train  

### Week B — BUILD-READ + MCP

5. **BUILD-READ-VISUAL-001** — post-commit production visual  
6. **BUILD-READ-PILOT-001** — catalog authority  
7. **MCP-P2-QUEUE-PHASE4-001** + **MCP-P2-VALID-CONSTRUCTION-001** — @coder-mcp (parallel, plan signed)
8. **BUILD-READ-GRAMMAR-v0-002** — APS preset UI  

---

## Queue files (machine)

| File | Role |
|:---|:---|
| [`post_drain_phase5_queue.json`](../tools/orchestrator/queues/post_drain_phase5_queue.json) | **NEW** — next picks |
| [`coder_drain_queue.json`](../tools/orchestrator/queues/coder_drain_queue.json) | Phase 4 drained |
| [`post_drain_phase4_queue.json`](../tools/orchestrator/queues/post_drain_phase4_queue.json) | Historical + G-PLAY open |
| [`mcp_active_queue.json`](../tools/orchestrator/queues/mcp_active_queue.json) | MCP P2 |
| [`HANDOFF.md`](../tools/orchestrator/queues/HANDOFF.md) | Session ritual |

```text
⟦/STATUS⟧  ΔWF→ G-PLAY-01 · G-PLAY-FIRE-001 · BUILD-READ-REWIRE-003 · BUILD-READ-REWIRE-001
```
