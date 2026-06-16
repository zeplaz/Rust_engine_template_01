# Stage 7 behavioral — `S7B-DESIGN-001` sign-off `v1`

| Field | Value |
|:---|:---|
| **Review ID** | **S7B-DESIGN-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@designer` → `@planner` **S7B-PLAN-001** |
| **Status** | **SIGNED — DESIGN GATE** |
| **Worksheet** | [`stage7_behavioral_decision_worksheet_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/stage7_behavioral_decision_worksheet_v1.md) |
| **Brief** | [`stage7_behavioral_world_designer_brief_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/stage7_behavioral_world_designer_brief_v1.md) |
| **Full plan** | [`stage7_behavioral_full_plan_v1.md`](stage7_behavioral_full_plan_v1.md) |

---

## Executive summary

**Stage 7 behavioral (S7-BEHAV)** locks v1 product choices for **command plane**, **overlay family**, **mission types**, **dispatch delay**, **stale-intel UX**, and **explainability** — **no Rust** in this gate.

**Verdict:** ☑ **SIGNED — DESIGN GATE** — unblocks **S7B-PLAN-001** (planner contract + witness field schema).

**VM-09:** Worksheet ungated per [`vm09_gate_v1.md`](vm09_gate_v1.md) — **PROJ-2 + designer**; **TRIAGE-VM-09-v2** required only for **M2+ full comm authority**, not this sign-off.

---

## Signed decisions (worksheet picks)

| ID | Topic | Pick | Rationale |
|:---|:---|:---|:---|
| **D-S7-01** | First comm plane | **A** — StrategicCommand only | Brief §2 MVP — prove authority + delay before LogisticsHub orders |
| **D-S7-02** | Overlay v1 | **A** — Recon + logistics stress | Brief §2 MVP; EW scalar fields ship via **UI-P3-M4-001** minimap, not behavioral overlay v1 |
| **D-S7-03** | Mission v1 | **A** — Move + secure corridor | Brief §2; defend deferred to M3+ mission pack |
| **D-S7-04** | Delay model | **A** — Fixed ticks | **S7B-M2-001** sizes; distance-based deferred |
| **D-S7-05** | Intel stale UI | **A** — Tray + map tint | Diegetic lag visible in HUD tray and strategic map read |
| **D-S7-06** | Explainability | **C** — F3 panel + context tray tab | Tray for operators (intel timeline family); F3 for diagnostics/replay export per explainability runbook |

---

## Rejected for v1 (explicit)

| Option | Why not now |
|:---|:---|
| **D-S7-01 B** — LogisticsHub orders in v1 | Second plane after StrategicCommand delay proof |
| **D-S7-02 B** — EW zones in behavioral overlay v1 | Minimap **UI-P3-M4-001** owns EW stress channel; behavioral M3 = recon + logistics path |
| **D-S7-03 B** — Defend mission | After corridor secure fixture green |
| **D-S7-04 B** — Distance-based delay | After fixed-tick witness stable |
| Full coalition theater AI | Brief §11 non-goals |

---

## Coder / planner handoff

| Next ID | Owner | Deliverable |
|:---|:---|:---|
| **S7B-PLAN-001** | `@planner` | Phase plan + `stage7_behavioral_live.json` field schema |
| **S7B-M1-001** | `@coder` | Contract enums/resources only (no gameplay AI) |
| **S7B-M2-001** | `@coder` | Fixed-tick dispatch + stale intel surface |
| **S7B-M3-001** | `@coder` | Recon + logistics → minimap overlay readers |

**Do not:** new `MapCameraDesired` writers, parallel minimap extract, egui mission authority in sim shell.

---

## §11 Designer sign-off

| # | Item | Done |
|:---|:---|:---:|
| 1 | Worksheet D-S7-01…06 **Pick** column filled | ☑ |
| 2 | Brief §2 MVP alignment | ☑ |
| 3 | VM-09 ungate policy documented | ☑ |
| 4 | Rust / gameplay implementation | ☐ (coder, after **S7B-PLAN-001**) |

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-25 | **SIGNED — DESIGN GATE** |
| Planner | 2026-05-25 | **SIGNED** — [`stage7_behavioral_implementation_plan_v1.md`](stage7_behavioral_implementation_plan_v1.md) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | S7B-DESIGN-001 worksheet signed |
