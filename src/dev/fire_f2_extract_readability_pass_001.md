# DESIGN-F2-EXTRACT-READ-001 — Fire F2 extract readability pass `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-F2-EXTRACT-READ-001** |
| **Coder lane** | **FIRE-F2-EXTRACT-001** |
| **Plan** | [`plan_fire_f2_extract_exec_001_v1.md`](plan_fire_f2_extract_exec_001_v1.md) |
| **Witness** | `debug_runs/stage5_full_app_live.json` |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | F2 follow-on polish + smoke bridge readability checks |
| **No Rust** | Readability verification only |

---

## Pass criteria hit

From current full-app witness:

- `f2_extract_witness.green == true`
- `f2_extract_witness.fire_instance_buffer_rows > 0` (observed: `1`)
- `projection_graph.fire_instance_buffer_rows > 0` (observed: `1`)
- `tactical_vfx_witness.fire_instance_buffer_rows_gt_0 == true`
- `tactical_vfx_witness.fire_projection_graph_native == true`

This satisfies DESIGN-F2-EXTRACT-READ-001: tactical fire instances are present and readable in the production extract path.

---

## Readability pass notes

| Surface | Result |
|:---|:---|
| Tactical map (WorldMain/SimulationMap) | PASS — instance rows present and stable with tactical witness green |
| Strategic zoom | PASS — remains governed by tactical/strategic cull policy; no forced bleed |
| Overlay layering | PASS — no evidence of fire instance starvation when logistics/ecology rows are present |

---

## Do not break

- `debug_runs/stage5_full_app_live.json`:
  - `/f2_extract_witness/green`
  - `/f2_extract_witness/fire_instance_buffer_rows`
  - `/projection_graph/fire_instance_buffer_rows`
  - `/tactical_vfx_witness/fire_instance_buffer_rows_gt_0`

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-05-27 |
