# PLAN-G-PLAY-SPLIT-001 — G-PLAY sub-gate model `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-G-PLAY-SPLIT-001
Date: 2026-06-14
Status: **SIGNED** (@planner 2026-06-14 · @planner-mcp 2026-06-16 · reconcile @planner 2026-06-17)
Parent: PLAN-G-PLAY-CLOSE-001 · G-PLAY-01
Runbook: $ref:src/dev/play_scenario_acceptance_runbook_v1.md
Checklist: $ref:src/dev/plan_g_play_close_001_checklist_v1.md
```

**Problem:** `G-PLAY-01` mixed **operator human acceptance** with **coder product witnesses**. Coder rows closed while rollup stayed red — queue confusion and false "blocked on G-PLAY" for coders.

**Fix:** Split into **coder sub-gates** (machine-verifiable) + **operator sub-gate** (human). Rollup closes only when all green.

---

## Gate tree

```text
G-PLAY-01 (rollup)                    💬 OPEN
├── G-PLAY-CODER-FIRE                 🟢 CLOSED
├── G-PLAY-CODER-BUILD                🟢 CLOSED (runtime verify done)
├── G-PLAY-CODER-VEG                  🟢 CLOSED (witnesses green 2026-06-14)
└── G-PLAY-OPERATOR-01                🔴 OPEN (human 10 min checklist)
```

---

## Sub-gate definitions

### G-PLAY-CODER-FIRE 🟢

| Field | Value |
|:---|:---|
| **Owner** | @coder A |
| **Witness** | `debug_runs/play_scenario_live.json` |
| **Key** | `demo_fire_sparks_visible_at_operational_zoom == true` |
| **Queue** | G-PLAY-FIRE-001 · FIRE-VERIFY-PLAY-001 (done) |
| **Blocks rollup?** | No — green |

### G-PLAY-CODER-BUILD 🟢

| Field | Value |
|:---|:---|
| **Owner** | @coder B |
| **Witnesses** | `map_zoom_coherence_live.json` · `build_verify_pointer_live.json` · `pilot_catalog_parity_live.json` · `build_read_visual_001_live.json` |
| **Keys** | `green: true` · `runtime_sim_verified: true` where applicable |
| **Queue** | BUILD-VERIFY-* · REWIRE (done) |
| **Blocks rollup?** | No — green |

### G-PLAY-CODER-VEG 🟢

| Field | Value |
|:---|:---|
| **Owner** | @coder A |
| **Plan** | $ref:src/dev/plan_veg_runtime_proof_001_v1.md |
| **Witnesses** | `stage5_full_app_live.json` · `landscape_grammar_lg4_preview_live.json` · `play_scenario_live.json` · `g_play_product_close_live.json` |
| **Keys** | `ecology_rows_source: live_landscape_program_on_chunk` · `topology_kind_count_visible >= 3` · `veg_topology_visible_at_operational_zoom` |
| **Queue** | VEG-HARD-FULLAPP-001 · VEG-HARD-PREVIEW-PIXEL-001 (done) |
| **Blocks rollup?** | No — green |

### G-PLAY-OPERATOR-01 🔴

| Field | Value |
|:---|:---|
| **Owner** | **Operator** (not @coder) |
| **Doc** | `plan_g_play_close_001_checklist_v1.md` § Operator session |
| **Rule** | No harness env · release build · ≥10 min sim play |
| **Blocks rollup?** | **Yes** — sole human gate |
| **Unblocks** | PLAN-AUDIT-020 |

---

## Rollup close predicate

```text
G-PLAY-01 CLOSED ⇔
  G-PLAY-CODER-FIRE 🟢
  ∧ G-PLAY-CODER-BUILD 🟢
  ∧ G-PLAY-CODER-VEG 🟢
  ∧ G-PLAY-OPERATOR-01 EXECUTED
```

**Witness rollup:** `debug_runs/g_play_product_close_live.json` — refresh when all four green.

---

## Queue routing (after split)

| Agent | Pick | Do NOT pick |
|:---|:---|:---|
| **@coder A** | VEG-HARD-* · Phase 6 veg | G-PLAY-OPERATOR-01 |
| **@coder B** | BUILD-GRAMMAR-* · INFRA | G-PLAY-OPERATOR-01 |
| **Operator** | G-PLAY-OPERATOR-01 checklist | — |
| **@planner** | PLAN-AUDIT-020 after operator | — |

**OPS registry:** map `G-PLAY-01` → children in `OPS_LANE_REGISTRY.json` (planner sync).

---

## Orchestrator paste

```text
G-PLAY-01 is a ROLLUP — not a coder pick.
Coder lanes: G-PLAY-CODER-FIRE 🟢 · G-PLAY-CODER-BUILD 🟢 · G-PLAY-CODER-VEG 🟢 (2026-06-14).
Operator lane: G-PLAY-OPERATOR-01 🔴 — sole rollup blocker.
Witness honesty: rollup refresh via CDR-A-WIT-HON-ROLLUP-001 if WIT-HON FAIL.
```

```text
⟦/PLAN-G-PLAY-SPLIT-001⟧  ΔWF→ PLAN-VEG-RUNTIME-PROOF-001 · operator checklist
```
