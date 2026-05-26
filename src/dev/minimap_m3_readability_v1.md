# Wave 3 minimap M3 readability — `DESIGN-W3-M3-READ-001` `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-W3-M3-READ-001** |
| **Track** | Wave 3 / Minimap M3 operational readability |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@designer` |
| **Status** | **SIGNED — PASS** |
| **Compositor witness** | `debug_runs/minimap_compositor_live.json` |
| **Behavioral witness** | `debug_runs/stage7_behavioral_live.json` |
| **Primary gates** | `ui_p3_m4_green`, `ui_w3_m3_001.operational_green`, `s7b_m3_green` |

---

## Deliverable intent

This document closes Wave 3 readability review for minimap M3 overlays: fog/ew legibility, operational signal clarity, and Stage 7 overlay-read integration.

Verdict: `SIGNED — PASS`. M3 readability gates are green in compositor and behavioral witnesses.

---

## Readability contract

| Layer | Requirement |
|:---|:---|
| FoW | Clear dimming without erasing tactical orientation |
| EW | Distinct denial tint from FoW and logistics/construction heat |
| Operational blend | Overlay readability preserved in GPU compositor path |
| Stage 7 bridge | Recon/logistics overlay readers produce non-zero readable channels |

---

## Acceptance checklist

| # | Item | Result |
|:---:|:---|:---:|
| 1 | `ui_p3_m4_green` is true | PASS |
| 2 | `ui_w3_m3_001.operational_green` is true | PASS |
| 3 | `ui_w3_m3_001.stage7_operational_green` is true | PASS |
| 4 | `s7b_m3_green` is true | PASS |
| 5 | `s7b_steward_green` is true | PASS |

---

## Verification commands

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor::live_proof::ui_w3_m3_001_tests::ui_w3_m3_001_stage7_operational_witness_refresh
cargo test -p proc_A_dine01 --lib stage7_behavioral
```

---

## Notes for planner/coder

- This deliverable confirms readability and operational witness closure; it does not introduce a parallel minimap extract path.
- Future tuning should preserve existing gate fields so track rollups remain stable.

---

## History

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Initial Wave 3 minimap M3 readability deliverable record |
