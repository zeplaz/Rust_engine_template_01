# PLAN-VEG-RUNTIME-PROOF-001 — vegetation runtime proof exec `v1`

```text
⟦SYMLANG⟧⟐v1  ◈EXEC
⟨ID⟩ PLAN-VEG-RUNTIME-PROOF-001
Date: 2026-06-14
Status: **SIGNED** (@planner 2026-06-14 · @planner-mcp 2026-06-16 · reconcile @planner 2026-06-17)
Owner: @planner → @coder (A)
Parent: POST-DRAIN-PHASE-6-001
Queue: $ref:tools/orchestrator/queues/post_drain_phase6_coder_queue.json (seq 1–2, 7–13)
Honest status: $ref:src/dev/vegetation_system_honest_status_v1.md
Charter: $ref:src/dev/guide_landscape_grammar_v1.md
Hardening: $ref:src/dev/coder_queue_hardening_rules_v1.md
```

**Goal:** Close the gap between **lib/harness witnesses** and **running sim / FULL_APP / operator-visible** vegetation — without LG-5 atlas or empty stubs.

**Rejected:** claiming veg done on eval JSON alone · single-chunk pilot · biome scalar rewrite.

---

## Problem statement

| Layer | Today | Required |
|:---|:---|:---|
| LG-1 evaluator | 🟢 lib | unchanged |
| Map rollout | 🟢 ≥16 chunks harness | unchanged |
| LG-2 disturbances | 🟢 fire+build ≥1 harness | unchanged |
| **FULL_APP ecology rows** | 🟡 harness-fed patch | **live `LandscapeProgramOnChunk` query** |
| **Preview heterogeneity** | 🟡 tint bias ≥2 chunks | **≥3 topology kinds · pixel-visible** |
| **Operator play** | 🟡 sim harness keys | **operational zoom without test env** |
| LG-5 atlas | 🔴 MCP blocked | defer — consumer path only |

---

## Proof ladder (must climb in order)

```text
L0  lib unit tests green                    (necessary · insufficient)
L1  headless sim harness witnesses green     (necessary · insufficient)
L2  FULL_APP refresh from live ECS query     (VEG-HARD-FULLAPP-001)
L3  preview raster heterogeneity            (VEG-HARD-PREVIEW-PIXEL-001)
L4  play_scenario veg keys @ operational zoom (G-PLAY-CODER-VEG)
L5  operator --test visual sign-off          (G-PLAY-OPERATOR-01 / VEG-C14)
```

**Rule:** No row above L2 marked done until witness `exit_predicate.must` passes on disk.

---

## Coder slices (machine IDs)

| Seq | ID | Deliverable | Witness | Exit |
|:---:|:---|:---|:---|:---|
| 1 | **VEG-HARD-FULLAPP-001** | `EcologyVisualSnapshot` / stage5 extract reads live program count | `stage5_full_app_live.json` | `ecology_rows_source == live_landscape_program_on_chunk` |
| 2 | **VEG-HARD-PREVIEW-PIXEL-001** | Preview tint ≥3 topology kinds visible | `landscape_grammar_lg4_preview_live.json` | `topology_kind_count_visible >= 3` · `operator_visible: true` |
| 7 | **VEG-COMPOSITE-EVAL-001** | MACRO-* registry → topology subgraph | `landscape_grammar_composite_live.json` | composite expand green |
| 9 | **VEG-λ-LIVE-001** | λ from hydrology + weather (no coord hack) | lg3/lg1 witnesses | no heuristic refs in eval path |
| 11 | **VEG-FIRE-CORRIDOR-FULLAPP-001** | Fire corridor fuel ← population field in FULL_APP | `stage5_full_app_live.json` | fire corridor linked |
| 13 | **VEG-DIAG-COMPOSITE-001** | Diagnostics: nested topology + disturbance timeline | diagnostics witness | UI fields populated |

**Parallel (B):** VEG-MINIMAP-OVERLAY-002 after seq 2.

---

## Witness contract (all rows)

```json
{
  "exit_predicate": {
    "witness": "debug_runs/<file>.json",
    "must": [{ "path": "green", "eq": true }]
  },
  "forbidden_exit": [
    "lib_test_only",
    "witness_counter_zero",
    "eval_math_without_render",
    "single_chunk_pilot"
  ],
  "live_sim_required": true,
  "operator_visible": false
}
```

Set `operator_visible: true` on rows that gate **G-PLAY-CODER-VEG**.

---

## Regression (every slice)

```powershell
cargo test -p proc_A_dine01 --lib landscape_grammar fire_ecology
python -m rust_engine_mcp.cli validate-report cargo --compress 3
```

---

## MCP / art defer (explicit)

| ID | Owner | Coder action |
|:---|:---|:---|
| VEG-F01/F02 | designer-mcp / coder-mcp | **Wait** — registry consumer stub only |
| LG-6 flowers | designer-mcp | deferred |

**Planner-mcp sign:** $ref:src/dev/plan_landscape_grammar_mcp_sign_delegate_v1.md (schema validator + preset CI).

---

## Unblocks

| Gate | When |
|:---|:---|
| **G-PLAY-CODER-VEG** | seq 1+2 green + play key |
| **G-PLAY-01 rollup** | G-PLAY-CODER-VEG + G-PLAY-OPERATOR-01 |
| **VEG-C14 operator checklist** | L3+L4 green |
| **LG-5 production atlas** | MCP sign + designer-mcp |

**Planner reconcile (2026-06-17):** Plan doc **closed**. Queue `reopened` = witness `veg_runtime_proof_live.json` WIT-HON only → **CDR-A-WIT-HON-ROLLUP-001** (@coder A). Ladder L0–L4 coder slices **done**; L5 = **G-PLAY-OPERATOR-01**.

```text
⟦/PLAN-VEG-RUNTIME-PROOF-001⟧  ΔWF→ CDR-A-WIT-HON-ROLLUP-001 · G-PLAY-OPERATOR-01
```
