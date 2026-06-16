# PLAN-UI-P3-M3-001 — minimap operational + Stage 7 HUD `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-UI-P3-M3-001** |
| **UI-OH lane** | **UI-OH-M3-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — compositor + Stage 7 HUD bridge **CLOSED** on disk |
| **Naming** | [`ui_phase3_minimap_track_naming_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_minimap_track_naming_v1.md) |
| **OH closure** | [`ui_oh_m3_001_plan_v1.md`](ui_oh_m3_001_plan_v1.md) |
| **Design M3 spec** | [`minimap_m3_operational_overlay_spec_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/minimap_m3_operational_overlay_spec_v1.md) |
| **Stage 7** | [`stage7_behavioral_implementation_plan_v1.md`](stage7_behavioral_implementation_plan_v1.md) (**S7B-PLAN-001**) |
| **Witness** | `debug_runs/minimap_compositor_live.json` · `debug_runs/stage7_behavioral_live.json` (stub until S7B-M1) |

**No Rust in this deliverable.** Rollup for **three related tracks** that share the misleading “M3” label.

---

## Naming guard (read first)

| ID | Design phase | Witness | Status |
|:---|:---|:---|:---:|
| **UI-P3-M3-001** / **UI-OH-M3-001** | **M2** construction + ecology | `ui_p3_m3_green` | **CLOSED** |
| **UI-P3-M4-001** | **Design M3** FoW + EW + units + replay | `ui_p3_m4_green`, tails | **CLOSED** |
| **S7B-M3** (behavioral) | Recon + logistics stress **readers** | `stage7_behavioral_live.json` | **OPEN** (S7B-M1+) |

**Operational minimap (intel picture)** = design spec **D-MINIMAP-M3** implemented under **UI-P3-M4-001**, not **UI-P3-M3-001**.

---

## Track A — M2 construction + ecology (**UI-P3-M3-001**)

| # | Criterion | Witness | 2026-05-25 |
|:---:|:---|:---|:---:|
| A1 | Construction heat | `construction_rows: 18` | ☑ |
| A2 | Ecology heat | `ecology_rows: 100` | ☑ |
| A3 | Rollup | `ui_p3_m3_green` | ☑ |
| A4 | OH gate | `ui_oh_m3_001.green` | ☑ |

**Impl authority:** [`ui_phase3_minimap_m2_impl_full_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_minimap_m2_impl_full_plan_v1.md) (**PLAN-UI-P3-M2-IMPL-001**).

```powershell
cargo test -p proc_A_dine01 --lib ui_p3_m3
```

---

## Track B — Design M3 operational overlays (**UI-P3-M4-001**)

| Channel | Spec § | Witness | 2026-05-25 |
|:---|:---|:---|:---:|
| FoW veil | M3-01 | `fow_rows`, `fow_enabled` | ☑ |
| EW stress | M3-02 | `ew_rows`, `ew_overlay_enabled` | ☑ |
| Units | M3-03 | `ui_p3_m3_units_001_green` | ☑ |
| Replay scrub | M3-04 | `ui_p3_m3_replay_001_green` | ☑ |
| Rollup | — | `ui_p3_m4_green` | ☑ |

**Design sign-off:** [`minimap_d_m3_signoff_v1.md`](minimap_d_m3_signoff_v1.md).

**Do not** fold Track B into **UI-P3-M3-001** queue titles or PR labels.

---

## Track C — Stage 7 HUD + minimap readers (**S7B**)

**Product locks** ([`stage7_behavioral_implementation_plan_v1.md`](stage7_behavioral_implementation_plan_v1.md)):

| Lock | HUD / minimap consequence |
|:---|:---|
| **D-S7-02** | Recon + logistics stress on minimap path — **read** compositor snapshots, no second extract |
| **D-S7-05** | Tray badge + `MapViewInstanceId::Stage7IntelMap` tint |
| **D-S7-06** | F3 explainability + context tray intel tab |

**Existing code:**

| Path | Role |
|:---|:---|
| `src/gui/hud/stage7_ui_shell.rs` | Mock DTO viewers (M1 partial) |
| `src/strategic/comms_contract.rs` | `StrategicOverlayType`, mission DTOs |
| `src/render/minimap_compositor/` | Sole overlay presentation |

**Coder forward (not this plan):**

| Slice | Blocks |
|:---|:---|
| **S7B-PREFLIGHT-001** | steward witness refresh |
| **S7B-M1-001** | contracts + `stage7_behavioral_live.json` scaffold |
| **S7B-M3-001** | overlay readers wired to minimap (after M1) |

**EW on minimap:** **UI-P3-M4-001** only — not behavioral overlay v1.

---

## Combined PASS (planner rollup)

```text
PLAN-UI-P3-M3-001 PASS (qualified) :=
  ui_p3_m3_green AND ui_oh_m3_001.green     -- Track A
  AND ui_p3_m4_green                        -- Track B (design M3)
  AND phase2b_closed                        -- sim shell gate for S7 HUD
  AND (stage7_behavioral_live stub OK OR S7B-M1 landed)
```

**2026-05-25:** Tracks **A+B** green on disk; Track **C** witness stub (`behavioral_contract_ok: false`) — **does not** reopen Tracks A/B.

---

## Forbidden

| Wrong | Correct |
|:---|:---|
| Implement FoW in **UI-P3-M3-001** | **UI-P3-M4-001** |
| Second minimap extract for S7 | Read `MinimapCompositorState` / snapshots |
| egui mission authority in sim | **UI-P2B** gate |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Operational + Stage 7 HUD rollup for **PLAN-UI-P3-M3-001** |
