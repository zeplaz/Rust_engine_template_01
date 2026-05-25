# Stage 7 Behavioral — design after Wave P + preview stable `v1`

| Field | Value |
|:---|:---|
| **Track ID** | `S7-BEHAV` |
| **Version** | `1.0.0` |
| **Status** | **GATED** — do not implement comm authority until prerequisites green |
| **Designer brief** | [`../../prompts/guides/stage7_behavioral_world_designer_brief_v1.md`](../../prompts/guides/stage7_behavioral_world_designer_brief_v1.md) |
| **UI shell stub** | `src/gui/hud/stage7_ui_shell.rs` (contracts only) |

---

## Hard prerequisites (all required)

| Gate | Witness / doc |
|:---|:---|
| Wave P operational | `wave_p_live.json` in sim |
| UI Phase 4 D-04 | **UI-WP-LAYOUT-002** done |
| Infra VM-09 slice 1 | **INFRA-VM09-001** done |
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

### S7B-DESIGN-001 — Contract worksheet (required)

**Template:** extend [`../../prompts/guides/stage7_behavioral_world_designer_brief_v1.md`](../../prompts/guides/stage7_behavioral_world_designer_brief_v1.md) with decision table:

| ID | Decision | Options | Pick |
|:---|:---|:---|:---|
| D-S7-01 | First comm plane | StrategicCommand only / + LogisticsHub | |
| D-S7-02 | Overlay v1 | Recon + logistics stress / + EW | |
| D-S7-03 | Mission v1 | Move + secure corridor / + defend | |
| D-S7-04 | Delay model | Fixed ticks / distance-based | |
| D-S7-05 | Intel stale UI | Tray badge / map tint / both | |
| D-S7-06 | Explainability surface | F3 panel / context tray tab | |

**Deliver:** `prompts/guides/stage7_behavioral_decision_worksheet_v1.md` — **SIGNED** blocks planner.

### S7B-DESIGN-002 — UX-D HUD hooks

**Read:** [`../../prompts/guides/experience_layer_ux_hud_designer_brief_v1.md`](../../prompts/guides/experience_layer_ux_hud_designer_brief_v1.md) §5

| Deliverable | Maps to |
|:---|:---|
| Orders-pending chrome | Ops strip zone or context tray |
| Command queue timeline | Intel tab extension mock |
| Ghost contact glyph | Minimap + main map legend |

**Do not** assign pixels to egui product shell in sim — use Bevy chrome ([`ui_phase2b_egui_gate_plan_v1.md`](../../prompts/guides/ui/ui_phase2b_egui_gate_plan_v1.md)).

### S7B-DESIGN-003 — Transmission shell note (UX-E03)

One-page alignment: how behavioral comms relate to `transmission_media.rs` stub — editor vs sim visibility.

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

- S7B-DESIGN-001 **SIGNED**
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
| v1.0.0 | 2026-05-24 | Gated behavioral track |
