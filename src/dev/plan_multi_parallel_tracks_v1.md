# PLAN-MULTI-PARALLEL-TRACKS-001 — cross-drain parallel dispatch `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-MULTI-PARALLEL-TRACKS-001
Date: 2026-06-20
Status: **ACTIVE** (@planner · @orchestrator)
Owner: @orchestrator (sequencing) · all agents pull rows
Machine board: tools/orchestrator/queues/multi_parallel_tracks_dispatch_v1.json
Wave-0 orders: tools/orchestrator/queues/multi_parallel_tracks_wave0_orders_v1.md
Hub: src/dev/agent_hub_queue_v1.md §Multi-parallel pull
Deferrals: src/dev/plan_deferral_registry_v1.md — evaluate DR-* before picking blocked rows
Migration truth: debug_runs/mig_bevy_019/mig_v1_gate.json (MIG-V1 green)
```

**Headline:** Eight **independent waterfalls** run at the same time. Agents never wait for another track’s primary — they pull the next `ready` row **in their owner lane** on **any** track. Tracks only serialize **within themselves** (wave 0 → 1 → 2). Cross-track work is **`parallel_ok` by default** unless a `cross_track_lock` is listed.

---

## 0. Problem this solves

| Failure mode | Symptom | Fix |
|:---|:---|:---|
| **Global primary** | Power (or APS) blocks everything in HANDOFF | Per-track waterfalls + owner filter |
| **Machine CLOSED ≠ done** | APS UIUX 24/24 but artist 8/10 tails open | Honest rows stay `ready` on dispatch board |
| **Idle on block** | One `depends_on` freezes an agent | Pull next `ready` row on another track (same owner) |
| **Queue sprawl** | 20 JSON files, no cross-lane view | One dispatch rollup + home queue per program |

---

## 1. Operating model

### 1.1 Pull ritual (every session)

```text
BLANG:boot <agent>
  → read multi_parallel_tracks_dispatch_v1.json
  → FILTER owner=<me> status∈{ready,in_progress}
  → SORT track_wave ASC · priority P0<P1<P2
  → PICK top row whose depends_on ⊆ done (within track OR satisfied cross-track)
  → if cross_track_lock hit → skip · pick next ready row
  → work → WIT-HON → WIT → Q✓ on HOME queue row + dispatch row
```

**CLI filter (manual):**

```bash
node .claude/skills/agent-lang/driver.mjs doc tools/orchestrator/queues/multi_parallel_tracks_dispatch_v1.json --max-lines 80
# Home queue Q✓ still required on program queue (designer_active, coder_vegetation_drain, etc.)
```

### 1.2 Waterfall vs parallel

```text
                    ┌─ T1 APS Studio ───── wave0→1→2 ────┐
                    ├─ T2 Grammar Ship ── wave0→1→2 ────┤
GLOBAL PARALLEL     ├─ T3 Veg/Landscape ─ wave0→1→2 ────┤  ← all tracks run together
(no global primary) ├─ T4 Fire Sim ────── wave0→1 ─────┤
                    ├─ T5 Sim HUD ─────── wave0→1→2 ────┤
                    ├─ T6 Power UX ────── wave0→1→2 ────┤
                    ├─ T7 Play/Operator ─ wave0 ────────┤
                    └─ T8 Infra/Perf ──── wave0→1 ──────┘
                              │
                    per-track: seq N blocks N+1 only
                    cross-track: parallel_ok unless cross_track_lock
