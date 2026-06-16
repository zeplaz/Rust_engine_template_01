# MAP zoom smooth — eliminate ghost `v1` (PLAN-MAP-ZOOM-SMOOTH-001)

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **PLAN-MAP-ZOOM-SMOOTH-001** |
| **Parent** | ⟨POST-DRAIN-PHASE-4-001⟩ · [`plan_product_polish_exec_001_v1.md`](plan_product_polish_exec_001_v1.md) |
| **Owner** | `@planner` sign → `@coder` implement |
| **Status** | **READY** — planner sign-off pending |
| **Coder slice** | **⟨TRIAGE-MAP-ZOOM-SMOOTH-001⟩** ⚡P0 |

---

## Problem

Scroll zoom in/out: world tiles and overlays **lag or double** relative to camera (operator “ghostiness”).

**MAP-PICK closed:** `map_pick_closure_001.green` in `construction_stage_live.json` — pick/ghost math is green; zoom coherence is the remaining view defect.

---

## Mechanism (from code review)

| Layer | Behavior | Risk |
|:---|:---|:---|
| Pan | `map_camera_smooth_toward_desired` lerps translation (`SMOOTH_LAMBDA=12`) | World slides while ortho already moved |
| Zoom | Ortho / projection commits from `MapCameraDesired.scale` immediately | Tiles still at old band |
| Tiles | `TileRasterSpikeFeedback.defer_zoom_dirty` skips `mark_all_dirty` when raster hot | Stale chunk textures during zoom bursts |

**Authority:** single writer `ViewProjectionAuthority` → `MainWorldCamera` — no third camera path.

---

## Implementation options (pick one — @planner signs)

### Option A — Snap ortho on zoom axis (recommended default)

- On scroll wheel delta: apply zoom to ortho/projection **same frame**; keep pan lerp for translation only.
- Accept brief pan-only smooth; zoom must not lag.

### Option B — Bounded tile dirty on zoom band change

- When `zoom_band_quantum` crosses: force `mark_all_dirty` even if `defer_zoom_dirty` (cap chunks/frame).
- Keep full pan+zoom lerp; fix raster staleness only.

**Forbidden:** separate zoom smoothing resource that desyncs from `ViewProjectionAuthority`.

---

## Acceptance

| Probe | Green |
|:---|:---|
| Manual 5× scroll zoom | No visible double-world > 1 frame |
| Witness | `debug_runs/map_zoom_coherence_live.json` → `map_zoom_coherence_001.green: true` |
| Regression | MAP-PICK probes stay green (`pick_delta_world_max ≤ 1`) |

```powershell
cargo run -p proc_A_dine01 --release -- --test vfx
cargo test -p proc_A_dine01 --lib map_pick_closure map_zoom_coherence
```

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@planner` | pending | — |
| `@coder` | pending | — |
