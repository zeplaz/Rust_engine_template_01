# Stage 7 behavioral — full plan `v1` (PLAN-STAGE7-BEHAVIORAL-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-STAGE7-BEHAVIORAL-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — **S7-PLAY CLOSED** · **S7-BEHAV planning ACTIVE** |
| **Track rollup** | [`stage7_behavioral_track_plan_v1.md`](stage7_behavioral_track_plan_v1.md) |
| **Handoff** | [`stages/stage7_behavioral_planner_handoff_v1.md`](stages/stage7_behavioral_planner_handoff_v1.md) |
| **Behavior track** | [`stages/stage7_behavioral_plan_v1.md`](stages/stage7_behavioral_plan_v1.md) |
| **Play track** | [`stages/stage7_play_plan_v1.md`](stages/stage7_play_plan_v1.md) |
| **Designer brief** | [`../prompts/guides/stage7_behavioral_world_designer_brief_v1.md`](../prompts/guides/stage7_behavioral_world_designer_brief_v1.md) |
| **S7P steward** | [`steward_s7p_gate_v1.md`](steward_s7p_gate_v1.md) |
| **Ledger** | [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) |

**No Rust.** Authoritative planner rollup for **Stage 7** after **S7P** sign-off: closed **play product** lanes + active **behavioral** planning and implementation sequence.

---

## What this plan unblocks

| Blocked work | Unblocked when |
|:---|:---|
| **S7B-DESIGN-001** (worksheet) | This plan **SIGNED** + prerequisites green (below) |
| **S7B-PLAN-001** (implementation plan) | Worksheet **SIGNED** |
| **S7B-PREFLIGHT-001** | **S7B-PLAN-001** published |
| **S7B-M1-001** (contract stubs) | Plan + preflight **GO** |
| **S7B-M2/M3** gameplay | M1 landed + **TRIAGE-VM-09-v2** for full comm authority |

**Does not unblock:** Reopening **S7-PLAY** product slices; strategic AI; EW solvers; preview gameplay mutation.

---

## Two tracks (do not collapse)

| Track | ID | Status | Purpose |
|:---|:---|:---|:---|
| **Play product** | **S7-PLAY** / **G-S7P** | **CLOSED** | Industrial + construction + logistics in play — witnesses only |
| **Behavioral product** | **S7-BEHAV** | **PLANNING** | StrategicCommand plane — worksheet → plan → stubs → delay → overlays |

**Operational readiness** (Stage 5/6, construction, industrial) stays **closed**. Behavioral work is a **new product lane**, not a spine regression fix.

---

## Master gate chain

```text
VISUAL SPINE + Wave S/P/C + Stage 6 ops     ☑ (ledger)
        │
        ▼
S7P-DESIGN-001 (play scenario)              ☑ SIGNED 2026-05-24
S7P coder lanes (IND/CON/LOG/STEWARD)       ☑ DONE
G-S7P / stage7_play_live.json             ☑ CLOSED 2026-05-25
        │
        ▼
PLAN-STAGE7-BEHAVIORAL-001 (this plan)    ☑ SIGNED 2026-05-25
        │
        ▼
S7B-DESIGN-001 worksheet                    ☑ SIGNED 2026-05-25 → stage7_behavioral_d_signoff_v1.md
        │
        ▼
S7B-PLAN-001 implementation plan            ☑ SIGNED — stage7_behavioral_implementation_plan_v1.md
        │
        ▼
S7B-PREFLIGHT-001 (sim-steward)             ☑ GO 2026-05-25
        │
        ▼
S7B-M1-001 contract stubs (≤3 files)        ☑ DONE
        │
        ▼
S7B-M2-001 dispatch delay loop              ☑ DONE
S7B-M3-001 recon + logistics overlays         ☑ DONE
stage7_behavioral_live.json                   ☑ CURRENT
```

**Soft gate (gameplay authority):** **TRIAGE-VM-09-v2** — required before mission authority / full comm gameplay; **not** required to **draft** worksheet or publish **S7B-PLAN-001**.

---

## S7-PLAY — closed product lanes (maintain only)

