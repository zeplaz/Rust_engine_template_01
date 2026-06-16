# Product polish exec — zoom coherence · MAP-PICK · fire finish `v1`

| Field | Value |
|:---|:---|
| **Program** | `POST-DRAIN-PHASE-4-001` |
| **Slice ID** | **⟨PLAN-PRODUCT-POLISH-001⟩** |
| **Queue** | `tools/orchestrator/queues/post_drain_phase4_queue.json` |
| **Date** | 2026-06-11 |
| **Status** | **SIGNED** (`@planner` 2026-06-11) |
| **Owner** | `@orchestrator` → `@planner` (thin plan) → `@coder` / `@designer` |
| **Not in scope** | MCP art lane · new bpy · Stage 5 gate reopen |

---

## Problem (operator-reported)

1. **Zoom ghostiness** — scroll zoom in/out: world tiles / overlays appear to lag or double (“ghost”) relative to camera.
2. **Cursor vs world** — pick / ghost placement still drifts under zoom and viewport heal (MAP-PICK path).
3. **Fire unfinished** — chunk heat blobs dominate; sparks/smoke weak or absent until extreme zoom; `fire_inst` flicker (VR-05) persists in play.

Witness JSON marks many fire rows **done**; **product** still fails operator acceptance.

---

## Authority spine (do not invert)

```text
⊚ViewProjectionAuthority → ⊚MainWorldCamera (ortho + viewport) → ⊚SimMapProjectionFrame → pick/ghost
⊚FireVisualFrameSet → ⊚FireVisualFramesByView → projection graph / compositor
```

Single writers: [`07-repo-authority-map.md`](../.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md) · pick contract: [`09-sim-map-projection-placement.md`](../.cursor/skills/bevy-simulation-grade/09-sim-map-projection-placement.md).

---

## Root-cause hypotheses (for @planner sign-off)

| Symptom | Likely mechanism | Files |
|:---|:---|:---|
| Zoom ghost | Pan **lerps** (`map_camera_smooth_toward_desired`) while **ortho zoom** commits immediately; tile raster deferred via `TileRasterSpikeFeedback.defer_zoom_dirty` | `map_camera.rs`, `visual_perf_budget.rs`, `tile_world_fallback.rs` |
| Cursor drift | `fixed_w/h` vs `visible_w/h` mix; hole latch vs full-window scissor during zoom | `map_egui_projection.rs`, `placement_debug.rs` |
| Fire weak | LOD tier + `zoom_alpha < 0.28` culls sparks; overlay heat always on; extract cadence / residency | `fire_visual_extract.rs`, `fire_lod_player_read_v1.md`, `visual_run_blockers.md` VR-05 |

---

## Phases

### P0 — Planner thin exec (1 session) ✅

**⟨PLAN-PRODUCT-POLISH-001⟩** — confirm hypotheses, name acceptance probes, no re-architecture.

| Deliverable | Exit |
|:---|:---|
| This doc signed | **🟢 done** — §Planner sign-off (`@planner` 2026-06-11) |
| Probe table | Pick Δ < 1 world · Ghost Δ < 4px · zoom band change ≤ 1 frame ortho lag · tactical sparks visible per FIRE7 table |

### P1 — MAP-PICK + zoom coherence (@coder)

**⟨TRIAGE-MAP-PICK-CLOSURE-001⟩** (P0)

- Run `--test vfx` + `CONSTRUCTION_PLACEMENT_DEBUG=1`; record probe under zoom in/out.
- Fix: `view_px` from healed viewport; manual path uses `visible_w/h` only; pick after `ApplyCameraScissor`.
- **Do not** add second placement writer outside `src/construction/`.

**⟨TRIAGE-MAP-ZOOM-SMOOTH-001⟩** (P1, after pick green)

- Option A: snap ortho/projection on zoom axis while pan still smooth.
- Option B: tiered dirty — zoom band change forces tile dirty even when spike defer active (bounded budget).
- Witness: `debug_runs/map_zoom_coherence_live.json` (new lib writer).

### P2 — Fire product finish (@coder + @sim-steward)

**⟨TRIAGE-FIRE-PRODUCT-001⟩**

- Wire player-read bands from [`fire_lod_player_read_v1.md`](../docs/archive/2026-06-src-dev/plans/fire_lod_player_read_v1.md) at **operational/tactical** default play zoom — not only `--test visual` tactical lock.
- Stabilize `fire_inst` / overlay rev (VR-05, MAP-BLINK-001); refresh `fire_ecology_live.json` + `stage5_full_app_live.json`.
- Smoke bridge follow-up: `plan_wss_smoke_bridge_exec_001_v1.md` witness keys on disk.
- `@sim-steward`: regression `cargo test -p proc_A_dine01 --lib stage5 fire_ecology`.

### P3 — Designer read (@designer)

**⟨DESIGN-ZOOM-FIRE-READ-001⟩** — when player should see heat vs sparks vs smoke during zoom; crosshair/ghost UX when probe yellow.

---

## Acceptance (operator)

```powershell
cargo run -p proc_A_dine01 --release -- --test vfx
# 1) Scroll zoom 5× — no double-world ghost > 2 frames
# 2) Construction mode — white/magenta/green crosshairs aligned (probe green)
# 3) Mid-tactical zoom — visible sparks on active fire front (not heat-only blob)
```

---

## Regression

```text
@coder:  cargo test -p proc_A_dine01 --lib stage5 fire_ecology construction::placement
@sim-steward: maintain fire/replay — no stale FIRE-STREAM picks
BLANG:CARGO --cached --compress 4
```

