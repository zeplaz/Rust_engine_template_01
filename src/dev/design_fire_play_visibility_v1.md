# Fire visibility in normal play `v1` (DESIGN-FIRE-PLAY-VIS-001)

| Field | Value |
|:---|:---|
| **Program** | **DESIGN-FIRE-PLAY-VIS-001** · parent **TRIAGE-FIRE-PLAY-VIS-001** |
| **Date** | 2026-06-13 |
| **Owner** | `@designer` (charter) · `@coder` product path |
| **Verdict** | **PASS** |
| **Source** | [`operator_playtest_report_20260612_v1.md`](operator_playtest_report_20260612_v1.md) |
| **Extends** | [`design_zoom_fire_read_v1.md`](design_zoom_fire_read_v1.md) · [`fire_lod_player_read_v1.md`](../docs/archive/2026-06-src-dev/plans/fire_lod_player_read_v1.md) |
| **Witness** | [`debug_runs/design_fire_play_visibility_live.json`](../debug_runs/design_fire_play_visibility_live.json) |

---

## Problem

Operator sees **no fire or sparks** in normal `cargo run --release` — overlay off by default, empty world, sparks culled at low zoom.

**Acceptance test:** *Start default sim scenario with demo ignition OR enable overlay — active burn shows heat **and** sparks at operational zoom (α≥0.42).*

---

## 1. Default sim presentation

| Layer | Default (Simulation enter) | Override |
|:---|:---|:---|
| Chunk heat overlay | **Off** on main map (minimap same) | Tray toggle `Fire heat` |
| GPU sparks | On when sim has `fire_inst` + band allows | — |
| Smoke | Operational+ when intensity > threshold | zoom band table |
| Starter scenario | **Recommend:** one demo burn within first camera span | scenario script / sim effect |

**Product rule:** Player must see fire **without** opening diagnostics or `--test visual`.

---

## 2. Minimum viable demo path (coder — pick one)

| Path | Designer preference | Notes |
|:---|:---|:---|
| **A — Scenario ignite** | **Preferred** | G-PLAY script lights 1 tile within 30s |
| **B — Tray default on** | Defer | Pink wash risk on minimap — keep off |
| **C — Auto-ignite near camera** | Fallback dev only | Must be seed-deterministic |

Charter: **Path A** for G-PLAY; B unchanged from [`design_sim_hud_minimap_v1.md`](../docs/archive/2026-06-src-dev/plans/design_sim_hud_minimap_v1.md).

---

## 3. Zoom band read (operational default)

At `zoom_alpha ≈ 0.42` (operational play):

| Element | Visible? |
|:---|:---:|
| Heat field (overlay on) | Yes — amber/red read |
| Sparks (`FIRE_SPARK_OPERATIONAL_PLAY_ZOOM_ALPHA`) | **Yes** |
| Smoke plume | Yes if intensity ≥ tier |
| Debug chunk tint only | **No** — product path uses tile/VFX |

---

## 4. HUD copy

| Surface | Text |
|:---|:---|
| Tray toggle | `Fire heat` (unchanged) |
| First burn toast (optional) | `Fire reported — toggle Fire heat on minimap for district view` |
| No active fires | (silent — no error) |

---

## 5. Acceptance

| # | Pass |
|:---:|:---|
| 1 | Scenario or sim effect produces ≥1 `fire_inst` in first 60s play |
| 2 | Sparks visible at operational zoom on that inst |
| 3 | Overlay toggle works without restart |
| 4 | No `--test visual` required |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-13 |
| `@coder` | pending TRIAGE-FIRE-PLAY-VIS-001 | — |

```text
DESIGN-FIRE-PLAY-VIS-001 complete
Unblocks: TRIAGE-FIRE-PLAY-VIS-001
```