**Exit:** [`stage7_play_scenario_v1.md`](stage7_play_scenario_v1.md) **SIGNED** · [`steward_s7p_gate_v1.md`](steward_s7p_gate_v1.md) **GO (qualified)** · ledger **G-S7P** **CLOSED**.

### Lane map

| Lane ID | Domain | Owner | Status | Evidence |
|:---|:---|:---|:---:|:---|
| **S7P-DESIGN-001** | Operator scenario (steps 1–8) | designer | **SIGNED** | Scenario doc |
| **S7P-IND-001** | Industrial activation | coder | **DONE** | `activation_green: true` |
| **S7P-CON-001** | Construction P9 catalog | coder | **DONE** | `construction_stage_live.json` operational |
| **S7P-LOG-001** | Logistics throughput in play | coder | **DONE** | `logistics_throughput_live.json` → `s7p_log_001_green` |
| **S7P-IND-002** | Grid overload harness (I3-02) | coder | **DONE** | `industrial_i3_02_green` |
| **S7P-STEWARD-001** | Play-exit witness bundle | sim-steward | **DONE** | `s7p_steward_green: true` |
| **S7P-GRID-001** | Optional smelter load | coder | **DONE** (optional) | `s7p_grid_optional_green` — not in steward rollup |
| **S7P-DESIGN-002** | Grid overload UX note | designer | **OPEN** (optional) | toast/tray — no Rust |

### Witness bundle (CURRENT — refresh only)

| File | Key fields | Role |
|:---|:---|:---|
| [`debug_runs/stage7_play_live.json`](../debug_runs/stage7_play_live.json) | `activation_green`, `s7p_steward_green`, `concrete_chain_e2e.production_green` | **G-S7P** primary |
| [`debug_runs/industrial_activation_live.json`](../debug_runs/industrial_activation_live.json) | `activation_green`, `concrete_chain_e2e` | IND lane |
| [`debug_runs/logistics_throughput_live.json`](../debug_runs/logistics_throughput_live.json) | `throughput_green`, `routes_open` | LOG lane |
| [`debug_runs/construction_stage_live.json`](../debug_runs/construction_stage_live.json) | `operational_green` | CON lane |
| [`debug_runs/minimap_compositor_live.json`](../debug_runs/minimap_compositor_live.json) | `logistics_rows > 0` | cross-check |

```powershell
cargo test -p proc_A_dine01 --lib construction industrial_activation s7p_steward stage5
# Optional sim seed:
$env:RUST_ENGINE_STAGE7_PLAY_SEED=1
cargo run -p proc_A_dine01 --release -- --test visual
```

**Forbidden:** Re-seed industrial chain to “fix” green witnesses; second construction execute path; gameplay mutation from World Preview chrome.

---

## S7-BEHAV — active product lanes

**North star:** **StrategicCommand** plane — delayed dispatch, stale intel, logistics stress overlay, move + secure corridor — per designer brief §2.

### Planning lane map

| Lane ID | Deliverable | Owner | Status | Blocks |
|:---|:---|:---|:---:|:---|
| **PLAN-STAGE7-BEHAVIORAL-001** | This full plan + track rollup | planner | **DONE** | — |
| **S7B-DESIGN-001** | Decision worksheet **SIGNED** | designer | **DONE** | **S7B-PLAN-001** |
| **S7B-DESIGN-002** | UX-D HUD hooks (orders-pending, queue timeline) | designer | **OPEN** | M2+ chrome |
| **S7B-DESIGN-003** | Transmission shell note (UX-E03) | designer | **OPEN** (optional) | — |
| **S7B-PLAN-001** | [`stage7_behavioral_implementation_plan_v1.md`](stage7_behavioral_implementation_plan_v1.md) | planner | **DONE** | worksheet |
| **S7B-PREFLIGHT-001** | Steward GO/NO-GO | sim-steward | **DONE** | [`steward_s7b_preflight_gate_v1.md`](steward_s7b_preflight_gate_v1.md) **GO** |
| **S7B-M1-001** | ECS contract stubs + witness writer | coder | **DONE** | preflight GO |
| **S7B-M2-001** | Dispatch delay loop | coder | **DONE** | `dispatch_delay_ticks: 8` |
| **S7B-M3-001** | Recon + logistics overlays | coder | **DONE** | `s7b_m3_green` + minimap rows |

