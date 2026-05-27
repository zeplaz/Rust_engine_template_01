# R4-PLAN-002 — Round 4 multiview ghost presets `v1` (DESIGN-R4-MV-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **R4-PLAN-002** |
| **Design ID** | **DESIGN-R4-MV-001** |
| **Parent** | **PLAN-CONSTRUCTION-R4-001** |
| **Planner spec** | **R4-PLAN-001** — [`construction_round4_corridor_phase_spec_v1.md`](construction_round4_corridor_phase_spec_v1.md) |
| **Baseline** | [`construction_multiview_ghost_readability_v1.md`](construction_multiview_ghost_readability_v1.md) (**DESIGN-CONSTRUCTION-MV-001** **SIGNED**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |
| **Witness** | `debug_runs/construction_stage_live.json` → `construction_r4_mv_ghost_001` |

**No Rust.** Round 4 **delta** on MV ghosts — corridor phase overlays + preset table. Does **not** replace MV-001 tokens.

---

## Executive summary

| Layer | MV-001 (closed) | R4 delta (this doc) |
|:---|:---|:---|
| Road valid/invalid polylines | ☑ | unchanged |
| Zone / footprint ghosts | ☑ | unchanged |
| **Corridor phase on edges** | — | **NEW** — planned / in-progress / completed strip |
| **Legend / tray** | — | **NEW** — phase key beside construction tools |
| Witness | `construction_mv_001` | `construction_r4_mv_ghost_001` |

---

## Corridor phase overlay (SimulationMap + WorldMain)

Drawn **on top of** terrain, **under** tool ghosts (same projection as MV-001).

| Phase | Stroke | Color (RGBA) | Pattern |
|:---|:---:|:---|:---|
| **Planned** | 3px | `#E8B040` @ 180/255 | Dashed 8px / 4px |
| **InProgress** | 4px | `#50A0E8` @ 200/255 | Solid; alpha × `progress` along polyline |
| **Completed** | — | Use `road_committed_color()` | No duplicate overlay |

**Rules:**

- One overlay pass per edge id — no full-map second extract.
- Progress fill grows **head → tail** along edge polyline (design intent; coder may approximate per segment).
- Overlap with **invalid road ghost**: corridor overlay wins on committed topology; ghost valid/invalid still shows for **uncommitted** picks.

---

## Preset table — construction ghost tokens (R4 canonical)

Consolidates MV-001 § Ghost families for Round 4 catalog work.

| Token | RGBA | Use |
|:---|:---|:---|
| `road_valid` | `#50DCB4` @ 140 | Valid segment |
| `road_invalid` | `#F05A5A` @ 160 | Invalid segment |
| `road_committed` | `#2A2C34` @ 230 | Executed road |
| `corridor_planned` | `#E8B040` @ 180 | R4 edge planned |
| `corridor_in_progress` | `#50A0E8` @ 200 | R4 edge building |
| `footprint_valid` | `#308C48` @ 220 | Building valid |
| `footprint_invalid` | `#B43030` @ 240 | Building invalid |
| `control_point` | `#FFE68C` @ 200 | Road CP |

**Single code source:** [`ghost_visual.rs`](../construction/ghost_visual.rs) — designer changes require token table + code sync.

---

## Per-view matrix (R4)

| Surface | Tool ghosts | Corridor phase overlay |
|:---|:---:|:---:|
| **SimulationMap** | ☑ | ☑ |
| **WorldMain** (map hole) | ☑ | ☑ |
| **World Preview** | ✗ | ✗ |
| **Minimap** | heat only | **heat** may dim when `traffic_factor < 1` — no polylines |

---

## Tray / legend (UX)

| Element | Spec |
|:---|:---|
| Placement | Construction command tray footer — 3 swatches + labels |
| Labels | `Planned` · `Building` · `Open` |
| Visibility | Shown when **any** corridor row not Completed **or** road tool active |

---

## Witness — `construction_r4_mv_ghost_001`

| Path | Green when |
|:---|:---|
| `/construction_r4_mv_ghost_001/gate` | `"DESIGN-R4-MV-001"` |
| `/construction_r4_mv_ghost_001/green` | rollup |
| `/construction_r4_mv_ghost_001/corridor_overlay_tokens_wired` | coder aligned to table above |
| `/construction_r4_mv_ghost_001/legend_wired` | tray shows phase key |
| `/construction_r4_mv_ghost_001/mv_001_still_green` | `construction_mv_001.green: true` |

```text
construction_r4_mv_ghost_001.green :=
  mv_001_still_green
  AND corridor_overlay_tokens_wired
  AND legend_wired
```

---

## Acceptance (designer)

| # | Pass | Fail |
|:---:|:---|:---|
| 1 | Planned corridor readable on forest + desert | Invisible dashed line |
| 2 | In-progress shows partial fill direction | Binary on/off only |
| 3 | Completed edge drops R4 overlay (committed road visible) | Double-drawn thick line |
| 4 | MV-001 road valid/invalid still distinct | Regression |
| 5 | No corridor overlay on World Preview | Wrong surface |

**Optional capture:** `assets/ui/construction/r4_corridor_phase_target_v1.png` — not required for **SIGNED**.

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-26 | **SIGNED** — R4 corridor overlay + presets |
| Planner | 2026-05-26 | **SIGNED** — pairs with **R4-PLAN-001** |
| Coder | — | **BLOCKED** until `product_board_open` |

---

## Forbidden

| Pattern | Why |
|:---|:---|
| egui-only corridor draw | DQ-POST-04 |
| New execute from overlay click | Invariants §1–2 |
| Minimap corridor polylines | Wrong scale |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **R4-PLAN-002** / **DESIGN-R4-MV-001** signed |
