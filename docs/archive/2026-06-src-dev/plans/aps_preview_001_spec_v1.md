# APS-PREVIEW-001 — Selected slot visual previews `v1`

| Field | Value |
|:---|:---|
| **ID** | **APS-PREVIEW-001** |
| **Track** | **A — APS Product** |
| **Priority** | **P1** (after ARCH-MAT-001 enforcement) |
| **Why** | Previews unlock **understanding** — not decoration |
| **Status** | **implemented** (2026-06-03) — [`slot_preview_panel.py`](../../tools/mcp/art_pipeline_suite/slot_preview_panel.py) |
| **Witness** | [`debug_runs/aps_preview_001_slot_live.json`](../../debug_runs/aps_preview_001_slot_live.json) |

---

## Problem

Assembly editor shows `wall_industrial_a` + `steel_panel_01` but not **what am I looking at?**

---

## Panels (four thumbs)

| Panel | Content |
|:---|:---|
| **Module preview** | Isolated GLB ortho (trimesh) or placeholder |
| **Material preview** | `steel_panel_01` on **wall strip + sphere** from albedo |
| **Combined** | Module + material side-by-side with blend |
| **Placement context** | Assembly context strip; footprint grid highlights selected cell |

**Slot hint line:** cell coords + grammar chain (massing, facade, roof, district, seed).

---

## Near-term stack order (planner)

| Priority | ID |
|:---|:---|
| **P0** | **ARCH-MAT-001** — enforce snapshot authority everywhere |
| **P1** | **APS-PREVIEW-001** — this spec |
| **P2** | **APS-MAT-002** — full material browser (not combobox at 300 profiles) |
| **P3** | **APS-MAT-003** — thumbnails + industrial/residential categories |
| **P4** | **GRAMMAR-001** — archetype → massing layer maturity |
| **P5** | **GRAMMAR-002** — facade + roof strategies |
| **P6** | **Warehouse Track B** — spine completion after artists can see module/material/assembly |

Previews before more warehouse work — every B/C task is easier when Blender is not the only viewer.

---

## Follow-ups

- Feed **assembly-level** Bevy/browser PNG into placement context panel after Preview assembly
- Combined preview with real material-on-mesh (worker) when BUILD-WORKER-001 lands