### Implementation phases (S7B-PLAN-001 must detail)

| Phase | ID | Scope | Proof (target) |
|:---|:---|:---|:---|
| **M1** | **S7B-M1-001** | Enums, resources, DTOs in `stage7_ui_shell.rs`; save/queue types | `cargo test --lib stage7` |
| **M2** | **S7B-M2-001** | Fixed-tick dispatch delay; orders-pending state | `stage7_behavioral_live.json` partial |
| **M3** | **S7B-M3-001** | Recon + logistics stress → minimap overlay path | compositor + behavioral JSON |

**Safe in M1 only (per brief §1):** `CommunicationPlane`, `DispatchMessage`, `BeliefRecord`, `IntelConfidence`, `UtilityChannel`, `StrategicOverlayType`, `MissionIntent`.

**Not in M1:** strategic AI, coalition planners, EW solvers, new `MapCameraDesired` writers.

---

## Decision worksheet (S7B-DESIGN-001 template)

**Deliverable path:** [`prompts/guides/stage7_behavioral_decision_worksheet_v1.md`](../prompts/guides/stage7_behavioral_decision_worksheet_v1.md) — designer copies table, picks column, adds **SIGNED** row.

| ID | Topic | v1 default (brief) | Options (pick one) |
|:---|:---|:---|:---|
| **D-S7-01** | First comm plane | StrategicCommand only | + LogisticsHub |
| **D-S7-02** | Overlay v1 | Recon + logistics stress | + EW |
| **D-S7-03** | Mission v1 | Move + secure corridor | + defend |
| **D-S7-04** | Delay model | Fixed ticks (M2 sizes) | Distance-based |
| **D-S7-05** | Intel stale UI | Tray + map tint | Tray only / tint only |
| **D-S7-06** | Explainability | F3 / context tray tab | Panel vs tab |

**Ungate policy:** Worksheet is **PROJ-2 + designer** — do **not** block on **TRIAGE-VM-09-v2** unless a row explicitly requires invert bridge v2.

---

## Prerequisite matrix (honest)

| Gate | S7-PLAY maintain | S7B worksheet | S7B M1 code | S7B M2+ gameplay |
|:---|:---:|:---:|:---:|:---:|
| `stage7_play_live.json` green | **required** ✅ | ✅ | ✅ | ✅ |
| `wave_p_live.json` green | ✅ | ✅ | ✅ | ✅ |
| UI-P4 D-04 / D-07 | ✅ | ✅ | ✅ | ✅ |
| `phase2b_closed` (UI shell) | ✅ | ✅ | ✅ | ✅ |
| VM-09 slice 2 (CODER-B + PROJ2) | ✅ | ✅ | ✅ | ◐ |
| **S7B-DESIGN-001 SIGNED** | — | **required** | **required** | **required** |
| **S7B-PLAN-001** published | — | — | **required** | **required** |
| **S7B-PREFLIGHT GO** | — | — | **required** | **required** |
| **TRIAGE-VM-09-v2** | — | optional | optional | **required** for full comm authority |

---

## Authority map (no drift)

| Layer | Writer | Reader | v1 rule |
|:---|:---|:---|:---|
| Mission / dispatch ECS | `economy/` + new `stage7/` module (per impl plan) | HUD DTOs only | No duplicate mission writers |
| Overlay fields | logistics + recon resources | minimap compositor (M3) | Snapshot readers — no parallel extract |
| Preview / WorldGen | Wave P contract | preview panel | **No** gameplay mutation |
| View pose | `ViewManager` / `view_runtime` | all views | **No** new `MapCameraDesired` from stubs |
| Explainability | per [`simulation_explainability_runbook_v1.md`](../prompts/guides/simulation_explainability_runbook_v1.md) | F3 / tray | Record types in M1 |

---

## Parallel lanes (OK while S7-BEHAV plans)

