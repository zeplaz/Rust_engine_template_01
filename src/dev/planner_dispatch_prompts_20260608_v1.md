# Planner dispatch prompts — Phase 2 `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **PLANNER-DISPATCH-PROMPTS-20260608** |
| **Date** | 2026-06-08 |
| **Audience** | @planner → paste blocks to @coder / @designer / @sim-steward / Operator |
| **Queue** | $ref:tools/orchestrator/queues/post_drain_phase2_queue.json |
| **Prior program** | $ref:src/dev/post_drain_dispatch_program_v1.md — **Phase 1 drained** |

**Use:** Copy **§@planner** into planner chat first, then issue **one paste block per agent** from §Dispatch blocks.

---

## Crosswalk — status across all places

**Rule:** When columns disagree, **witness JSON** wins. Planner job #0 is sync before new code.

### DSM AUTH spine

| Node | Witness | post_drain | HANDOFF | tensor | Truth |
|:---|:---|:---:|:---:|:---:|:---|
| MAT | profiles on disk | ★ | ★ | ★ | 🟢 |
| APS | `aps_ux_async_001` green | ★ | ○ stale | ★ | 🟢 |
| SNAP | `grammar_iter_001_e2e` | ★ | ★ | ★ | 🟢 |
| WRK | `build_worker_001` | ★ | ★ | ★ | 🟢 |
| ATL | `atl_sign_001` + production brief | done | ◐ | ○ | 🟡 qualified |
| RT | `rt_registry_001` + `rt_eng_001` | done | ○ | 🧊 | 🟡 needs signoff |

### POST-DRAIN Phase 1 (post_drain_active_queue.json)

| Lane | done | ready | deferred | Notes |
|:---|:---:|:---:|:---:|:---|
| K RT | 4/4 | 0 | 0 | Registry + brief + eng on disk |
| L Fire | 3/4 | 1 | 0 | STREAM witness **green** but row still **ready** |
| M Wx | 2/2 | 0 | 0 | `weather_sim_live.json` **green: true** |
| N VM | 2/2 | 0 | 0 | `vm_10_lockstep_001.green: true` |
| O Prod | 3/3 | 0 | 0 | IND-E02, BQ-128 done |
| P MCP | 1/2 | 0 | 1 | promote test done |
| Q Plan | 2/3 | 0 | 1 | audit 020 waits G-PLAY |

### Continuation queue drift

| ⟨ID⟩ | continuation | post_drain | witness | Fix |
|:---|:---:|:---:|:---|:---|
| SLICE-TRIAGE-FIRE-STREAM | **ready** | ready | `fire_streaming` **green** | **CLOSE** — sync only |
| SLICE-MD-F2-03 fuel spread | **ready** | done | `fuel_spread_counters_wired: false` | **REOPEN tail** FIRE-FUEL-COUNTERS |
| SLICE-MD-F2-01/02 | done | done | green | 🟢 |

### Agent idle truth (2026-06-08)

| Agent | HANDOFF says | Actually | Phase 2 pick |
|:---|:---|:---|:---|
| @planner | idle | Phase 1 plan done | **PLAN-QUEUE-SYNC-003** then dispatch |
| @coder | RT-ENG-001 | **done** on disk | **FIRE-FUEL-COUNTERS-001** |
| @coder-mcp | idle | Lane K done | **maintain** · optional MCP-OPS |
| @coder A/B/C | VM/WX/IND picks | **all done** | **idle** until Phase 2 assign |
| @designer | idle | design docs signed | **DESIGN-WX-HUD-IMPL-001** |
| @sim-steward | FIRE-STREAM | witness green | **FIRE-STREAM-CLOSE-001** |
| Operator | G-PLAY | still open | **G-PLAY-01** |

### Gates

