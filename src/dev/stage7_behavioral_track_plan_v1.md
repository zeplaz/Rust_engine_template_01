# Stage 7 — product lanes track plan `v1` (post S7P sign)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-STAGE7-BEHAVIORAL-001** |
| **Version** | `1.1.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` (rollup) |
| **Status** | **S7-PLAY CLOSED** · **S7-BEHAV PLANNING ACTIVE** |
| **Full plan** | [`stage7_behavioral_full_plan_v1.md`](stage7_behavioral_full_plan_v1.md) — **PLAN-STAGE7-BEHAVIORAL-001** |
| **Handoff** | [`stages/stage7_behavioral_planner_handoff_v1.md`](stages/stage7_behavioral_planner_handoff_v1.md) |
| **Behavior track** | [`stages/stage7_behavioral_plan_v1.md`](stages/stage7_behavioral_plan_v1.md) |
| **Play track** | [`stages/stage7_play_plan_v1.md`](stages/stage7_play_plan_v1.md) |
| **Ledger** | [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) |

**No Rust in this doc.** Maps **Stage 7 product lanes** after **S7P** sign-off and sequences **S7-BEHAV** planning vs implementation.

**Authoritative gate chain:** [`stage7_behavioral_full_plan_v1.md`](stage7_behavioral_full_plan_v1.md).

---

## Executive summary

| Track | Verdict | Next |
|:---|:---|:---|
| **S7-PLAY** (product) | **CLOSED** | Maintain witnesses only — no feature rework |
| **S7-BEHAV** (behavioral) | **PLANNING** | **S7B-DESIGN-001** **SIGNED** → **S7B-PLAN-001** → **S7B-M1-001** stubs |
| **Infra coupling** | **Soft gate** | **TRIAGE-VM-09-v2** for full comm gameplay authority — **not** S7P blocker |

**Operational readiness:** Stage 5/6 + construction + industrial play are **closed**. Behavioral work is a **new product lane**, not a spine regression fix.

---

## S7-PLAY — product lanes (post sign-off)

**Exit:** [`stage7_play_scenario_v1.md`](stage7_play_scenario_v1.md) **SIGNED** · [`steward_s7p_gate_v1.md`](steward_s7p_gate_v1.md) **GO (qualified)** · **G-S7P** ledger row **CLOSED**.

### Lane map

| Lane ID | Domain | Owner | Status | Evidence |
|:---|:---|:---|:---:|:---|
| **S7P-DESIGN-001** | Operator scenario (steps 1–8) | designer | **SIGNED** | Scenario header 2026-05-24 |
| **S7P-IND-001** | Industrial activation | coder | **DONE** | `activation_green: true` |
| **S7P-CON-001** | Construction P9 catalog | coder | **DONE** | `con_e01_p9_green` |
| **S7P-LOG-001** | Logistics throughput in play | coder | **DONE** | `logistics_throughput_live.json` → `s7p_log_001_green: true` |
| **S7P-IND-002** | Grid overload harness (I3-02) | coder | **DONE** | `industrial_activation` lib **5/5** · `industrial_i3_02_green` |
| **S7P-STEWARD-001** | Play-exit witness bundle | sim-steward | **DONE** | `s7p_steward_green: true` |
| **S7P-GRID-001** | Optional smelter load | coder | **DONE** (optional) | `s7p_grid_optional_green` — not in steward rollup |
| **S7P-DESIGN-002** | Grid overload UX note | designer | **OPEN** (optional) | toast/tray — no Rust |

### Witness bundle (maintain — do not reopen track)

| File | Key fields | Verdict |
|:---|:---|:---|
| `stage7_play_live.json` | `production_green`, `activation_green`, `s7p_steward_green` | **CURRENT** |
| `industrial_activation_live.json` | `activation_green`, `concrete_chain_e2e` | **CURRENT** |
| `construction_stage_live.json` | `operational_green` | **CURRENT** |
| `logistics_throughput_live.json` | `throughput_green`, `routes_open > 0` | **CURRENT** |
| `minimap_compositor_live.json` | `logistics_rows > 0` | **CURRENT** |