| Track | Disjoint from S7B stubs? |
|:---|:---:|
| **UI-P3-M4-001** (minimap design M3) | ☑ |
| **OPS-F01** / **WC-D04** | ☑ |
| Fire P7 preflight | ☑ |
| **S7P-DESIGN-002** grid UX note | ☑ |
| **PLAN-WP** / World Preview optional polish | ☑ |

**Coordinate** if one session touches `stage7_ui_shell.rs` + `simulation_shell_phase2` + `dock_shell.rs`.

---

## Future proof schema — `stage7_behavioral_live.json`

**S7B-PLAN-001** must define fields; planner defaults:

| Field | M1 | M2 | M3 |
|:---|:---:|:---:|:---:|
| `behavioral_contract_ok` | target | — | — |
| `dispatch_delay_ticks` | — | target | — |
| `stale_intel_surface` | — | target | — |
| `recon_overlay_enabled` | — | — | target |
| `logistics_stress_overlay_enabled` | — | — | target |
| `s7b_steward_green` | rollup | rollup | rollup |

Envelope: [`debug_run_envelope.rs`](debug_run_envelope.rs) · index: [`debug_runs/agent_debug_index.json`](../debug_runs/agent_debug_index.json).

---

## Copy-paste blocks

### S7B-DESIGN-001 (@designer)

```
Lane: S7-BEHAV — S7B-DESIGN-001
Read: src/dev/stage7_behavioral_full_plan_v1.md § worksheet
      prompts/guides/stage7_behavioral_world_designer_brief_v1.md
Deliver: prompts/guides/stage7_behavioral_decision_worksheet_v1.md — SIGNED
Do NOT: assign egui product pixels in sim; no Rust
Unblocks: S7B-PLAN-001
```

### S7B-PLAN-001 (@planner)

```
Lane: S7-BEHAV — S7B-PLAN-001
Read: src/dev/stage7_behavioral_full_plan_v1.md
      src/dev/stages/stage7_behavioral_plan_v1.md
Prereq: stage7_behavioral_decision_worksheet_v1.md SIGNED
Deliver: src/dev/stage7_behavioral_implementation_plan_v1.md
      phases M1 → M2 → M3; proof schema stage7_behavioral_live.json
Handoff: S7B-PREFLIGHT-001 then S7B-M1-001 @coder
```

### S7B-M1-001 (@coder — blocked)

```
Track: S7-BEHAV — S7B-M1-001
Read: stage7_behavioral_implementation_plan_v1.md (when published)
Prereq: worksheet SIGNED + S7B-PREFLIGHT GO
First: enums/resources ≤3 files; stage7_ui_shell read-only DTO
Do NOT: strategic AI, coalition planners, preview gameplay mutation, MapCameraDesired writers
Verify: cargo test -p proc_A_dine01 --lib stage7
```

### S7-PLAY maintenance (any merge touching IND/CON/LOG)

```powershell
cargo test -p proc_A_dine01 --lib stage5 construction industrial_activation
```

---

## Acceptance — PLAN-STAGE7-BEHAVIORAL-001

| # | Criterion | Met |
|:---:|:---|:---:|
| F1 | S7-PLAY lanes + witness bundle documented | ☑ |
| F2 | S7-BEHAV gate chain through M3 published | ☑ |
| F3 | Worksheet template + ungate policy explicit | ☑ |
| F4 | Prerequisite matrix separates planning vs gameplay | ☑ |
| F5 | Authority map + parallel lanes listed | ☑ |
| F6 | Copy-paste for designer / planner / coder / steward | ☑ |

**Planning exit (S7-BEHAV):** B1 worksheet SIGNED · B2 impl plan exists · B3 UX-D mocks (optional) · B4 witnesses green · B5 no full AI before plan exit — per [`stages/stage7_behavioral_plan_v1.md`](stages/stage7_behavioral_plan_v1.md).

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — PLAN-STAGE7-BEHAVIORAL-001 full plan |
| Designer (S7P) | 2026-05-24 | **S7P-DESIGN-001 SIGNED** |
| Sim-steward (S7P) | 2026-05-25 | **S7P-STEWARD-001 GO** |
| Designer (S7B) | 2026-05-25 | **S7B-DESIGN-001** **SIGNED** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Full plan — post S7P; unblocks S7B-DESIGN-001 |
