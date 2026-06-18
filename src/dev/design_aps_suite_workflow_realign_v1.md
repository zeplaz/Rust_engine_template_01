# APS suite-wide workflow realignment `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-SUITE-WORKFLOW-001** |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Helper** | `tools/mcp/art_pipeline_suite/aps_workflow_layout.py` |
| **Assembly detail** | [`design_aps_assembly_workflow_realign_v1.md`](design_aps_assembly_workflow_realign_v1.md) |

---

## Pattern (every tab)

```text
1. workflow_intro     — one-line artist path
2. primary_row        — main action / filter / tier strip
3. work_area          — paned lists, previews, editors (expand=True)
4. file_row           — load · save · validate · ship (where applicable)
5. advanced (collapsed) — manual fallback, agent patch, paths, debug
```

---

## Buildings lane

| Tab | Primary | Work area | Advanced |
|:---|:---|:---|:---|
| **Catalog** | Batch / category filter | Module list \| metadata | — (actions on detail pane) |
| **Materials** | Studio hint | Library \| Material preview (horizontal) | — |
| **Assembly** | Type · district · Generate | Footprint \| Materials \| Inspector | Setup · kit reference |
| **Variants** | New from assembly | Variant list \| layer editor | Agent patch |
| **Atlas** | Run batch · Pack · Refresh · Validate | Atlas preview panel | Setup paths · smoke/debug log |

---

## Landscape lane

| Tab | Primary | Work area | Advanced |
|:---|:---|:---|:---|
| **Presets** | Refresh · Validate | Preset list \| summary | — |
| **Grammar** | Validate schema | Tree \| graph \| node inspector | — |
| **States** | Validate catalog | Axis row + state tree | Extract parity |
| **Atlas** | Same as buildings | Preview-first | Setup paths |

---

## Preview system alignment

| Surface | Role | Fidelity chip |
|:---|:---|:---|
| **Material preview** (Materials tab) | Profile modes: sphere / wall / section | quick |
| **Piece previews** (Assembly inspector) | Module · material · combined · context | quick / layout |
| **Assembly preview** (Assembly inspector) | Whole snapshot 3D | interactive |
| **Atlas preview** (Atlas tab) | UV grid + cell strip | ship chip |

**Authority chain:** Materials tab authors profiles → Assembly assigns to slots → Piece previews read slot → Assembly preview reads snapshot → Variants/Atlas bake.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
