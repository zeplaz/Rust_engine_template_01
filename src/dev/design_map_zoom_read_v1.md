# Map zoom ghost — player read `v1` (DESIGN-MAP-ZOOM-READ-001)

| Field | Value |
|:---|:---|
| **Program** | **DESIGN-MAP-ZOOM-READ-001** |
| **Date** | 2026-06-13 |
| **Owner** | `@designer` (charter) · `@planner` thin plan · `@coder` **TRIAGE-MAP-ZOOM-SMOOTH-001** |
| **Verdict** | **PASS** |
| **Parent** | [`plan_product_polish_exec_001_v1.md`](plan_product_polish_exec_001_v1.md) |
| **Witness** | [`debug_runs/design_map_zoom_read_live.json`](../debug_runs/design_map_zoom_read_live.json) |

---

## Problem

Scroll zoom produces a **double-world ghost** — ortho scale commits immediately while pan/tiles lerps or defer dirty. Player sees trails, misaligned ghost, or stale tile raster.

**Acceptance test:** *5× scroll in/out — no visible double-world > 1 frame; ghost footprint tracks pick within 4px after zoom settles.*

---

## 1. Player-visible symptoms

| Symptom | Player words | Likely cause |
|:---|:---|:---|
| **Double world** | "Map leaves a ghost copy when zooming" | Ortho zoom instant · tile raster deferred |
| **Ghost lag** | "Building preview lags behind cursor after zoom" | Pick uses healed viewport late |
| **Pop-in** | "Tiles snap after zoom stops" | Spike defer clears on settle |

---

## 2. Designer recommendation — Option A primary

Align with [`plan_product_polish_exec_001_v1.md`](plan_product_polish_exec_001_v1.md) P1:

| Option | Player experience | Designer verdict |
|:---|:---|:---|
| **A — Snap ortho on zoom band change** | Brief crisp snap; pan still smooth between bands | **Preferred** — predictable RTS read |
| **B — Force tile dirty on band change** | Softer zoom feel; more GPU work | Acceptable fallback if A breaks perf budget |

**Defer:** simultaneous pan lerp + deferred tile dirty without band snap — current failure mode.

---

## 3. Zoom band table (operational)

Reuse constants from [`design_zoom_fire_read_v1.md`](design_zoom_fire_read_v1.md):

| Band | `zoom_alpha` (approx) | Player label | Ghost / pick |
|:---|:---:|:---|:---|
| Strategic | ≤ 0.28 | District | Tile grid preferred · mesh hidden |
| Operational | 0.28–0.55 | City block | Full ghost + sparks band |
| Tactical | ≥ 0.55 | Street | Max detail |

**On band crossing:** ortho + projection refresh **same frame** as zoom input (Option A).

---

## 4. UX during zoom (ghost + crosshair)

| Probe tier | Ghost behavior | Copy |
|:---|:---|:---|
| Green | Footprint locked to pick | — |
| Yellow | 1-frame settle allowed after band change | `Adjusting zoom…` (optional, ≤300ms) |
| Red | Pick Δ ≥ 1 world or ghost Δ ≥ 4px | Debug only — block G-PLAY |

Crosshair: hide during yellow only if ghost Δ > 4px — otherwise keep visible (fire read charter parity).

---

## 5. Witness acceptance (@coder)

Target `debug_runs/map_zoom_coherence_live.json`:

| Field | Pass |
|:---|:---|
| `double_world_frames_max` | ≤ 1 per zoom step |
| `ghost_screen_delta_px_max` | ≤ 4 after settle |
| `pick_delta_world_max` | ≤ 1 |
| `band_snap_same_frame` | true (Option A) |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-13 |
| `@planner` | pending PLAN-MAP-ZOOM-SMOOTH-001 | — |
| `@coder` | pending TRIAGE-MAP-ZOOM-SMOOTH-001 | — |

```text
DESIGN-MAP-ZOOM-READ-001 complete
Unblocks: PLAN-MAP-ZOOM-SMOOTH-001 · TRIAGE-MAP-ZOOM-SMOOTH-001
```