```powershell
cargo test -p proc_A_dine01 --lib construction industrial_activation s7p_steward
# Optional seed:
$env:RUST_ENGINE_STAGE7_PLAY_SEED=1
cargo run -p proc_A_dine01 --release -- --test visual
```

**Forbidden:** Re-seed industrial chain to “fix” green witnesses; second construction execute path; gameplay mutation from World Preview chrome.

---

## S7-BEHAV — product lanes (active)

**North star:** StrategicCommand plane — delayed dispatch, stale intel, logistics stress overlay, move + secure corridor — per [`stage7_behavioral_world_designer_brief_v1.md`](../prompts/guides/stage7_behavioral_world_designer_brief_v1.md) §2.

### Planning sequence

```text
DONE     S7P sign-off + play witnesses (above)
DONE     Wave P / UI-P4 / UI-P2B / VM-09 slice 2 (parallel-safe)
         │
         ▼
DONE     S7B-DESIGN-001 — decision worksheet SIGNED (2026-05-25)
NOW      S7B-PLAN-001 — planner implementation plan
         │
         ▼
THEN     S7B-PLAN-001 — stage7_behavioral_implementation_plan_v1.md (planner)
         │
         ▼
THEN     S7B-PREFLIGHT-001 — sim-steward GO (no new MapCameraDesired writers)
         │
         ▼
CODER    S7B-M1-001 — contract stubs only (≤3 files)
         │
         ▼
LATER    S7B-M2 dispatch delay · S7B-M3 overlays · stage7_behavioral_live.json
```

### Lane map

| Lane ID | Deliverable | Owner | Status | Blocks |
|:---|:---|:---|:---:|:---|
| **PLAN-STAGE7-BEHAVIORAL-001** | This track plan | planner | **DONE** | — |
| **S7B-DESIGN-001** | `prompts/guides/stage7_behavioral_decision_worksheet_v1.md` | designer | **DONE** | **S7B-PLAN-001** |
| **S7B-PLAN-001** | `src/dev/stage7_behavioral_implementation_plan_v1.md` | planner | **OPEN** | **S7B-M1-001** |
| **S7B-DESIGN-002** | UX-D HUD hooks (orders-pending, queue timeline) | designer | **OPEN** | M2+ chrome |
| **S7B-DESIGN-003** | Transmission shell note (UX-E03) | designer | **OPEN** | optional |
| **S7B-PLAN-001** | `stage7_behavioral_implementation_plan_v1.md` | planner | **BLOCKED** | worksheet **SIGNED** |
| **S7B-PREFLIGHT-001** | Steward GO/NO-GO | sim-steward | **QUEUED** | plan published |
| **S7B-M1-001** | ECS contract stubs | coder | **BLOCKED** | plan + preflight |
| **S7B-M2-001** | Dispatch delay loop | coder | **FUTURE** | M1 |
| **S7B-M3-001** | Recon + logistics overlays | coder | **FUTURE** | M2 |

### Worksheet decisions (template — designer fills)

| ID | Topic | v1 default (brief) |
|:---|:---|:---|
| **D-S7-01** | First comm plane | StrategicCommand only |
| **D-S7-02** | Overlay v1 | Recon + logistics stress |
| **D-S7-03** | Mission v1 | Move + secure corridor |
| **D-S7-04** | Delay model | Fixed ticks (planner sizes in M2) |
| **D-S7-05** | Intel stale UI | Tray + map tint |
| **D-S7-06** | Explainability | F3 / context tray tab |

### Safe now (contracts only)

Per brief §1 — **no** strategic AI, **no** EW solvers:

| Artifact | Location |
|:---|:---|
| `CommunicationPlane`, `DispatchMessage`, `BeliefRecord` | new module + [`stage7_ui_shell.rs`](../gui/hud/stage7_ui_shell.rs) DTOs |
| Save schemas / queue types | planner plan § ECS |
| Explainability record types | align [`simulation_explainability_runbook_v1.md`](../prompts/guides/simulation_explainability_runbook_v1.md) |

---

## Prerequisite matrix (honest)

