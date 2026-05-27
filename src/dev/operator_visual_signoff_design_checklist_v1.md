# DESIGN-OPERATOR-VISUAL-BUNDLE-001 — designer-side B1–B6 checklist `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-OPERATOR-VISUAL-BUNDLE-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS (qualified)** (mirrors operator bundle pass gate; operator still runs `--test visual` to close) |
| **Unblocks** | `PLAN-OPERATOR-VISUAL-BUNDLE-001` lane |
| **Witness** | `debug_runs/stage5_full_app_live.json` → `/readiness/passes`, `/projection_graph/logistics_active_rows`, `/vfx_visual_signoff_001/green` |
| **Do not break** | witness gates remain green: `/readiness/passes == true`, `/projection_graph/logistics_active_rows >= 1`, `/vfx_visual_signoff_001/green == true` |
| **Reference (operator plan)** | `src/dev/operator_visual_signoff_bundle_plan_v1.md` |

---
## Scope
Designer-side checklist mirroring the operator bundle sign-off gate:
B1..B6 criteria should be satisfied during operator `--test visual` execution.

No Rust changes; this is a sign-off convenience record.

---
## B1–B6 checklist (designer mirror)
| # | Criterion | Must observe |
|:---:|:---|:---|
| B1 | Exit code 0 | terminal run ends without crash |
| B2 | `readiness.passes: true` | `/readiness/passes == true` |
| B3 | `projection_graph.logistics_active_rows > 0` | `/projection_graph/logistics_active_rows == 1` |
| B4 | `vfx_visual_signoff_001.green: true` | `/vfx_visual_signoff_001/green == true` |
| B5 | No shader panic through inv 720+ | no VR-01..VR-09 panic |
| B6 | Optional VR-10 teardown clean | if exercised, no leaked resources |

---
## Acceptance checklist (designer)
1. If any criterion fails, mark this bundle as **DEFER** and route to the appropriate visual-run blocker section.

*** End Patch
