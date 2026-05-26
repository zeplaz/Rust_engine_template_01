# Stage 7 behavioral — decision worksheet `v1` (S7B-DESIGN-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **S7B-DESIGN-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |
| **Sign-off record** | [`stage7_behavioral_d_signoff_v1.md`](../../src/dev/stage7_behavioral_d_signoff_v1.md) |
| **Full plan** | [`../../src/dev/stage7_behavioral_full_plan_v1.md`](../../src/dev/stage7_behavioral_full_plan_v1.md) |
| **Brief** | [`stage7_behavioral_world_designer_brief_v1.md`](stage7_behavioral_world_designer_brief_v1.md) |

**No Rust.** **SIGNED** worksheet unblocks **S7B-PLAN-001**.

---

## Decisions

| ID | Topic | v1 default (brief) | Options | **Pick** |
|:---|:---|:---|:---|:---|
| **D-S7-01** | First comm plane | StrategicCommand only | A: StrategicCommand only · B: + LogisticsHub orders | **A** |
| **D-S7-02** | Overlay v1 | Recon + logistics stress | A: Recon + logistics · B: + EW zones | **A** |
| **D-S7-03** | Mission v1 | Move + secure corridor | A: Move + secure corridor · B: + defend | **A** |
| **D-S7-04** | Delay model | Fixed ticks | A: Fixed ticks · B: Distance-based | **A** |
| **D-S7-05** | Intel stale UI | Tray + map tint | A: Both · B: Tray only · C: Map tint only | **A** |
| **D-S7-06** | Explainability | F3 / context tray | A: F3 panel · B: Context tray tab · C: Both | **C** |

---

## Notes

| Topic | Decision |
|:---|:---|
| **EW presentation** | Behavioral overlay v1 = recon + logistics only (**D-S7-02 A**). EW stress on GPU minimap = **UI-P3-M4-001** (design M3), disjoint compositor channel. |
| **VM-09** | Worksheet **not** blocked on **TRIAGE-VM-09-v2** ([`vm09_gate_v1.md`](../../src/dev/vm09_gate_v1.md)). Full comm authority in code waits for invert bridge at **S7B-M2+**. |
| **UX shell** | Stale intel + explainability align with [`experience_layer_ux_hud_designer_brief_v1.md`](experience_layer_ux_hud_designer_brief_v1.md) §5 — overlay tray → intel timeline → command table. |
| **Hub storage** | Brief §3.2 hub-isolated storage is **policy** for planner/coder plans — not a worksheet row. |

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Designer | 2026-05-25 | ☑ **SIGNED** |

**When SIGNED:** notify `@planner` for **S7B-PLAN-001** ([`stages/stage7_behavioral_planner_handoff_v1.md`](../../src/dev/stages/stage7_behavioral_planner_handoff_v1.md)).