| Gate | S7-PLAY | S7-BEHAV planning | S7-BEHAV gameplay code |
|:---|:---:|:---:|:---:|
| `stage7_play_live.json` green | **required** ✅ | ✅ | ✅ |
| `wave_p_live.json` green | ✅ | ✅ | ✅ |
| `ui_shell_migration` phase2b | ✅ | ✅ | ✅ |
| VM-09 slice 2 (CODER-B + PROJ2) | ✅ (S7P unblocked) | ✅ draft worksheet | ◐ **TRIAGE-VM-09-v2** before mission authority |
| **S7B-DESIGN-001 SIGNED** | — | **required** | **required** |
| **S7B-PLAN-001** | — | publish | **required** |

**Ungate policy:** Mark **S7B-DESIGN-001** as **PROJ-2 + worksheet** — not “wait for invert bridge” unless worksheet explicitly requires v2.

---

## Authority map

| Layer | Writer | Reader |
|:---|:---|:---|
| Mission / dispatch ECS | `economy/` + new `stage7/` module (per plan) | HUD DTOs only in v1 |
| Overlay fields | logistics + recon resources | minimap compositor (future M3) |
| Preview / WorldGen | Wave P contract | **no** gameplay mutation |
| View pose | `ViewManager` / `view_runtime` | **no** new `MapCameraDesired` writers from stubs |

---

## Parallel lanes (OK while S7-BEHAV plans)

| Track | Disjoint? |
|:---|:---:|
| UI-P3-M4 minimap M3 | ☑ |
| OPS-F01 / WC-D04 | ☑ |
| Fire P7 preflight | ☑ |
| Optional S7P-DESIGN-002 grid UX | ☑ |

**Coordinate** if touching `stage7_ui_shell.rs` + `simulation_shell_phase2` same session.

---

## Copy-paste blocks

### S7B-DESIGN-001 (designer)

```
Lane: S7-BEHAV — S7B-DESIGN-001
Read: prompts/guides/stage7_behavioral_world_designer_brief_v1.md
      src/dev/stage7_behavioral_track_plan_v1.md § worksheet
Deliver: prompts/guides/stage7_behavioral_decision_worksheet_v1.md — SIGNED
Do NOT: assign egui product pixels in sim; no Rust
Unblocks: S7B-PLAN-001
```

### S7B-PLAN-001 (planner)

```
Lane: S7-BEHAV — S7B-PLAN-001
Read: src/dev/stages/stage7_behavioral_plan_v1.md
Prereq: S7B-DESIGN-001 worksheet SIGNED
Deliver: src/dev/stage7_behavioral_implementation_plan_v1.md
      phases M1 stubs → M2 delay → M3 overlays
      proof schema: debug_runs/stage7_behavioral_live.json
Handoff: S7B-PREFLIGHT-001 then S7B-M1-001 @coder
```

### S7B-M1-001 (coder — blocked)

```
Track: S7-BEHAV — S7B-M1-001
Read: stage7_behavioral_implementation_plan_v1.md (when published)
Prereq: S7B-DESIGN-001 SIGNED + S7B-PREFLIGHT GO
First: enums/resources ≤3 files; stage7_ui_shell read-only DTO
Do NOT: strategic AI, coalition planners, preview gameplay mutation
Verify: cargo test -p proc_A_dine01 --lib stage7
```

---

## Regression (S7-PLAY maintenance)

After any merge touching construction / industrial / transport:

```powershell
cargo test -p proc_A_dine01 --lib stage5 construction industrial_activation
```

Behavioral slices add `stage7` module tests when M1 lands — **not** required for S7-PLAY closure.

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — PLAN-STAGE7-BEHAVIORAL-001 |
| Designer (S7P) | 2026-05-24 | **S7P-DESIGN-001 SIGNED** |
| Sim-steward (S7P) | 2026-05-25 | **S7P-STEWARD-001 GO** |
| Designer (S7B) | — | **S7B-DESIGN-001** pending |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-25 | Link full plan — PLAN-STAGE7-BEHAVIORAL-001 |
| v1.0.0 | 2026-05-25 | Post S7P sign — product lanes + S7-BEHAV planning queue |
