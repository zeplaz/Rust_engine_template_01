# Stage 7 Behavioral — design after Wave P + preview stable `v1`

| Field | Value |
|:---|:---|
| **Track ID** | `S7-BEHAV` |
| **Version** | `1.2.0` |
| **Full plan** | [`../stage7_behavioral_full_plan_v1.md`](../stage7_behavioral_full_plan_v1.md) (**PLAN-STAGE7-BEHAVIORAL-001**) |
| **Track rollup** | [`../stage7_behavioral_track_plan_v1.md`](../stage7_behavioral_track_plan_v1.md) |
| **Status** | **IMPLEMENTATION** — S7B-PLAN-001 **SIGNED**; **S7B-M1** queued |
| **Impl plan** | [`../stage7_behavioral_implementation_plan_v1.md`](../stage7_behavioral_implementation_plan_v1.md) |
| **Designer brief** | [`../../prompts/guides/stage7_behavioral_world_designer_brief_v1.md`](../../prompts/guides/stage7_behavioral_world_designer_brief_v1.md) |
| **UI shell stub** | `src/gui/hud/stage7_ui_shell.rs` (contracts only) |

---

## Hard prerequisites (all required)

| Gate | Witness / doc |
|:---|:---|
| Wave P operational | `wave_p_live.json` in sim |
| UI Phase 4 D-04 | **UI-WP-LAYOUT-002** done |
| S7-PLAY product | `stage7_play_live.json` green |
| Infra VM-09 slice 2 | **TRIAGE-VM-09-CODER-B** + **PROJ2** done |
| Infra VM-09 v2 | **TRIAGE-VM-09-v2** open — soft gate for full comm authority |
| UI Phase 2B | `phase2b_closed` |
| Transmission UX stub note | `UX-E03` in post_stage6 board |

```text
VISUAL SPINE → Wave S → Wave P → Wave C → Stage 6 → Stage 7 Behavioral (full sim)
```

**Safe now:** enums, DTOs, save schemas, queue types — **no** full AI / EW solvers.

---

## North star (MVP v1)

Prove **StrategicCommand** plane: delayed dispatch, stale intel, logistics stress overlay, move + secure corridor mission — per designer brief §2.

---

## @designer instructions (primary owner)

### S7B-DESIGN-001 — Contract worksheet — **DONE SIGNED** (2026-05-25)

**Worksheet:** [`stage7_behavioral_decision_worksheet_v1.md`](../../prompts/guides/stage7_behavioral_decision_worksheet_v1.md) · **Sign-off:** [`stage7_behavioral_d_signoff_v1.md`](../stage7_behavioral_d_signoff_v1.md)

| ID | Pick |
|:---|:---|
| D-S7-01 | **A** StrategicCommand only |
| D-S7-02 | **A** Recon + logistics stress |
| D-S7-03 | **A** Move + secure corridor |
| D-S7-04 | **A** Fixed ticks |
| D-S7-05 | **A** Tray + map tint |
| D-S7-06 | **C** F3 + context tray tab |

**Unblocks:** **S7B-PLAN-001**

### S7B-DESIGN-002 — UX-D HUD hooks

**Read:** [`../../prompts/guides/experience_layer_ux_hud_designer_brief_v1.md`](../../prompts/guides/experience_layer_ux_hud_designer_brief_v1.md) §5

| Deliverable | Maps to |
|:---|:---|
| Orders-pending chrome | Ops strip zone or context tray |
| Command queue timeline | Intel tab extension mock |
| Ghost contact glyph | Minimap + main map legend |

**Do not** assign pixels to egui product shell in sim — use Bevy chrome ([`ui_phase2b_egui_gate_plan_v1.md`](../../prompts/guides/ui/ui_phase2b_egui_gate_plan_v1.md)).

### S7B-DESIGN-003 — Transmission shell note (UX-E03) — **DONE**

[`ux_e03_transmission_shell_note_v1.md`](../ux_e03_transmission_shell_note_v1.md) — behavioral comms vs transmission shell; editor vs sim visibility.

---

## @planner instructions

### S7B-PLAN-001 (after worksheet SIGNED)

Produce `src/dev/stage7_behavioral_implementation_plan_v1.md`:

| Section | Content |
|:---|:---|
| ECS resources | `CommunicationPlane`, `DispatchMessage`, … (stubs) |
| Authority | No duplicate mission writers |
| Schedule | After logistics + viewport resolve |
| Phases | S7B-M1 contracts → S7B-M2 dispatch delay → S7B-M3 overlays |
| Proof | New `debug_runs/stage7_behavioral_live.json` schema |

**Handoff:** `@coder` only for **S7B-M1** stub resources (≤3 files).

---

## @coder instructions

### Do not start until

- S7B-DESIGN-001 **SIGNED** (2026-05-25 — [`stage7_behavioral_d_signoff_v1.md`](../stage7_behavioral_d_signoff_v1.md))
- S7B-PLAN-001 published
- Prerequisites table all ✅

### S7B-M1-001 — Contract stubs only (first coder slice)

```
Track: S7-BEHAV — S7B-M1-001
Read: src/dev/stages/stage7_behavioral_plan_v1.md
      prompts/guides/stage7_behavioral_world_designer_brief_v1.md
Prereq: worksheet SIGNED + S7B-PLAN-001
First: add enum/resources in new module; wire stage7_ui_shell read-only DTO
Do NOT: strategic AI, coalition planners, gameplay mutation in preview
Max files: 3
Verify: cargo test -p proc_A_dine01 --lib stage7 (or module tests)
```

---

## @sim-steward instructions

### S7B-PREFLIGHT-001

When prerequisites claim green, verify:

- Preview does not mutate gameplay ECS
- No new `MapCameraDesired` writers from mission stubs
- Route: GO → planner S7B-PLAN-001 | NO-GO → list blocking gate

---

## Acceptance — Stage 7 Behavioral planning exit

| # | Criterion |
|:---:|:---|
| B1 | Worksheet SIGNED |
| B2 | Implementation plan v1 exists |
| B3 | Designer UX-D mocks attached |
| B4 | Prerequisites witness bundle green |
| B5 | No full sim AI before planning exit |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.2.0 | 2026-05-25 | Link full plan + worksheet draft path |
| v1.1.0 | 2026-05-25 | S7-PLAY closed; link PLAN-STAGE7-BEHAVIORAL-001 |
| v1.0.0 | 2026-05-24 | Gated behavioral track |