| Gate | Status |
|:---|:---|
| G-CONTAIN-01 | 🟢 closed |
| G-STAB-01 | 🟢 closed |
| G-PLAY-01 | 💬 **OPEN** — blocks PLAN-AUDIT-020 |
| WH-TRACK-B | 🧊 paused |

---

## Phase 2 program — POST-DRAIN-PHASE-2-001

```text
R-SYNC  → queue/HANDOFF/tensor align (planner first)
T-FIRE  → fuel counters + F2-04 + close streaming row
S-INFRA → GPU tile + replay + perf (one primary per cycle)
U-PROD  → weather HUD impl + egui QC impl
V-OPS   → G-PLAY + audit 020
W-MCP   → ops report (P2 defer)
X-DSM   → orchestrator ATL★/RT★ signoff
```

**Cycle 1 (parallel):**

1. @planner — PLAN-QUEUE-SYNC-003 + issue paste blocks below  
2. @sim-steward — FIRE-STREAM-CLOSE-001  
3. @coder — FIRE-FUEL-COUNTERS-001  
4. @designer — DESIGN-WX-HUD-IMPL-001 (spec → @coder C wire)  
5. Operator — G-PLAY-01  

**Cycle 2:** TRIAGE-GPU-TILE-001 · EGUI-QC-IMPL-001 · TRIAGE-REPLAY-001  

---

## §@planner — your session prompt

```text
You are @planner on Rust_engine_template_01 — architecture + dispatch ONLY.
You do NOT implement src/ or tools/mcp/. You ISSUE paste blocks and thin exec plans.

HUB: src/dev/planner_dispatch_prompts_20260608_v1.md
QUEUE: tools/orchestrator/queues/post_drain_phase2_queue.json
HANDOFF: tools/orchestrator/queues/HANDOFF.md (STALE — fix in PLAN-QUEUE-SYNC-003)
LANG: src/dev/agent_lang_v1.md

AUTH SPINE (witness truth 2026-06-08):
  MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK★ ⇢ ATL◐ ⇢ RT◐
  Phase 1 POST-DRAIN: ~95% done on disk — agents idle due to QUEUE DRIFT not lack of work.

YOUR JOB THIS SESSION (in order):
  1. ⟨PLAN-QUEUE-SYNC-003⟩ — mark done rows in continuation + HANDOFF agent table
  2. Issue ONE paste block per agent from §Dispatch blocks below (copy verbatim)
  3. ⟨PLAN-PHASE2-EXEC-001⟩ — thin witness-key exec if any slice needs COMMIT:SPEC
  4. Do NOT re-plan grammar iter, rowhouse prod, or infra E-tail (closed)

SYNC CHECKLIST (PLAN-QUEUE-SYNC-003):
  ☐ continuation: SLICE-TRIAGE-FIRE-STREAM → done (witness green)
  ☐ continuation: SLICE-MD-F2-03 → ready (counters tail open)
  ☐ HANDOFF agent drain table → Phase 2 picks
  ☐ HANDOFF DSM: ATL◐ RT◐ · grammar open-slices table → all 🟢 or remove
  ☐ HANDOFF WEATHER-I → 🟢 (weather_sim green)
  ☐ master_chain_tensor: chains A/B/E/F/H/I phi refresh

DISPATCH ORDER (issue paste blocks):
  1. @sim-steward  FIRE-STREAM-CLOSE-001
  2. @coder         FIRE-FUEL-COUNTERS-001
  3. @designer      DESIGN-WX-HUD-IMPL-001
  4. @coder         TRIAGE-GPU-TILE-001 (cycle 2 if coder busy)
  5. Operator       G-PLAY-01
  6. @orchestrator  DSM-SIGNOFF-001 (after sync)

MARKER TEMPLATE (require from implementers):
  breakpoint: ⟨BP:SHARE⟩
  joint: "Reviewer @planner — witness key matches exec?"
  COMMIT:WIT debug_runs/<file>.json

DO NOT:
  Reopen Stage 5/6 gates
  Unblock warehouse Track B
  Assign grammar/MCP productivity rows (drained)
  Let agents sit idle without a paste block — idle = dispatch failure

EXIT:
  HANDOFF v2.2 changelog row
  Table: agent → paste issued? → witness target
  "Phase 2 Cycle 1 dispatched" or list blockers
```