```

### 1.3 Agent ownership (hard territory)

| Agent | Owns rows on tracks | Territory (do not collide) |
|:---|:---|:---|
| **@designer** | T1, T5 (spec), T7 (rubric spec) | `src/dev/design_*.md` only — no Python/Rust |
| **@designer-mcp** | T2 (QC/spec), T3 (atlas sign-off) | AssetSpec + DMCP docs — no Bevy/Tk impl |
| **@coder-mcp** | T1 (APS impl), T2 (grammar MCP), T3 (E4 bake) | `tools/mcp/` — **one `app.py` editor per session** |
| **@coder** / **@coder A** | T4, T5 (HUD wire), T6 (power sim), T8 (infra A) | `src/` sim/construction/infra — see territory matrix |
| **@coder B** | T3 (minimap), T5 (HUD product), T6 (overlay) | `src/gui/` product HUD — not APS Tk |
| **@operator** | T7 only | NEEDS-DISPLAY sessions — no code |
| **@sim-steward** | T8 triage | Read-only route unless acting as @coder |
| **@orchestrator-mcp** | Pause/resume only | Never implements — issues wave orders |

**Territory matrix:** $ref:src/dev/plan_territory_matrix_002_v1.md

### 1.4 Cross-track locks (only these block parallel)

| Lock ID | Rows affected | Reason |
|:---|:---|:---|
| `LOCK-APP-PY` | All `territory: app.py` coder-mcp rows | Single Tk event loop writer |
| `LOCK-G4-OPERATOR` | `MCP-PILOT-GRAMMAR-001`, warehouse production bake | Operator manual keyframe session |
| `LOCK-WITNESS-STAGE5` | Rows refreshing `stage5_full_app_live.json` | One harness writer per session |
| `LOCK-BLENDER-BATCH` | Sequential bpy jobs on shared Blender | coder-mcp serializes bpy per machine |

If locked, agent picks **next ready row** on a different track (same owner).

### 1.5 Witness + Q✓ rules

- **Dispatch row** `status: done` requires witness on **home queue** row Q✓ too (dual write).
- **WIT-HON** before every Q✓: `validate-report witness_honesty <path> --compress 3`
- **Hardening:** $ref:src/dev/coder_queue_hardening_rules_v1.md — no lib-only green on product rows.
- **Honest scope:** state which gate — schema-green ≠ bake-green ≠ operator-green.

---

## 2. The eight tracks

### T1 — APS Studio Polish (`APS-STUDIO`)

**Goal:** Artist opens APS and feels a **coherent studio** (9/10 target) — not “engineer control panel.”

**Home queues:** `designer_active_queue.json` · `grammar_continuation_queue.json` (OVR-P5*) · `aps_uiux_overhaul_queue.json` (closed — tails only)

**Plan authority:** $ref:src/dev/plan_designer_work_202606_v1.md Track A · $ref:src/dev/plan_aps_uiux_overhaul_20260616_v1.md (ignore “DRAIN P3” — closed)

| Wave | ID | Owner | Status | Depends (in-track) | Deliverable / witness |
|:---:|:---|:---|:---|:---|:---|
| 0 | **DES-APS-INTERACTION-001** | designer | **ready** | — | Interaction spec — feedback, disabled reasons, spine affordance |
| 0 | **DES-APS-ONBOARD-SPEC-002** | designer | **ready** | — | First-10s onboarding from outline |
| 0 | **DES-APS-PREVIEW-LADDER-001** | designer | **ready** | — | G0→G4 preview fidelity ladder |
| 0 | **DES-APS-MANUAL-FALLBACK-001** | designer | **ready** | — | Manual footprint lane deprecation banner |
| 0 | **OVR-P5-TAIL-001** | coder-mcp | **ready** | DS-V11 ✓ | status_atom migration tail — all panels |
| 1 | **DES-APS-OPERATOR-RUBRIC-002** | designer | **ready** | INTERACTION + ONBOARD | Pixel walk v2 → operator |
| 1 | **OVR-P55-PREVIEW-002** | coder-mcp | **ready** | PREVIEW-V2 ✓ + P5-TAIL | 4-state preview impl |
| 1 | **OVR-P56-ONBOARD-001** | coder-mcp | **ready** | ONBOARD spec + P55 | Replace MetadataFlowPanel |
| 2 | **APS-STUDIO-CLOSE-001** | orchestrator-mcp | **ready** | RUBRIC-002 + pytest aps | Rollup witness |

**Regression:** `pytest tools/mcp/python/tests -k aps -q`

**Do not pick:** Re-open `aps_uiux_overhaul_queue` closed phases — new work = Track A IDs above.

---

### T2 — Building Grammar & G4 Ship (`GRAMMAR-SHIP`)

**Goal:** `grammar_pilots ≥ 4` · build-set coverage green · warehouse **production** path unpaused.

**Home queues:** `grammar_continuation_queue.json` · `aps_grammar_evolution_queue.json` (closed infra)

**Plan authority:** $ref:src/dev/plan_building_grammar_evolution_v1.md · $ref:src/dev/plan_industrial_facility_grammar_suite_v1.md

| Wave | ID | Owner | Status | Depends | Deliverable |
|:---:|:---|:---|:---|:---|:---|
| 0 | **CODER-PILOT-REFACTOR-001** | coder | **ready** | — | Remove warehouse-only Rust branches |
| 0 | **CMCP-GRAMMAR-FACILITY-BRIEF-001** | coder-mcp | **ready** | facility specs ✓ | Grammar + catalog + chain brief tool |
| 0 | **CMCP-SITE-ZONE-VALIDATE-001** | coder-mcp | **ready** | site zone spec ✓ | Grid validator |
| 0 | **CMCP-GRAM-SWEEP-PROCESS-001** | coder-mcp | **ready** | — | Process histogram in eval sweep |
| 0 | **GRAM-CONTENT-005** | coder-mcp | **ready** | civic concept ✓ | `civic_block_v1.ron` on disk |
| 1 | **MCP-PILOT-GRAMMAR-001** | designer-mcp | **paused** | **LOCK-G4-OPERATOR** | Warehouse G4 manual keyframes |
| 1 | **BUILD-READ-VISUAL-002** | coder-mcp | **blocked** | PILOT-GRAMMAR / shape pilot | Keyframe batch ship |
| 2 | **GRAMMAR-SHIP-CLOSE-001** | orchestrator-mcp | **ready** | pilots≥4 + coverage | OPS build-set green |

**Ops gate:** `ops_get_build_set_health` — `grammar_pilot_count: 0` is **P0 honest red**.

**Parallel OK with T1, T3** — different territory (Rust grammar vs Tk APS).

---

### T3 — Veg + Landscape Art Ship (`VEG-SHIP`)

**Goal:** Player-visible ecology · landscape atlas **`ship:true`** · burn/scar in catalog.

**Home queues:** `parallel_wave_aps_veg_dispatch_v1.json` · `coder_vegetation_drain_queue.json` · `mcp_aps_evolution_queue.json`

**Honest authority:** $ref:src/dev/vegetation_system_honest_status_v1.md

| Wave | ID | Owner | Status | Depends | Deliverable |
|:---:|:---|:---|:---|:---|:---|
| 0 | **APS-EVO-E4-ATLAS-EXPAND-001** | coder-mcp | **ready** | teach batch ✓ | 16 keyframe stills — **`ship:false` stays** |
| 0 | **DMCP-VEG-ATLAS-SHIP-001** | designer-mcp | **done** | LG5 QC ✓ | G4/G5 ship sign-off criteria |
| 0 | **VEG-CATALOG-BURN-ROWS-001** | coder-mcp | **ready** | — | Burn/scar/recovery in `_vegetation_variant_catalog.ron` |
| 0 | **CDR-B-VEG-MINIMAP-LEGEND-UI-001** | coder_b | **ready** | DES-MINIMAP-VEG-LEGEND ✓ | Minimap topology legend wired |
| 1 | **VEG-F01-ATLAS-SHIP-001** | coder_a | **ready** | DMCP-VEG-ATLAS-SHIP | Engine LG-5 consumer |
| 1 | **VEG-F02-BURN-ATLAS-001** | coder-mcp | **ready** | CATALOG-BURN-ROWS | Burn atlas bake |
| 1 | **CDR-A-VISUAL-SMOKE-ECO-001** | coder_a | **ready** | — | `--test visual` ecology capture |
| 2 | **VEG-SHIP-CLOSE-001** | sim-steward | **ready** | F01+F02+operator | `vegetation_program_close` honest |

**G0 rule:** $ref:debug_runs/art_pipeline/landscape_expanded_g0_rules.yaml — `proceed_tile_ship: no` until G4.

**Blocks T7 rollup** until `operator_session_green`.

---

### T4 — Fire Sim Depth (`FIRE-SIM`)

**Goal:** Fuel-linked spread · smoke bridge · steward regress — **not** Stage 5 gate, but sim credibility.

**Home queues:** `post_drain_active_queue.json` · `coder_active_queue.json` · triage

**Plan authority:** $ref:src/dev/plan_fire_f2_extract_exec_001_v1.md · $ref:src/dev/plan_wss_smoke_bridge_exec_001_v1.md · $ref:src/dev/fire_ecology_f1_todos.md

| Wave | ID | Owner | Status | Depends | Deliverable |
|:---:|:---|:---|:---|:---|:---|
| 0 | **FIRE-F2-FUEL-SPREAD-001** | coder_a | **ready** | F2 extract ✓ | Ember + neighbor fuel depletion |
| 0 | **WSS-SMOKE-BRIDGE-001** | coder_a | **ready** | — | WSS smoke substrate bridge |
| 0 | **FIRE-F2-READINESS-ALIGN-001** | coder_a | **ready** | — | `fire_inst` metric vs sim heat stability |
| 1 | **SIM-STEWARD-FIRE-REGRESS-001** | sim-steward | **ready** | fuel spread ✓ | `cargo test stage5 fire` rollup |
| 1 | **OPS-VT5-OPERATOR-001** | operator | **ready** | — | VT-5 flicker visual confirm (NEEDS-DISPLAY) |

**Parallel OK with T3, T5, T6** — different `src/systems/` modules.

---

### T5 — Sim HUD Product Polish (`SIM-HUD`)

**Goal:** Professional build picker · tray · popup discipline · egui theme parity.

**Home queue:** `designer_active_queue.json` (specs done) · implement rows in dispatch

**Plan authority:** $ref:src/dev/plan_sim_hud_professional_polish_v1.md

| Wave | ID | Owner | Status | Depends | Deliverable |
|:---:|:---|:---|:---|:---|:---|
| 0 | **COD-SIM-HUD-EGUI-THEME-001** | coder | **ready** | COHESION spec ✓ | `UiPalette` enforcement |
| 0 | **COD-SIM-HUD-BUILD-PICKER-001** | coder | **ready** | BUILD-PICKER spec ✓ | Rail-anchored picker |
| 0 | **COD-SIM-HUD-TRAY-BUILD-001** | coder | **ready** | TRAY spec ✓ | Tray Build tab body |
| 0 | **COD-SIM-HUD-POPUP-MIGRATE-001** | coder_b | **ready** | POPUP-TIERS spec ✓ | Remove ad-hoc anchors |
| 1 | **DES-SIM-HUD-OPS-002** | designer | **ready** | wave0 impl | Ops strip v2 spec |
| 1 | **COD-SIM-HUD-OPS-002** | coder_b | **ready** | DES-OPS-002 | Ops strip wire |
| 1 | **COD-SIM-HUD-CURSOR-001** | coder_b | **ready** | — | Unified cursor |
| 2 | **SIM-HUD-PHASE2-CLOSE-001** | orchestrator | **ready** | picker+tray+popup | Product polish witness |

**Not APS.** Not egui Assembly QC (lane 4). Sim session HUD only (lane 5).

---

### T6 — Power Grid Construction UX (`POWER-UX`)

**Goal:** Strategic power lines — draw done · read/repair/integration open.

**Home queue:** $ref:tools/orchestrator/queues/power_grid_construction_ux_queue.json

**Plan authority:** $ref:src/dev/plan_power_grid_construction_ux_v1.md

| Wave | Track | Status | Pick |
|:---:|:---|:---|:---|
| A | Construction draw/commit/routers | **done** | — |
| B | Overlay + island + node hover | **ready** | COD-POWER-OVERLAY-RENDER · ISLAND-HIGHLIGHT · DES-NODE-HOVER |
| C | Damage + repair | **ready** | COD-POWER-DAMAGE · REPAIR-QUEUE |
| D | Activation + toast + tool rail | **ready** | COD-UTILITY-ACTIVATION-LINK · TOOL-RAIL |

**Art downstream CLOSED** — do not pull MCP-PWR utility rows except nuclear P2 deferred.

---

### T7 — Play & Operator Acceptance (`PLAY-ACCEPT`)

**Goal:** Close **G-PLAY-01** rollup — sole blocker for veg honest status + product sign-off.

**Home queues:** `post_drain_phase2/3_queue.json` · tensor gates

**Plan authority:** $ref:src/dev/plan_g_play_close_001_checklist_v1.md

| Wave | ID | Owner | Status | Unblocks |
|:---:|:---|:---|:---|:---|
| 0 | **G-PLAY-01** | operator | **ready** | Product rollup |
| 0 | **G-PLAY-OPERATOR-01** | operator | **ready** | Veg `operator_session_green` |
| 0 | **PERF-SHELL-001** | operator | **ready** | Perf triage signal |
| 0 | **DES-APS-OPERATOR-RUBRIC-002** | operator | **ready** | APS 9/10 score (after designer spec) |
| 0 | **OVR-P6-OPERATOR-EYEBALL-001** | operator | **done** | — |

**Rule:** Operator rows are **NEEDS-DISPLAY** — no agent self-certifies pixels.

**This track unblocks T1 close + T3 close** but does not block T2/T4/T5/T6 coding.

---

### T8 — Infra / Perf / Viewport (`INFRA-PERF`)

**Goal:** Infrastructure hardening — **not** operational readiness gate.

**Home:** $ref:src/dev/stage5_triage_backlog.md · $ref:src/dev/post_stage6_active_todos.md Phase C

| Wave | ID | Owner | Status | Notes |
|:---:|:---|:---|:---|:---|
| 0 | **TRIAGE-PERF-SHELL** | coder_b | **ready** | Frame wall / egui cost |
| 0 | **OPS-F01-WC-D04-001** | coder_b | **ready** | Infra slice 3 plan |
| 0 | **VM-09-V2-INVERT-BRIDGE** | sim-steward → coder | **ready** | Slice 2 closed; v2 open |
| 1 | **VM-10-MINIMAP-LOCKSTEP** | coder_a | **ready** | Diagnostics hardening |
| 1 | **VM-11-PREVIEW-AUDIT** | designer+coder | **ready** | Beyond readiness flags |

**Defer until** product slices stable if EV/Cx < 0.5 — but rows stay pullable for infra-focused sessions.

---

## 3. Per-agent pull cheat sheet

### @designer — pick **any** ready wave-0 row

```text
T1: DES-APS-INTERACTION-001 | ONBOARD-SPEC-002 | PREVIEW-LADDER-001 | MANUAL-FALLBACK-001
T5: DES-SIM-HUD-OPS-002 (after wave0 coder lands — or spec ahead)
T6: DES-POWER-NODE-HOVER-001
```

### @designer-mcp

```text
T2: (on-call) MCP-PILOT-GRAMMAR-001 when LOCK-G4-OPERATOR clears
T3: DMCP-VEG-ATLAS-SHIP-001  ← highest leverage for veg art
```

### @coder-mcp

```text
T1: OVR-P5-TAIL-001  (LOCK-APP-PY — one session)
T2: CMCP-GRAM-* facility tools | GRAM-CONTENT-005
T3: APS-EVO-E4-ATLAS-EXPAND-001 | VEG-CATALOG-BURN-ROWS-001
Rule: never ship:true without DMCP-VEG-ATLAS-SHIP sign-off
```

### @coder / @coder A

```text
T2: CODER-PILOT-REFACTOR-001
T4: FIRE-F2-FUEL-SPREAD-001 | WSS-SMOKE-BRIDGE-001
T5: COD-SIM-HUD-BUILD-PICKER | TRAY-BUILD | EGUI-THEME
T6: COD-POWER-ISLAND-HIGHLIGHT | UTILITY-ACTIVATION-LINK | TOOL-RAIL
T3: VEG-F01 | CDR-A-VISUAL-SMOKE-ECO-001
```

### @coder B

```text
T3: CDR-B-VEG-MINIMAP-LEGEND-UI-001
T5: COD-SIM-HUD-POPUP-MIGRATE | OPS-002 | CURSOR-001
T6: COD-POWER-OVERLAY-RENDER-001
T8: TRIAGE-PERF-SHELL | OPS-F01
```

### @operator

```text
T7 ONLY: G-PLAY-01 → G-PLAY-OPERATOR-01 → PERF-SHELL-001
         then DES-APS-OPERATOR-RUBRIC-002 when designer lands
