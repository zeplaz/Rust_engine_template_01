# Status overview — where we are `v1` (2026-06-14 reconcile)

```text
⟦SYMLANG⟧⟐v1  ◈STATUS
Authority: witness JSON · machine queues · HANDOFF
Program: POST-DRAIN-PHASE-5-001 (drained) → POST-DRAIN-PHASE-6-001
```

---

## Executive summary

| Layer | Verdict | Plain English |
|:---|:---|:---|
| **Stage 5 / FULL_APP spine** | 🟢 | Harness witnesses green; `--test visual` proof path works |
| **Phase 4 coder drain** | 🟢 **DRAINED** | SimEffect · build UX · MAP-PICK · zoom · fire harness — seq 1–14 done |
| **Phase 5 / J_REWIRE** | 🟢 **DRAINED** | HUD/construction spine wired — [`build_read_unwired_spine_v1.md`](build_read_unwired_spine_v1.md) |
| **G-PLAY (normal play)** | 🟡 **OPEN** | Lib gates pass; operator acceptance + play scenario witness **red** |
| **Fire (product)** | 🟡 **Split** | Demo fire + ecology refresh done in drain; operator G-PLAY still open |
| **Build readability** | 🟡 **Tail** | P0 spine wired; runtime visual verify + grammar consumer + designer MCP tails |
| **MCP art P2** | 🟡 | Rowhouse pilot green; sim validators plan signed; kit002 unfreeze block coder-mcp |
| **Compile / tree** | 🟢 lib | `cargo check -p proc_A_dine01 --lib` green — spine modules in binary |

**Primary lane now:** **Phase 6 product hardening** + **G-PLAY operator closure** — not J_REWIRE.

---

## Phase 4 — closed slices (witness-backed)

| ⟨ID⟩ | Status | Witness |
|:---|:---|:---|
| TRIAGE-MAP-PICK-CLOSURE-001 | 🟢 done | `construction_stage_live.json` |
| TRIAGE-MAP-ZOOM-SMOOTH-001 | 🟢 done | `map_zoom_coherence_live.json` — witness module wired (REWIRE-003) |
| TRIAGE-BUILD-CLICK-PLACE / CURSOR-UNIFY | 🟢 done | lib + design PASS |
| SIM-EFFECT-QUEUE / TEL | 🟢 done | `sim_effect_spine_live.json` |
| FIRE-IGNITION-P0-001 | 🟢 done (lib) | producers in `src/sim/effects/` |
| SCENARIO-TRIGGER-001 | 🟢 done | `EmitSimEffect` in scenario RON |
| TRIAGE-FIRE-PRODUCT-001 | 🟢 done (harness) | `stage5_full_app_live.json` spark_rows=12 |
| BUILD-READ-SHAPE-002/003, SITE-v0-002, WORLD-002 | 🟢 done | drain queue |
| EVENT-LOG-UI-001 | 🟢 done | design PASS |
| P0-VFX-ZOOM-LOCK / TERRAIN-BLOB | 🟢 done | vfx scroll free |

**Drain authority:** [`coder_drain_queue.json`](../tools/orchestrator/queues/coder_drain_queue.json) — drained.

---

## Phase 5 J_REWIRE — closed (2026-06-11…13)

**Authority:** [`build_read_unwired_spine_v1.md`](build_read_unwired_spine_v1.md) · [`coder_master_drain_queue.json`](../tools/orchestrator/queues/coder_master_drain_queue.json) seq 1–17

| Wire ID | φ | Witness |
|:---|:---:|:---|
| BUILD-READ-REWIRE-001…004 | 🟢 | mod.rs + `cargo check --lib` |
| MINIMAP-REWIRE-001 · APS-QC-REWIRE-001 | 🟢 | minimap + APS QC witnesses |
| BUILD-READ-P0-002/003 · DEBUG-001 | 🟢 | zoom + pointer + placement debug verify |
| MINIMAP-WIDGET-IMPL-001 | 🟢 | `design_minimap_widget_live.json` |
| BUILD-READ-PILOT-001 | 🟢 | `pilot_catalog_parity_live.json` |

---

## Open — operator / product (highest priority)

| ⟨ID⟩ | Owner | Gap | Witness |
|:---|:---|:---|:---|
| **G-PLAY-01** | operator | Play scenario acceptance checklist | `play_scenario_live.json` |
| **G-PLAY-FIRE-001** | @coder | **done** in master drain — operator re-verify | `play_scenario_live.json` |
| **FIRE-ECOLOGY-REFRESH-001** | @coder | **done** in master drain | `fire_ecology_live.json` |
| **VFX-FIRE-HIGHLIGHT-001** | @coder | **done** in master drain | `vfx_fire_test_highlight_live.json` |
| **BUILD-VERIFY-*** | @coder B | Runtime product verify (phase6) | `coder_product_verify_queue.json` |