---

## §Dispatch blocks — copy to each agent

### @sim-steward

```text
You are @sim-steward — simulation steward. Foreground implementation when Task quota fails.

TRACK: POST-DRAIN-PHASE-2 · Lane T-FIRE
SLICE: ⟨FIRE-STREAM-CLOSE-001⟩

CONTEXT: fire_streaming_live.json already green:true on disk. continuation_queue.json
still shows SLICE-TRIAGE-FIRE-STREAM as ready — this is QUEUE DRIFT not missing code.

READ (intent=ref):
  $ref:src/dev/planner_dispatch_prompts_20260608_v1.md§Crosswalk
  $ref:debug_runs/fire_streaming_live.json
  $ref:tools/orchestrator/queues/continuation_queue.json

WORK:
  1. Verify fire_streaming_live.json: green, streaming_wired, neighbor_wake_observed
  2. If green — update continuation row SLICE-TRIAGE-FIRE-STREAM → done
  3. Update post_drain FIRE-STREAM row → done if not already
  4. No new fire streaming logic unless witness regresses on cargo test

VALIDATE:
  cargo test -p proc_A_dine01 --lib fire_streaming fire_ecology
  validate_bevy_report or witness brief — do not read raw cargo log

COMMIT:WIT debug_runs/fire_streaming_live.json (refresh timestamp if needed)
COMMIT:OPS continuation_queue.json status fix

DO NOT: Re-architect chunk streaming · touch src/construction/

EXIT MARKER:
  breakpoint: ⟨BP:SHARE⟩
  joint: "@planner — continuation row closed?"
  ΔWF→@planner for HANDOFF sync
```

### @coder (primary)

```text
You are @coder on Rust_engine_template_01 — Bevy ECS / render / sim in src/.

TRACK: POST-DRAIN-PHASE-2 · Lane T-FIRE
SLICE: ⟨FIRE-FUEL-COUNTERS-001⟩

CONTEXT: fire_ecology_live.json fire_f2_fuel_spread_001.green is true BUT
fuel_spread_counters_wired is false. continuation SLICE-MD-F2-03 still ready.
Close the counters tail — do not re-do ember wiring.

READ (intent=implement):
  $ref:src/dev/fire_ecology_f1_todos.md§F2-03
  $ref:src/dev/plan_territory_matrix_002_v1.md — src/systems/fire/
  $ref:debug_runs/fire_ecology_live.json

WORK:
  1. Wire fuel spread counters into live proof writer
  2. Set fire_f2_fuel_spread_001.fuel_spread_counters_wired: true
  3. Maintain ember_events_emitted ≥ 1 in lib test
  4. After green → FIRE-F2-004-001 (fire_inst vs heat stability) if same session

VALIDATE:
  cargo test -p proc_A_dine01 --lib fire::
  validate_bevy_report — compression 3

COMMIT:WIT debug_runs/fire_ecology_live.json
DO NOT: tools/mcp/ · parallel fire extract (F2-EXTRACT already done)

EXIT:
  breakpoint: ⟨BP:SHARE⟩
  joint: "@planner — SLICE-MD-F2-03 closable?"
```

### @coder (cycle 2 — infra)

