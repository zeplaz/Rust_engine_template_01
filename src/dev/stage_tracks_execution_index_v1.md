# Stage tracks — execution index `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.2.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@orchestrator` / `@planner` |
| **Sign-off ledger** | [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) ← **truth table** |
| **Designer board** | [`stage_designer_workboard_v1.md`](stage_designer_workboard_v1.md) |
| **Coder board** | [`stage_coder_workboard_v1.md`](stage_coder_workboard_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) |
| **Coder hub** | [`coder_execution_plan_v1.md`](coder_execution_plan_v1.md) |

**Rule:** One **primary track** per cycle. One **secondary** infra or witness row allowed. Witness JSON wins over markdown checkboxes.

---

## Closed gates (do not reopen for feature work)

| Gate | Doc |
|:---|:---|
| Stage 5 FULL_APP | [`stage5_operational_signoff.md`](stage5_operational_signoff.md) |
| Stage 6 virtualization | [`stage6_operational_signoff.md`](stage6_operational_signoff.md) |
| UI Phase 2 + Phase 3 M1–M2 | [`ui_overhaul_plan.md`](ui_overhaul_plan.md) |
| Construction operational | [`construction_invariants.md`](construction_invariants.md) |

---

## Active tracks (plans + agents)

| Track | Plan | Primary agent | Designer? | First slice |
|:---|:---|:---|:---:|:---|
| **Stage 7 Play** | [`stages/stage7_play_plan_v1.md`](stages/stage7_play_plan_v1.md) | `@coder` + operator | `@designer` playtest | **S7P-DESIGN-001** · **S7P-LOG-001** (IND-001 **done**) |
| **VFX Phase 2 closure** (fire + shared proof) | [`stages/vfx_phase2_closure_plan_v1.md`](stages/vfx_phase2_closure_plan_v1.md) · triage [`vfx_triage_v1.md`](vfx_triage_v1.md) | `@coder` ×2 | `@designer` post-review | **WATER-W1-OCEAN-001** · **VX-P0-01** operator fire read |
| **Water VFX closure** | [`stages/water_vfx_closure_plan_v1.md`](stages/water_vfx_closure_plan_v1.md) | `@coder` A+B | **WATER-DESIGN-001** **done** (TUNE) | **WATER-W1-RIVER-001** · **WATER-W2-FOAM-001** |
| **UI Phase 4** | [`stages/ui_phase4_execution_plan_v1.md`](stages/ui_phase4_execution_plan_v1.md) | `@coder` | **D-WP** [`world_preview_d_wp_track_signoff_v1.md`](../prompts/guides/ui/world_preview_d_wp_track_signoff_v1.md) | **UI-WP-LAYOUT-002** |
| **Infra 5.5+** | [`stages/infra_55_execution_plan_v1.md`](stages/infra_55_execution_plan_v1.md) · gate [`vm09_gate_v1.md`](vm09_gate_v1.md) | `@sim-steward` → `@coder` | — | **INFRA-PROJ2-001** (VM09 s1 **done**) |
| **Wave C depth** | [`stages/wave_c_depth_plan_v1.md`](stages/wave_c_depth_plan_v1.md) | `@coder` + operator | — | **WC-DEPTH-001** |
| **Fire sim Phase 7** | [`stages/fire_sim_phase7_plan_v1.md`](stages/fire_sim_phase7_plan_v1.md) | `@sim-steward` + `@coder` | `@planner` LOD | **FIRE7-PREFLIGHT** |
| **Stage 7 Behavioral** | [`stages/stage7_behavioral_plan_v1.md`](stages/stage7_behavioral_plan_v1.md) | `@designer` → `@planner` | **required** | **S7B-DESIGN-001** |

---

## Recommended 6-cycle rhythm

| Cycle | Primary track | Secondary | Milestone |
|:---:|:---|:---|:---|
| 1 | **Water VFX** closure | **UI-SHELL-REFRESH-001** | W-T02…T04 witness |
| 2 | **Fire VFX** tune | Operator PNGs | F-T01…T03 |
| 3 | **UI Phase 4** | — | UI4-DESIGN-001 → LAYOUT-002 |
| 4 | **S7P-DESIGN-001** | — | Stage 7 Play designer SIGNED |
| 5 | **Infra 5.5+** | PERF 60s | VM-09 (S-VM-09 code done; witness refresh) |
| 6 | **Wave C** | — | depth + churn |
| 7+ | **Behavioral / Fire P7** | — | gated |

**Done (audit 2026-05-24):** S7P-IND-001 · P2-VFX-VISUAL-001 · FX-WATER first pass · UI-P3 minimap · UI-WP-LAYOUT-001

---

## Global commands (every coder slice)

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo orchestrate --skip-cargo
```

Product / render witness refresh:

```powershell
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Agent routing

| Agent | Read first |
|:---|:---|
| **@coder** | [`stage_coder_workboard_v1.md`](stage_coder_workboard_v1.md) → track plan |
| **@designer** | [`stage_designer_workboard_v1.md`](stage_designer_workboard_v1.md) |
| **@orchestrator** | [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) |
| **@sim-steward** | Infra + Fire7 preflight sections |
| **@planner** | Behavioral + Fire7 LOD before large sim refactors |
| **@orchestrator** | Pick cycle row; update `continuation_queue.json` status |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.2.0 | 2026-05-24 | Sign-off ledger + designer/coder workboards; audit 2026-05-24 |
| v1.1.0 | 2026-05-24 | Added **FX-WATER** dedicated closure track (not done) |
| v1.0.0 | 2026-05-24 | Initial seven-track execution index |
