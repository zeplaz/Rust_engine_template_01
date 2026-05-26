# UI Phase 3 minimap — track naming authority `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **AUTHORITATIVE** — use in queues, witnesses, and PR titles |

---

## Rule (one line)

**Code id `UI-P3-M3-001` = design M2 construction + ecology.** **Design M3 (fog / EW / units) = `UI-P3-M4-001` + `D-MINIMAP-M3`.**

---

## Matrix

| Label | Queue / review ID | Design phase | Witness fields | Status |
|:---|:---|:---|:---|:---:|
| **M2 logistics** | **UI-P3-M2-001** | M2-01 | `logistics_rows`, `ui_p3_m2_green` | **CLOSED** |
| **M2 construction + ecology** | **UI-P3-M3-001** (misleading “M3” in id) | M2-02, M2-03 | `construction_rows`, `ecology_rows`, `ui_p3_m3_green` | **CLOSED** |
| Alt coder name | **UI-P3-M2-CODER-A** | same as M2-02/03 | same as `ui_p3_m3_green` | **CLOSED** |
| **M2 tray bridge** | **UI-P3-M2-TRAY-OPT** | M2-06 | `ui_p3_m2_tray_opt_green` | **CLOSED** |
| **Design M3 overlays** | **UI-P3-M4-001** | M3 FoW + EW (M3-01/02) | `ui_p3_m4_green` | **CLOSED** (units/replay optional) |
| **Design M3 sign-off** | **D-MINIMAP-M3** / **MINIMAP-DESIGN-M3-001** | spec only | — | **SIGNED** |

---

## Do not

| Wrong | Correct |
|:---|:---|
| “UI-P3-M3-001 implements design M3 fog/EW” | **UI-P3-M4-001** implements design M3 |
| “`ui_p3_m3_green` means M3 fog green” | **`ui_p3_m3_green`** = M2 construction + ecology only |
| Reopen **UI-P3-M3-001** for FoW | Use **UI-P3-M4-001** |
| Label `MinimapOverlayMask::construction_heat` as “design M3” in new docs | **M2** channel (code slice **UI-P3-M3-001**) |

---

## Code anchors

| Symbol | Meaning |
|:---|:---|
| `ui_p3_m3_minimap_acceptance_green` | M2 construction + ecology (slice **UI-P3-M3-001**) |
| `ui_p3_m2_minimap_acceptance_green` | Full M2 rollup (logistics + **UI-P3-M3-001** + tray) |
| `seed_minimap_m2_overlay_witness` | Test seed for M2-02/03, not design M3 |

---

## Linked plans

| Doc | Role |
|:---|:---|
| [`ui_phase3_minimap_compositor_full_plan_v1.md`](ui_phase3_minimap_compositor_full_plan_v1.md) | **PLAN-UI-P3-COMPOSITOR-001** — M1+M2+M3 rollup |
| [`ui_phase3_minimap_compositor_plan_v1.md`](ui_phase3_minimap_compositor_plan_v1.md) | M1 spine (APPROVED) |
| [`ui_phase3_minimap_m2_impl_full_plan_v1.md`](ui_phase3_minimap_m2_impl_full_plan_v1.md) | **PLAN-UI-P3-M2-IMPL-001** — unblocks **UI-P3-M2-CODER-A** |
| [`ui_phase3_minimap_m2_impl_plan_v1.md`](ui_phase3_minimap_m2_impl_plan_v1.md) | M2 closure rollup |
| [`minimap_d_m2_signoff_v1.md`](../../../src/dev/minimap_d_m2_signoff_v1.md) | D-MINIMAP-M2 |
| [`minimap_d_m3_signoff_v1.md`](../../../src/dev/minimap_d_m3_signoff_v1.md) | D-MINIMAP-M3 → **UI-P3-M4-001** |
| [`minimap_m3_operational_overlay_spec_v1.md`](minimap_m3_operational_overlay_spec_v1.md) | Design M3 spec |
| [`ui_oh_m3_001_plan_v1.md`](../../../src/dev/ui_oh_m3_001_plan_v1.md) | **PLAN-UI-P3-M3-001** → **UI-OH-M3-001** closure |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Canonical naming — UI-P3-M3-001 ≠ design M3 |
