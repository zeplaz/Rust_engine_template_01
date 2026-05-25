# Stage 6 active todos — virtualization host

**Handoff:** [`stage6_agent_handoff.md`](stage6_agent_handoff.md)  
**Sign-off:** [`stage6_operational_signoff.md`](stage6_operational_signoff.md)  
**Design:** [`stage6_design_decisions.md`](stage6_design_decisions.md)

**Status:** **Stage 6 operational gate CLOSED** (2026-05-23). Refresh witnesses on release train.

---

## S6-0 — Bootstrap ✅ DONE

All S6-00…S6-07 complete.

---

## S6-1 — Residency authoritative ✅ DONE

All S6-10…S6-18 complete — see sign-off.

---

## S6-2 — Atlas / async / per-view ✅ DONE

All S6-20…S6-26 complete — see sign-off.

---

## S6-3 — Exit gate ✅ DONE

| ID | Task | Agent | Status | Proof |
|----|------|-------|--------|-------|
| S6-30 | Live JSON `stage6_readiness.passes` | debug-intelligence | [x] | `stage6_virtualization_live.json` |
| S6-31 | F3 BQ-134 authoritative DTO | debug-intelligence | [x] | `stage6_telemetry.rs`, `dock_shell.rs`, side panel |
| S6-32 | Lib tests | debug-intelligence | [x] | 622+ lib tests |
| S6-33 | Stage 5 spine regression | debug-intelligence | [x] | `stage5_full_app_live.json` (re-run visual on train) |
| S6-34 | Operational sign-off | debug-intelligence | [x] | `stage6_operational_signoff.md` |

---

## Wave S — save spine ✅ DONE

| ID | Task | Status | Proof |
|----|------|--------|-------|
| S6-S1 | Product shell on-disk (`product_shell.ron`) | [x] | `wave_s_artifacts.rs`, capture system |
| S6-S3 | Blueprint presets (`blueprints/presets.ron`) | [x] | `wave_s_blueprint_roundtrip.json` |
| S6-S2 | HUD layout slot (BQ-130) | [x] | `stage6_design_decisions.md` |

See [`wave_s_open.md`](wave_s_open.md).

---

## Parallel lanes (optional)

---

## Launch queue

| Order | Agent | Package | Status |
|-------|-------|---------|--------|
| 1–5 | All S6 packages | A–D | **done** |

**Next program:** [`post_stage6_active_todos.md`](post_stage6_active_todos.md) · [`post_stage6_design_plan.md`](post_stage6_design_plan.md).