```text
You are @coder — infrastructure hardening slice.

TRACK: POST-DRAIN-PHASE-2 · Lane S-INFRA
SLICE: ⟨TRIAGE-GPU-TILE-001⟩

READ:
  $ref:src/dev/stage5_triage_backlog.md§T2
  $ref:prompts/guides/base_finsh_5.md§2
  Playbook: tools/orchestrator/agents/render_pipeline_agent.md

WORK:
  1. Confirm instanced tile debug path authoritative in Simulation
  2. Document gizmo fallback policy in witness (not delete without policy)
  3. Extend stage5_full_app_live.json keys if needed

VALIDATE:
  cargo test -p proc_A_dine01 --lib stage5
  Optional: cargo run -p proc_A_dine01 --release -- --test visual

COMMIT:WIT debug_runs/stage5_full_app_live.json
DO NOT: New representation stack · Stage 5 gate reopen

EXIT: ⟨BP:SHARE⟩ joint @sim-steward if render authority drift
```

### @coder (cycle 2 — egui QC)

```text
You are @coder — Bevy product HUD implementation.

TRACK: POST-DRAIN-PHASE-2 · Lane U-PROD
SLICE: ⟨EGUI-QC-IMPL-001⟩

DESIGN (signed — implement not redesign):
  $ref:docs/archive/2026-06-src-dev/plans/design_aps_bevy_qc_hud_v2.md
  Witness target: debug_runs/aps_bevy_qc_hud_001_v2_live.json

WORK:
  1. Implement v2 footprint + P0 strip per design doc
  2. Separate surface from Tk APS (three-surface rule)
  3. Refresh witness JSON on test pass

VALIDATE:
  cargo test -p proc_A_dine01 --lib aps_bevy_qc
  joint: @designer sign-off on layout

DO NOT: Tk art_pipeline_suite changes

EXIT: COMMIT:WIT aps_bevy_qc_hud_001_v2_live.json
```

### @coder C (paired with designer weather HUD)

```text
You are @coder C — weather territory only.

TRACK: POST-DRAIN-PHASE-2 · Lane U-PROD
SLICE: ⟨DESIGN-WX-HUD-IMPL-001⟩ (implementation half)

READ:
  $ref:docs/archive/2026-06-src-dev/plans/design_weather_player_read_v1.md
  $ref:src/dev/plan_territory_matrix_002_v1.md — src/systems/weather/
  $ref:debug_runs/weather_sim_live.json (maintain green)

WORK:
  1. Wire player-readable weather HUD per designer spec
  2. New witness: debug_runs/weather_hud_player_read_live.json
  3. cross_system_hooks — only if spec requires; do not break green rollup

VALIDATE:
  cargo test -p proc_A_dine01 --lib weather
DO NOT: src/construction/ · tile coupling

EXIT: COMMIT:WIT weather_hud_player_read_live.json
```

### @designer

```text
You are @designer — presentation / HUD specs and review.

TRACK: POST-DRAIN-PHASE-2 · Lane U-PROD
SLICE: ⟨DESIGN-WX-HUD-IMPL-001⟩

CONTEXT: design_weather_player_read_v1.md exists (brief done). Need **implementation
spec delta** for @coder C — layout, tokens, sim-session placement.

READ:
  $ref:docs/archive/2026-06-src-dev/plans/design_weather_player_read_v1.md
  $ref:prompts/designer_questions/weather_player_read_brief_v1.md
  $ref:docs/archive/2026-06-src-dev/plans/bevy_hud_lanes_agent_orders_v1.md

WORK:
  1. Add §Implementation — widget tree, data bindings, witness keys
  2. ΔWF→@coder C with paste block reference
  3. Review egui QC when @coder opens EGUI-QC-IMPL-001

DO NOT: Tk APS implementation · warehouse Track B

EXIT:
  Updated design_weather_player_read_v1.md §Implementation
  joint: "@coder C — bindings clear?"
```

### @coder-mcp (maintain / optional P2)

