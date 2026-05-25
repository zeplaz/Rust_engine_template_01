# UX-A M1 — GPU minimap compositor architecture (planner slice)

| Field | Value |
|:---|:---|
| **Version** | `1.1.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@planner` (read-only) |
| **Status** | **3.1–3.5 DONE** · witness green 2026-05-24 |
| **Full planner** | [`ui_phase3_gpu_minimap_m1_planner_v1.md`](../prompts/guides/ui/ui_phase3_gpu_minimap_m1_planner_v1.md) |

**Entry:** Phase 3 = [`ui_phase2_coder_queue_v1.md`](../prompts/guides/ui/ui_phase2_coder_queue_v1.md) UX-E01 · P3 = [`ui_phase0_panel_mocks_v1.md`](../prompts/guides/ui/ui_phase0_panel_mocks_v1.md) § P3 · spine = [`map_view_spine.json`](../tools/orchestrator/knowledge/map_view_spine.json)

---

## 1. Authority — minimap pixels vs HudShellSync

| Concern | Single authority | Role |
|:---|:---|:---|
| **Pixel production** | `run_minimap_compositor_pass` → `MinimapRenderTargetRegistry` | Writes `RenderTarget::Image` (M1: CPU RGBA bridge; M1.5: WGSL) |
| **Overlay fields** | `SharedOverlayFieldBuffers` | Fire heat — one producer after `FireVisualFrameSet` |
| **Policy / caps** | `RepresentationResult` + `PerViewRepresentationPolicy` | LOD, overlay matrix, minimap Hz — read-only in compositor |
| **Bevy display** | `MinimapGpuImageNode` (`ImageNode`) | Binds committed handle; does not composite |
| **P3 chrome layout** | `sync_minimap_chrome_root_system` | Positions `MinimapChromeRoot` wire from `last_image_rect` |
| **GPU image layout** | `sync_minimap_gpu_image_node_system` | Sizes child `ImageNode`; publishes `last_image_rect` when GPU path active |
| **Shell UX** | `MinimapShellState` | Zoom, follow, toggles, rects — never extract |
| **egui (fallback)** | `resolve_minimap_egui_texture` | CPU path + controls; **no** world image when `SharedRenderTargetImage` |

**Rule:** Render extract owns pixels on the GPU path. Hud shell sync owns **chrome geometry only** — not raster content. Do not feed pixels through `hud_root_tick` / `MapViewTextureCache` when `SharedRenderTargetImage` is active.

**Fixed in 3.1:** Minimap RT is **not** aliased to `WorldPreviewRenderTargetRegistry` — separate `MinimapRenderTargetRegistry` in `src/render/minimap_compositor/render_target.rs`.

---

## 2. Inputs — no duplicate LOD / extract

Compositor reads published frames only.

| Input | Producer | Use |
|:---|:---|:---|
| `SharedOverlayFieldBuffers` | `SyncOverlayField` after fire extract | Fire heat field |
| `FireVisualFrame` (minimap view) | `FireVisualFramesByView` + projection | Per-view cap channel |
| `RepresentationResult` | `WorldRepresentationResolver` | `overlay_matrix`, budgets, cadence |
| `ViewRepresentationSnapshot` | `build_view_representation_snapshot` | World bounds |
| `TileWorldFallbackState` | Existing fallback raster | Terrain color (M1 bridge source) |
| `ResolvedViewports.minimap_panel` | `ViewportPipelineSet::Resolve` | RT extent (authority) |
| `MinimapOverlayMask` | Shell / `MapViewInstances.minimap` | Fire toggle uniform |

**Forbidden:** `MinimapOnlyExtract`, second fire ECS query, parallel chunk residency pass, shell writes to overlay buffers.

**Revision hash:** `ResolvedMapViewFrames.minimap.projection_revision` = registry revision + overlay revision + panel extent + compositor stamp — **not** global `ResolvedViewports.revision`.

---

## 3. Schedule — PostUpdate vs WorldRender vs `minimap_shell.rs`

```
Update · ViewRepresentationPlugin (chained):
  ResolveViewport  → sync_resolved_map_view_frames (reads registry)
  RenderTargets    → queue/apply minimap RT resize + bind
  WorldRender      → run_minimap_compositor_pass (after SyncOverlayField)
  SyncOverlayField → after FireVisualFrameSet::BuildProfiles

PostUpdate · MapViewPlugin:
  update_minimap_view → pan/zoom/focus (presentation only)

EguiPrimaryContextPass:
  hud_root_tick → egui bridge (gated off for GPU world image)
```

| System | Phase | Notes |
|:---|:---|:---|
| `apply_minimap_camera_intent` | Update / ResolveViewport | Minimap pose only — never `MapCameraDesired` |
| Compositor pass | Update / **WorldRender** | **Not** PostUpdate |
| `update_minimap_view` | PostUpdate | Unchanged consumer |
| `minimap_shell_*` zoom/toggle | Update (tile_world_fallback) | Shell UX only |

Repo uses `ViewRepresentationSystemSet` — no new global `RenderPrepare` set for M1.

---

## 4. Bevy `ImageNode` in P3 — egui retirement

```
MinimapChromeRoot     ← P3 wire stroke (existing)
  └── MinimapGpuImageNode   ← ImageNode → MinimapRenderTargetRegistry
```

| `MinimapPresentationSource` | World pixels | egui |
|:---|:---|:---|
| `SharedCpuRaster` (default) | egui `Image` via `MapViewTextureCache` | Panel + controls |
| `SharedRenderTargetImage` | `ImageNode` | Controls / legend only |

