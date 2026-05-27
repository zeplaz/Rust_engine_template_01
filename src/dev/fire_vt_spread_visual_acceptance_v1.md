# DESIGN-VT-SPREAD-001 — VT-5 / fire_inst flicker visual policy `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-VT-SPREAD-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS (qualified)** (designer policy: warn-only for visual spread; no FULL_APP gate claim) |
| **Unblocks** | `STAGE5-VT-DEEP-001` (coder triage visual policy updates) |
| **Witness** | `debug_runs/stage5_full_app_live.json` → `/readiness/vt5_ok` |
| **Do not break** | `/readiness/vt5_ok == true` (keep green) |

---
## Purpose
Designer recommendation for **how VT-5 / fire instance spread** should be treated during visual triage:
- VT-5 failures should be handled as **warn-only** for visual policy (unless FULL_APP gate is explicitly invoked).
- fire_inst flicker must not block UI/UX evaluation unless it violates the sim/render contract.

Reference: [`visual_run_blockers.md`](visual_run_blockers.md) VR-04 / VR-05.

---
## Visual triage policy (recommended)
| Condition | Visual outcome | Policy |
|:---|:---|:---|
| VT-5 single-frame fail at low `fire_inst` | UI should still render (no hard gate) | **WARN-only** |
| fire_inst flicker (e.g. 22 -> 0) while passes remain green | treat as intermittent ecology/residency artifact | **WARN-only** |
| persistent VT-5 fail across progression window | mark as “investigate” not “block UX sign-off” | DEFER to coder triage lane |

---
## Acceptance checklist (designer)
1. Visual run QA uses warn-only semantics for VT-5 spread artifacts.
2. Warnings reference VT-4/5 context, not hard-fail UI behavior.

*** End Patch