```text
You are @coder-mcp — tools/mcp/ only.

TRACK: POST-DRAIN-PHASE-2 · Lane W-MCP
STATUS: Lane K (RT registry, spine, atlas brief) COMPLETE — maintain regression.

PRIMARY (if promoted from defer):
  ⟨MCP-OPS-REPORT-001⟩ — ops_intelligence_scan MCP wrapper
  $ref:src/dev/plan_agent_operations_intelligence_v1.md

DEFAULT (if P2 not promoted):
  BLANG:PY -k aps
  BLANG:PY tools/mcp/python/tests/test_agent_doc_read.py
  Report green — no new features

DO NOT: Warehouse Track B · rewrite grammar queue

EXIT: pytest green OR MCP-OPS witness path
```

### @coder A · @coder B (idle — assign on demand)

```text
You are @coder A (infra) / @coder B (economy) — Phase 1 CLOSED.

BLANG:Q+ → **no primary pick** until planner assigns Phase 2 infra slice.

AVAILABLE (planner may assign cycle 2):
  @coder A: TRIAGE-REPLAY-001 support · VM deep maintain
  @coder B: TRIAGE-CONSTRUCTION polish · organic growth maintain

REGRESSION ONLY this cycle:
  @coder A: cargo test -p proc_A_dine01 --lib construction:: transport::
  @coder B: cargo test -p proc_A_dine01 --lib industrial_activation

DO NOT: Re-open INFRA-E tail · ECON-OG-SAVE (closed)

EXIT: "idle — regression green" OR witness from assigned slice
```

### @orchestrator

```text
You are @orchestrator — sequencing only. No production code.

TRACK: POST-DRAIN-PHASE-2 · Lane X-DSM + R-SYNC
SLICE: ⟨DSM-SIGNOFF-001⟩ after @planner PLAN-QUEUE-SYNC-003

WITNESSES (all green on disk):
  rt_registry_001_live.json · rt_lookup_brief_001_live.json
  procedural_tiles_runtime_live.json rt_eng_001
  atl_sign_001 / aps_atlas production path

WORK:
  1. Confirm planner HANDOFF sync landed
  2. Update master_chain_tensor: ATL φ→2, RT φ→2 (undefer)
  3. Issue Cycle 1 paste blocks if @planner has not

DO NOT: Unblock WH-TRACK-B · write code

EXIT: Tensor AUTH line ATL★ RT★ in HANDOFF
```

### Operator

```text
OPERATOR SESSION — ⟨G-PLAY-01⟩

CHECKLIST: src/dev/plan_g_play_close_001_checklist_v1.md
RUNBOOK: docs/archive/2026-06-src-dev/plans/play_scenario_acceptance_runbook_v1.md

Preconditions: release build · no --test visual · no harness seed env

Execute §1–8 · 10 min play minimum · record pass/fail table in checklist
On PASS: sign EXECUTED row → unblocks PLAN-AUDIT-020

Also if time: OPS-F01 perf capture · OPS-F03 stage6 JSON refresh in sim
```

---

## Planner one-line dispatch table (print view)

| Agent | Paste § | Slice | Witness |
|:---|:---|:---|:---|
| @sim-steward | §sim-steward | FIRE-STREAM-CLOSE-001 | fire_streaming_live.json |
| @coder | §coder primary | FIRE-FUEL-COUNTERS-001 | fire_ecology_live.json |
| @coder | §coder infra (c2) | TRIAGE-GPU-TILE-001 | stage5_full_app_live.json |
| @coder | §coder egui (c2) | EGUI-QC-IMPL-001 | aps_bevy_qc_hud_001_v2_live.json |
| @coder C | §coder C | DESIGN-WX-HUD-IMPL-001 | weather_hud_player_read_live.json |
| @designer | §designer | DESIGN-WX-HUD-IMPL-001 | design doc §Implementation |
| @coder-mcp | §coder-mcp | maintain / MCP-OPS | pytest |
| @coder A/B | §coder A/B | regression | — |
| @orchestrator | §orchestrator | DSM-SIGNOFF-001 | tensor |
| Operator | §Operator | G-PLAY-01 | checklist EXECUTED |

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-08 | Phase 2 crosswalk + planner dispatch prompts after Phase 1 drain |