---

## Open — BUILD-READ tail (Phase 6)

| ⟨ID⟩ | Owner | Status |
|:---|:---|:---|
| BUILD-READ-CONSUMER-MCP-001 | @coder B | **ready** — APS DNA+β consumer in Rust |
| BUILD-READ-VISUAL-001 | @coder B | lib done · **runtime** operator verify open |
| BUILD-READ-VISUAL-002 | @coder-mcp | blocked — production tile bake |
| BUILD-READ-PILOT-002 | @designer-mcp | blocked — catalog row expansion |
| BUILD-READ-DESIGN-001/002 | @designer | blocked — readability brief + HUD copy |
| BUILD-READ-GRAMMAR-v0-002 | @coder-mcp | blocked — APS preset UI |

Plan: [`plan_operator_build_readability_exec_001_v1.md`](plan_operator_build_readability_exec_001_v1.md) · Wired spine: [`build_read_unwired_spine_v1.md`](build_read_unwired_spine_v1.md)

---

## Open — planner / MCP (doc gates)

| ⟨ID⟩ | Owner | Blocks |
|:---|:---|:---|
| MCP-P2-SIM-VALIDATORS-PLAN-001 | @planner-mcp | **SIGNED** — Phase 1+2 coder-mcp ready |
| MCP-P2-KIT002-PLAN | @planner-mcp | kit002+ frozen lane |
| ARCH-002 variant graph schema | @planner-mcp | variant-aware bakes |
| PLAN-AUDIT-020 | @planner | After G-PLAY-01 EXECUTED |

Backlog: [`planner_backlog_sweep_001_v1.md`](planner_backlog_sweep_001_v1.md)

---

## Fire status (detailed)

```text
Harness (--test visual/vfx)     Normal play (cargo run)
─────────────────────────     ────────────────────────
spark_rows = 12 ✅              demo fire path landed (drain)
operational_spark green ✅      G-PLAY operator checklist open
GPU compute on ✅               operator acceptance pending
f2_smoke pipeline green ✅
```

| Component | Code | Witness |
|:---|:---|:---|
| Ember + spread | `ember_spot_ignition.rs` | lib green |
| SimEffect producers | `src/sim/effects/producers.rs` | `sim_effect_spine_live.json` |
| Scenario ignite | `default_industrial_demo_fire.scenario.ron` | drain green |
| Play visibility | `play_fire_visibility.rs` | drain witnesses |
| VFX highlight box | `vfx_fire_test_highlight.rs` | `vfx_fire_test_highlight_live.json` |

---

## Witness red flags (disk truth)

| JSON | green | Note |
|:---|:---|:---|
| `play_scenario_live.json` | varies | G-PLAY operator gate |
| `minimap_compositor_live.json` | partial | M4 tails |
| `ui_shell_migration_live.json` | partial | infra not product gate |

Green anchors: `stage5_full_app_live.json`, `map_zoom_coherence_live.json`, `construction_stage_live.json`, `sim_effect_spine_live.json`.

---

## Recommended pick order (Phase 6)

### Week A — product closure

1. **G-PLAY-01** operator run + sign-off checklist  
2. **BUILD-READ-CONSUMER-MCP-001** — APS DNA+β consumer contract  
3. **BUILD-VERIFY-*** rows — runtime gates from product verify queue  

### Week B — BUILD + MCP

4. **BUILD-READ-VISUAL-001** runtime — post-commit production visual in sim  
5. **MCP-P2-QUEUE-PHASE4-001** + **MCP-P2-VALID-CONSTRUCTION-001** — @coder-mcp  
6. **VEG-HARD-*** / **INFRA-*** — dual-coder phase6 waves  

---

## Queue files (machine)

| File | Role |
|:---|:---|
| [`post_drain_phase6_coder_queue.json`](../tools/orchestrator/queues/post_drain_phase6_coder_queue.json) | **ACTIVE** — Phase 6 picks |
| [`post_drain_phase5_queue.json`](../tools/orchestrator/queues/post_drain_phase5_queue.json) | Drained — J_REWIRE historical |
| [`coder_master_drain_queue.json`](../tools/orchestrator/queues/coder_master_drain_queue.json) | Master drain seq 1–24 |
| [`HANDOFF.md`](../tools/orchestrator/queues/HANDOFF.md) | Session ritual |

```text
⟦/STATUS⟧  ΔWF→ G-PLAY-01 · BUILD-READ-CONSUMER-MCP-001 · BUILD-VERIFY-VISUAL-001
```
