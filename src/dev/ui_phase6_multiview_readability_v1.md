# DESIGN-UI-P6-MULTIVIEW-001 — phase6 multiview readability contract `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-UI-P6-MULTIVIEW-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **DEFER** (per-view VM alignment is green, but full `UI-W3-P6-001` witness rollup is currently false in `ui_shell_migration_live.json`) |
| **Unblocks** | `UI-W3-P6-001` (coder B) |
| **Witness (VM alignment)** | `debug_runs/infrastructure_view_isolation_live.json` → `/vm_08/overlay_masks_aligned` and `/vm_11/minimap_cap_respected` and `/infrastructure_view_isolation_green` |
| **Witness (stage6 green)** | `debug_runs/stage6_virtualization_live.json` → `/stage6_virtualization_green` |
| **Witness (cross-file rollup)** | `debug_runs/ui_shell_migration_live.json` → `/ui_w3_p6_001/green` (currently false) |
| **Do not break** | `/vm_08/overlay_masks_aligned` and `/infrastructure_view_isolation_green` |

---
## Scope
Per-view chrome isolation readability for VM-08 / VM-10 / VM-11 (operator spot-check).

PLAY-01 rule (sim HUD vs editor HUD):
- Simulation must show the contracted HUD chrome subset only.
- Editor-specific panels stay editor-only (egui gating), no visual authority bleed.

---
## Per-view chrome isolation readability (VM-08/10/11)
Readability contract:
1. VM-08 overlays align with the same projection coordinate space (no “drift” vs map hole).
2. VM-11 minimap cap is respected (no unintended zoom/cap escalation in that view).
3. In all three views, phase6 chrome does not reuse the wrong buffer/source path.

VM witness expectations:
- VM-08: `/vm_08/overlay_masks_aligned == true`
- VM-11: `/vm_11/minimap_cap_respected == true`

---
## Sim HUD vs editor (what operators see)
Simulation (BaseState::Simulation) shows:
- collapsed command tray baseline
- map hole / minimap chrome contracts
- debug overlays only when their gate allows (F3/F4/etc)

Editor / WorldGen shows:
- additional dev tool panels, egui contexts (not contracted as sim HUD)

---
## Acceptance checklist (designer)
1. VM-08 overlay alignment matches map hole without visible edge jitter.
2. VM-11 respects minimap cap and does not exceed contracted zoom bounds.
3. Sim HUD matches PLAY-01: no editor-only panels appear in simulation.

*** End Patch