---

## Planner sign-off

| Field | Value |
|:---|:---|
| **Agent** | `@planner` |
| **Slice** | **⟨PLAN-PRODUCT-POLISH-001⟩** |
| **Verdict** | **SIGNED** — thin exec on existing authority contracts; no re-architecture |
| **Date** | 2026-06-11 |
| **EV/Cx** | **≥ 1.0** — incremental closure on named writers (`ViewProjectionAuthority`, `SimMapProjectionFrame`, `FireVisualFrameSet`); defers clever parallel extract |

### Hypothesis verdict

| Symptom | Verdict | Notes |
|:---|:---|:---|
| Zoom ghost | **CONFIRM** | `map_camera_smooth_toward_desired` (pan lerp) vs immediate ortho zoom + `TileRasterSpikeFeedback.defer_zoom_dirty` in `visual_perf_budget.rs` / `tile_world_fallback.rs` — coherent multi-rate presentation, not random drift |
| Cursor drift (MAP-PICK) | **CONFIRM** | `fixed_w/h` ≠ `visible_w/h` per `$ref:.cursor/skills/bevy-simulation-grade/09-sim-map-projection-placement.md`; hole latch + scissor heal during zoom per VR-10 — fix stays inside existing pick chain after `ApplyCameraScissor` |
| Fire weak / flicker | **CONFIRM** (split) | **Weak:** chunk heat always on; GPU sparks culled at `zoom_alpha < 0.28` (`visual_run_blockers.md` § VR-10) — wire `$ref:docs/archive/2026-06-src-dev/plans/fire_lod_player_read_v1.md` bands at **operational** default play zoom. **Flicker:** VR-05 / MAP-BLINK-001 — separate stabilization slice in P2, not LOD table alone |

**Authority check:** spine `ViewProjectionAuthority → MainWorldCamera → SimMapProjectionFrame → pick/ghost` and `FireVisualFrameSet → FireVisualFramesByView` preserved — no second placement writer, no parallel fire extract.

### Acceptance probes (operator + lib)

| Probe | Threshold | Witness / command |
|:---|:---|:---|
| Pick Δ world | **< 1** | `CONSTRUCTION_PLACEMENT_DEBUG=1` or `--test vfx` · `placement_debug.rs` |
| Ghost screen Δ | **< 4 px** | same probe path |
| Zoom ortho lag | **≤ 1 frame** visible double-world after scroll band change | manual 5× zoom scroll · target `debug_runs/map_zoom_coherence_live.json` |
| Tactical sparks | visible on active fire front at mid-tactical zoom | `$ref:fire_lod_player_read_v1.md` Operational row · not heat-only blob |
| Regression | green | `cargo test -p proc_A_dine01 --lib stage5 fire_ecology construction::placement` |

### Explicit deferrals

| Item | Status |
|:---|:---|
| MCP art lane (`@orchestrator-mcp`, bpy, `tools/mcp/` jobs) | **⏸ deferred** until G-PLAY MAP-PICK φ→🟢 |
| Stage 5 gate reopen / new `STAGE5_TODOS` | **🧊 out of scope** |
| VM-06…11 full infrastructure hardening | **🧊 triage backlog** — only slices named in P1–P2 |

### Phase gate (no re-architecture)

| Phase | Owner | Gate |
|:---|:---|:---|
| P1 ⟨TRIAGE-MAP-PICK-CLOSURE-001⟩ | `@coder` | Pick + ghost probes green before zoom-smooth |
| P1 ⟨TRIAGE-MAP-ZOOM-SMOOTH-001⟩ | `@coder` | Option A **or** B — bounded; no third camera authority |
| P2 ⟨TRIAGE-FIRE-PRODUCT-001⟩ | `@coder` + `@sim-steward` | FIRE7 bands at default play zoom; VR-05 witness refresh |
| P3 ⟨DESIGN-ZOOM-FIRE-READ-001⟩ | `@designer` | **🟢 PASS** — [`design_zoom_fire_read_v1.md`](design_zoom_fire_read_v1.md) |

### Sign-off row

| Role | Verdict | Date | Note |
|:---|:---|:---|:---|
| `@planner` | **SIGNED** | 2026-06-11 | ⟨PLAN-PRODUCT-POLISH-001⟩ |
| `@coder` | **🟡 qualified** | 2026-06-12 | MAP-PICK 🟢 · FIRE-PRODUCT 🟢 · **ΔWF→ MAP-ZOOM** ⚡P0 |
| `@designer` | **PASS** | 2026-06-13 | [`design_zoom_fire_read_v1.md`](design_zoom_fire_read_v1.md) · [`design_minimap_widget_v1.md`](design_minimap_widget_v1.md) · [`design_map_zoom_read_v1.md`](design_map_zoom_read_v1.md) · [`design_fire_play_visibility_v1.md`](design_fire_play_visibility_v1.md) |
| Operator | **partial** | — | G-PLAY placement OK · zoom ghost until MAP-ZOOM |

### Orchestrator paste

```text
⟨POST-DRAIN-PHASE-4-001⟩ 🟡 MAP-PICK★ · FIRE-PRODUCT★ · ZOOM○
ΔWF→@planner ⟨PLAN-MAP-ZOOM-SMOOTH-001⟩ → @coder A ⟨TRIAGE-MAP-ZOOM-SMOOTH-001⟩ ⚡P0
Parallel: @coder B P0-VFX-ZOOM · P0-TERRAIN · @coder MINIMAP-WIDGET-IMPL-001 · Operator G-PLAY 💬
```
