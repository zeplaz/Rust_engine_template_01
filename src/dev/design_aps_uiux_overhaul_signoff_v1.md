# APS UI/UX Overhaul — designer sign-off `v1` (OVR-P6-DESIGN-SIGN-001)

| Field | Value |
|:---|:---|
| **Program** | PLAN-APS-UIUX-OVERHAUL-001 |
| **Gate** | **OVR-P6-DESIGN-SIGN-001** |
| **Owner** | `@designer` |
| **Date** | 2026-06-02 |
| **Authority** | [`aps_design_system_v1.md`](aps_design_system_v1.md) · north star §0 |
| **Machine witness** | `debug_runs/aps_uiux_overhaul_close_live.json` |
| **Operator witness** | `debug_runs/aps_uiux_operator_eyeball_live.json` |
| **Verdict** | **PASS WITH NOTES** |

```text
OVR-P6-DESIGN-SIGN-001 Q✓ (machine + designer)
Pixel walk: operator records pass via APS_OPERATOR_EYEBALL_VERDICT=pass
```

---

## Headline

The overhaul delivers on the **north star**: APS reads as a calm artist tool — led by a clickable pipeline spine, plain voice, collapsed explainers, and IA that matches how artists think (Catalog → Materials → Assembly → Variants → Atlas). Machine guards are green (`163` pytest `-k aps`, ban-list `0`). **Operator pixel confirmation** remains the final human gate for preview feel, first-run tone, and MIN-window layout rubric rows that require a display.

---

## Definition of done — designer lens

| # | Criterion | Verdict | Evidence |
|:---:|:---|:---:|:---|
| 1 | P3–P6 coder-mcp rows done + guards green | **pass** | `aps_uiux_overhaul_close_live.json` |
| 2 | Buildings tab order Catalog → Materials → Assembly → Variants → Atlas | **pass** | `test_aps_lane_tab_swap.py` |
| 3 | Landscape Stamp folded into Atlas (4-step spine) | **pass** | `test_aps_domain_router.py` · `test_aps_pipeline_landscape_validity.py` |
| 4 | Clickable pipeline spine — sole “where am I / what’s next” | **pass** | `test_aps_runtime_callbacks.py` · spine spec |
| 5 | MIN 960×600 usable; footprint visible @ 1280×800 | **pass w/ notes** | `test_aps_min_window_layout.py` (display); operator rubric §1–3 |
| 6 | One status language; no jargon / gate IDs in visible strings | **pass** | `test_aps_no_jargon.py` · G0 audit `0` violations |
| 7 | Preview: labelled states, not blank black | **pass w/ notes** | 2×2 thumb grid; full 4-state async contract partial |
| 8 | First-run: how-it-works, not schema dump | **pass w/ notes** | `test_aps_onboarding.py`; metadata collapsed default |
| 9 | Operator eyeball recorded | **pending** | `aps_uiux_operator_eyeball_live.json` — structural pre-pass |

---

## Notes (non-blocking for machine close)

| ID | Note | Severity |
|:---:|:---|:---:|
| N1 | P5 style migration incremental — not every status site uses `status_atom()` yet | P2 |
| N2 | P5.5 preview — 2×2 layout landed; full async-on-select + fidelity labels partial | P2 |
| N3 | P3 AS-1 — Assembly Setup collapsible strip may need operator confirm vs rubric R-P3-11 | P1 |
| N4 | `design_aps_uiux_onboard_spec_v1.md` not authored — implementation followed onboard outline | info |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS WITH NOTES** | 2026-06-02 |
| `@operator` | **pending** pixel walk | — |

```text
OVR-P6-DESIGN-SIGN-001 complete (designer)
Blocks lifted for DMCP-OVR-ARTIST-ACCEPT-001
Program CLOSED only after operator eyeball pass recorded
```