```

---

## 4. Cross-drain examples (how parallel actually works)

### Example A — @coder-mcp Monday

```text
Session 1: OVR-P5-TAIL-001 (T1) — app.py territory
Session 2: APS-EVO-E4-ATLAS-EXPAND-001 (T3) — landscape batch, no app.py
Session 3: CMCP-GRAMMAR-FACILITY-BRIEF-001 (T2) — CLI only
```

No track waited for another. `LOCK-APP-PY` only serializes T1 app.py edits.

### Example B — @coder B blocked on power overlay deps

```text
Primary T6 COD-POWER-OVERLAY blocked? → pick T5 COD-SIM-HUD-POPUP-MIGRATE
                                  or → pick T3 CDR-B-VEG-MINIMAP-LEGEND
                                  or → pick T8 TRIAGE-PERF-SHELL
```

### Example C — Designer never idle

```text
Wave 0 parallel: INTERACTION + ONBOARD + PREVIEW-LADDER + MANUAL-FALLBACK (four specs, any order)
Wave 1: OPERATOR-RUBRIC after INTERACTION+ONBOARD land
```

---

## 5. Orchestrator wave issuance

| Wave | Action | When |
|:---:|:---|:---|
| **W0** | All agents pull wave-0 `ready` rows on **all** tracks | **Now** — see wave0 orders doc |
| **W1** | Auto-unlock when in-track `depends_on` Q✓ | Per track — no global sync |
| **W2** | Track close witnesses | When track tail rows green |

**Prompts (paste per agent):** [`multi_parallel_agent_prompts_v1.md`](multi_parallel_agent_prompts_v1.md)

---

## 6. Success metrics (program level)

| Metric | Target | Track |
|:---|:---|:---|
| APS artist score | 9/10 | T1 |
| `grammar_pilot_count` | ≥ 4 | T2 |
| Landscape `ship:true` | honest G4 | T3 |
| `operator_session_green` | true | T7 |
| Sim HUD picker/tray | wired in play | T5 |
| Power overlay readable | operator T4–T5 | T6 |
| PERF-SHELL spot-check | documented | T8 |

---

## 7. Anti-patterns

| Do not | Do instead |
|:---|:---|
| Set one global PRIMARY in HANDOFF | Point to dispatch board + owner filter |
| Mark APS UIUX queue “done” as APS product done | Pull T1 tail rows |
| Idle when one track blocked | Cross-drain pick same owner |
| Q✓ dispatch row without home queue Q✓ | Dual-write both |
| Flip `ship:true` on landscape | DMCP-VEG-ATLAS-SHIP-001 + G0 rules |
| Re-open closed overhaul phases | New IDs on dispatch board |

---

## 8. File index

| File | Role |
|:---|:---|
| [`multi_parallel_tracks_dispatch_v1.json`](../tools/orchestrator/queues/multi_parallel_tracks_dispatch_v1.json) | Machine pull board (rollup) |
| [`multi_parallel_tracks_wave0_orders_v1.md`](../tools/orchestrator/queues/multi_parallel_tracks_wave0_orders_v1.md) | Copy-paste agent orders |
| [`agent_hub_queue_v1.md`](agent_hub_queue_v1.md) | Human hub — §Multi-parallel |
| [`HANDOFF.md`](../tools/orchestrator/queues/HANDOFF.md) | Session ritual — no single primary |
| Per-program home queues | Witness + Q✓ authority |

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-20 | Initial — 8 parallel tracks · cross-drain pull model |

```text
⟦/PLAN-MULTI-PARALLEL-TRACKS-001⟧  ΔWF→ ALL agents: filter owner · pick ready wave-0 · cross-drain when blocked
```