**Env gate:** GPU compositor **on by default** (3.5); `MINIMAP_GPU_COMPOSITOR=0` → CPU fallback.

**Retirement ladder:** 3.1 bridge (landed) → 3.2 witnesses → 3.3 real WGSL → 3.5 default flip + CPU hot-path demotion. Keep `map_view/consumers/minimap.rs` as fallback adapter per spine.

---

## 5. `UiStressState` — presentation hooks (sim read-only)

Resource: `src/gui/hud/ui_stress_state.rs`

| System | Reads | Writes |
|:---|:---|:---|
| `sync_ui_stress_from_sim_system` | `SharedOverlayFieldBuffers`, `EcologyVisualSnapshot`, `RepresentationResult` | `UiStressState` only |
| `apply_minimap_stress_chrome_system` | `UiStressState`, `MinimapChromeRoot` | Border tint (chrome) |

Fire/ecology stress drives **chrome** tints, not compositor sim state. Compositor fire layer comes from `SharedOverlayFieldBuffers` + `MinimapOverlayMask.fire_heat`. Witness: `ui_stress_wrote_sim: false` in live JSON.

---

## 6. Risks

| Risk | Mitigation |
|:---|:---|
| **Viewport drift** | Extent from `ResolvedViewports.minimap_panel` → RT → Bevy node → `last_image_rect`; witness `extent_match_px ≤ 1` |
| **Dual minimap** | Mutual exclusion in `hud_root_tick`; witness `dual_minimap_present: false` |
| **VT-4** | Composite after `SyncOverlayField`; include `overlay_revision` in hash; `stage5` + `vt4_ok` |
| **VT-5** | `apply_minimap_camera_intent` → `ViewSurfaceId::Minimap` only; `minimap_shell_wrote_map_camera_desired: false` |
| **VM-08** | No new extract; refresh `infrastructure_view_isolation_live.json` |
| **Spine flicker** | Do not fold preview/window revision into minimap projection |

---

## @coder handoff — task 3.1 (numbered)

**Status:** Code landed 2026-05-24. Use §3.2 in [`ui_phase3_coder_queue_v1.md`](../prompts/guides/ui/ui_phase3_coder_queue_v1.md) for witness closure.

### 3.1.1 — Minimap render-target registry

**Files:** `src/render/minimap_compositor/render_target.rs`, `mod.rs`, `src/render/mod.rs`, `src/gui/view_representation.rs`  
**Accept:** `minimap_commit_waits_until_frame_after_resize_request` test green.

### 3.1.2 — Compositor pass

**Files:** `src/render/minimap_compositor/pass.rs`, `assets/shaders/minimap/minimap_composite.wgsl`  
**Accept:** M1 CPU bridge runs; no new extract systems. WGSL real composite → task 3.3.

### 3.1.3 — Projection / backend

**Files:** `src/gui/map_view/backend/mod.rs`, `src/gui/map_view/projection/mod.rs`  
**Accept:** `minimap_and_preview_handles_differ_when_both_allocated` test green.

### 3.1.4 — `ImageNode` + P3 chrome

**Files:** `src/gui/hud/simulation_shell_phase2.rs`, `src/gui/in_game_hud.rs`  
**Accept:** `minimap_chrome_aligned: true` in `ui_shell_migration_live.json` (GPU path).

### 3.1.5 — egui bridge gate

**Files:** `src/gui/hud/hud_root_tick.rs`, `src/gui/map_view/consumers/minimap.rs`  
**Accept:** `dual_minimap_present: false`.

### 3.1.6 — `UiStressState`

**Files:** `src/gui/hud/ui_stress_state.rs`, `simulation_shell_phase2.rs`  
**Accept:** Resource registered; `ui_stress_wrote_sim: false`.

### 3.1.7 — Live witness

**Files:** `src/render/minimap_compositor/live_proof.rs`, `full_render_diagnostic.rs`, `stage5_full_app_harness.rs`  
**Accept:** `debug_runs/minimap_compositor_live.json` with `composite_ok`, `stamp`, `extent` — **operator refresh pending**.

### 3.1 exit criteria

1. RT bound; fire toggle via overlay mask.  
2. `cargo test -p proc_A_dine01 --lib stage5 minimap_compositor` green.  
3. Isolation witness — no minimap → main camera writes.  
4. `stage5_full_app_live.json` minimap block when flag on.  
5. P3 chrome aligned on GPU path.  
6. No duplicate extract / LOD.

**Operator command:**

```powershell
$env:MINIMAP_GPU_COMPOSITOR = "1"
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## 7. Task 3.5 — **DONE** (2026-05-24)

**Witness PASS:** `debug_runs/minimap_compositor_live.json` — `composite_ok`, `GpuCompute`, `SharedRenderTargetImage`, `dual_minimap_present: false`.

**Shipped (3.5.1 + 3.5.2):**
- Default `MinimapPresentationSource::SharedRenderTargetImage` in `minimap_shell.rs`
- GPU compositor **on by default**; CPU fallback via `MINIMAP_GPU_COMPOSITOR=0` one release
- Stage5 harness `minimap_source` aligns to compositor when GPU composite active

**3.5.3** CPU hot-path demotion: **deferred** one stable release post-flip.

---

## Agent routing

| Role | Action |
|:---|:---|
| `@coder` | **3.2 witness closure** (this doc §Handoff exit) → 3.3 WGSL |
| `@sim-steward` | VT-4/VT-5 — **PASS 2026-05-24** · 3.5 **DONE** (see §7) |
| `@designer` | P3 stress tint tokens vs `design_theme.md` |
