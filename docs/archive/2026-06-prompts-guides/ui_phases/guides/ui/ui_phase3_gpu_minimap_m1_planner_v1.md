# Phase 3 planner slice — GPU minimap compositor (UX-A M1)

| Field | Value |
|:---|:---|
| **Version** | `1.1.0` |
| **Date** | 2026-05-23 |
| **Owner** | `@planner` (read-only) · implement via `@coder` task **3.1** |
| **Status** | **M1+M2 SIGNED** — [`minimap_d_m1_signoff_v1.md`](../../../docs/archive/2026-06-src-dev/plans/minimap_d_m1_signoff_v1.md) · [`minimap_d_m2_signoff_v1.md`](../../../docs/archive/2026-06-src-dev/plans/minimap_d_m2_signoff_v1.md) |
| **Coder queue** | [`ui_phase3_coder_queue_v1.md`](ui_phase3_coder_queue_v1.md) |

**Entry docs (Phase 3 mapping):**

| Referenced | Actual path |
|:---|:---|
| `ui_overhaul_plan.md` Phase 3 | [`ui_phase2_coder_queue_v1.md`](ui_phase2_coder_queue_v1.md) § Phase 3 / UX-E01 M1 |
| P3 panel mock | [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) § P3 |
| Map spine | [`tools/orchestrator/knowledge/map_view_spine.json`](../../../tools/orchestrator/knowledge/map_view_spine.json) |
| Design north star | [`docs/archive/2026-06-src-dev/plans/ux_gpu_minimap_design_v1.md`](../../../docs/archive/2026-06-src-dev/plans/ux_gpu_minimap_design_v1.md) |
| Product board | [`src/dev/post_stage6_active_todos.md`](../../../src/dev/post_stage6_active_todos.md) UX-E01 |

**Prerequisite:** Phase 2B sign-off (`ui_phase2_designer_signoff_v1.md` SIGNED) — P3 chrome contract stable (`MinimapChromeRoot` ≤2px witness green).

---

## 1. Authority — who owns minimap pixels

### Single-writer rule

| Layer | Authority | Writes | Reads |
|:---|:---|:---|:---|
| **Sim + extract** | `FireVisualFrameSet` → `SharedOverlayFieldBuffers` | Overlay fields only | Sim snapshots |
| **Policy / LOD** | `RepresentationResult` (+ `PerViewRepresentationPolicy` for minimap caps) | Policy output per frame | `WorldRepresentationFrame`, budgets |
| **GPU composite (M1 target)** | **`MinimapCompositorPass`** (new render module) | Dedicated `MinimapRenderTargetRegistry.committed_image` | See §2 — read-only |
| **Frame resolution** | `sync_resolved_map_view_frames` | `ResolvedMapViewFrames.minimap` | Registry + shell + fallback metadata |
| **Bevy shell display (M1 target)** | **`MinimapGpuImageNode`** under `MinimapChromeRoot` | `ImageNode` handle bind only | Registry handle |
| **egui (bridge → retire)** | `resolve_minimap_egui_texture` / `draw_simulation_minimap_egui` | egui texture rebind | CPU raster or RT handle via `MapViewTextureCache` |
| **Chrome layout** | `sync_minimap_chrome_root_system` (`SimulationShellPhase2Plugin`) | `MinimapChromeRoot` `Node` rect / visibility | `MinimapShellState::last_image_rect` |
| **Shell UX** | `MinimapShellState` | zoom, follow, toggles, rects — **never** terrain/fire extract | Presentation only |

### HudShellSync vs render extract

- **Render extract owns pixels** when `MinimapPresentationSource::SharedRenderTargetImage` is active. The compositor is the sole producer of the minimap `RenderTarget::Image`.
- **Hud shell sync owns layout chrome**, not raster content:
  - `sync_minimap_chrome_root_system` positions `MinimapChromeRoot` from shell rects (P3 wire frame).
  - New **`sync_minimap_gpu_image_node_system`** (coder name) binds the compositor handle to a child `ImageNode` and publishes `last_image_rect` back into `MinimapShellState` so chrome stays aligned.
- **Do not** route pixel production through `hud_root_tick` or `MapViewTextureCache` on the GPU path — those remain bridge/fallback only.

### Preview RT aliasing — **resolved in 3.1**

~~`resolve_minimap_texture_source` aliased minimap to `WorldPreviewRenderTargetRegistry`.~~ M1 uses dedicated [`MinimapRenderTargetRegistry`](../../../src/render/minimap_compositor/render_target.rs). See [`ux_gpu_minimap_m1_architecture_v1.md`](../../../docs/archive/2026-06-src-dev/plans/ux_gpu_minimap_m1_architecture_v1.md) for short reference.

---

## 2. Inputs — no duplicate LOD / extract

Compositor reads **published frames only**. No `MinimapOnlyExtract`, no second fire query, no parallel chunk residency pass.

| Input | Producer | Compositor use | Forbidden |
|:---|:---|:---|:---|
| **`SharedOverlayFieldBuffers`** | `ViewRepresentationSystemSet::SyncOverlayField` after `FireVisualFrameSet::BuildProfiles` | Fire heat sample (chunk-uniform field) | Re-extract fire entities |
| **`FireVisualFrame`** (minimap view) | `FireVisualFramesByView` + [`view_fire_projection.rs`](../../../src/render/view_fire_projection.rs) | Per-view cap / instance channel when policy allows | Tactical-only global frame |
| **`RepresentationResult`** | `WorldRepresentationResolver` | `overlay_matrix`, `extract_plan`, `gpu_budget`, minimap Hz via `VisualCadence` | Mutating policy from UI |
| **`ViewRepresentationSnapshot`** | `build_view_representation_snapshot` | World bounds, viewport contract | Direct `ViewManager` mutation |
| **`TileWorldFallbackState`** | Existing fallback raster path | Terrain/color fallback layer when GPU tiles not authoritative | New fallback writer |
| **`ResolvedViewports.minimap_panel`** | `ViewportPipelineSet::Resolve` | RT extent + DPR | egui-measured extent as authority |
| **`MinimapOverlayMask`** | `MinimapShellState` / `MapViewInstances.minimap` | Uniform toggle bits (fire on/off M1) | Shell writing overlay buffers |

**Revision coupling:** `ResolvedMapViewFrames.minimap.projection_revision` must hash **minimap registry revision + overlay revision + panel extent + compositor stamp** — not `ResolvedViewports.revision` (preview/window churn caused VT flicker; see [`projection/mod.rs`](../../../src/gui/map_view/projection/mod.rs) comments).

**M2 defer:** `LogisticsVisualSnapshot`, `EcologyVisualSnapshot`, construction GPU channel — design in [`ux_gpu_minimap_design_v1.md`](../../../docs/archive/2026-06-src-dev/plans/ux_gpu_minimap_design_v1.md) §4, out of M1 scope.

---

## 3. Schedule slot — PostUpdate vs RenderPrepare vs existing consumers

### Anchor sets (existing spine)

```
Update (ViewRepresentationPlugin, chained):
  UiCollect → ResolveViewport → CameraSync → RenderTargets → WorldRender → PostFX
  SyncOverlayField (after FireVisualFrameSet::BuildProfiles)
  SyncRepresentationMetrics (after SyncOverlayField)

PostUpdate (MapViewPlugin):
  update_minimap_view → commit_map_view_* (presentation interactions)

EguiPrimaryContextPass:
  hud_root_tick → resolve_minimap_egui_texture (bridge)
```

### M1 placement

| System | Schedule | Set / ordering |
|:---|:---|:---|
| `request_minimap_render_target_resize` | `Update` | `ViewRepresentationSystemSet::RenderTargets` · after `ViewportPipelineSet::Resolve` · reads `ResolvedViewports.minimap_panel` |
| `commit_minimap_render_target_bind` | `Update` | same set · after resize · mirror `pending_render_target_bind_ready` |
| `run_minimap_compositor_pass` | `Update` | **`ViewRepresentationSystemSet::WorldRender`** · **after** `SyncOverlayField` · **after** `run_render_projection_graph` / fire projection · **before** `sync_resolved_map_view_frames` |
| `sync_resolved_map_view_frames` | `Update` | `ResolveViewport` (unchanged slot — now reads minimap registry) |
| `sync_minimap_gpu_image_node_system` | `Update` | **after** `sync_resolved_map_view_frames` · **before** `PostUpdate` |
| `sync_minimap_chrome_root_system` | `Update` | existing `SimulationShellPhase2Plugin` order — after shell rect published |
| `update_minimap_view` | `PostUpdate` | unchanged — pan/zoom/focus presentation only |
| `apply_minimap_camera_intent` | `Update` | unchanged — never writes `MapCameraDesired` |

**Not PostUpdate for composite:** GPU work belongs in `WorldRender` (same lane as world preview composite and projection graph). PostUpdate stays presentation/interaction.

**Not a new CoreSystemSet name:** repo uses `ViewRepresentationSystemSet`; do not introduce parallel `RenderPrepare` unless planner expands global schedule — M1 stays inside `ViewRepresentationPlugin`.

**Cadence gate:** wrap compositor in `on_visual_cadence_minimap` (`VisualCadence.minimap_hz`) — same multirate policy as existing minimap throttles.

### `minimap_shell.rs` consumers (unchanged contracts)

| Consumer | Change in M1 |
|:---|:---|
| `MinimapShellState` | Add `compositor_revision`, keep `presentation_source` switch |
| `apply_minimap_camera_intent` | None |
| `minimap_shell_smooth_zoom_system` / keyboard toggle | None (render/tile_world_fallback.rs) |
| `full_render_diagnostic.rs` | Extend minimap block: `source`, `composite_revision`, `rt_bound` |
| `stage5_full_app_harness` / live proof | New minimap compositor witness fields |

---

## 4. Bevy `ImageNode` in P3 `MinimapChromeRoot` — egui texture retirement

### Target hierarchy (embedded HUD)

```
MinimapChromeRoot          ← wire stroke (existing P3)
  └── MinimapGpuImageNode  ← NEW: ImageNode bound to compositor RT
```

### Bridge behavior

| `MinimapPresentationSource` | Pixel display | egui role |
|:---|:---|:---|
| `SharedCpuRaster` (default until witness green) | egui `Image` via `MapViewTextureCache` | Full panel + controls |
| `SharedRenderTargetImage` | **`ImageNode`** under `MinimapChromeRoot` | Controls, detached window chrome, legend — **no** world `egui::Image` |

### Retirement path (3.1 → 3.2)

1. **3.1:** Spawn `MinimapGpuImageNode` at shell startup; bind when registry committed; gate egui world image with `presentation_source` check in `hud_root_tick.rs`.
2. **3.1:** Publish `MinimapShellState.last_image_rect` from Bevy node layout (not egui response) when GPU path active — keeps `sync_minimap_chrome_root_system` witness (`minimap_chrome_aligned`, ≤2px).
3. **3.2 (after witness):** Flip default to `SharedRenderTargetImage` behind env flag `MINIMAP_GPU_COMPOSITOR=1`.
4. **3.3:** Remove hot-path CPU upload (`TileWorldFallbackState.minimap_image` consumer) when FULL_APP + visual regression clean; keep CPU fallback one release.

**Do not** delete egui minimap module in M1 — [`map_view/consumers/minimap.rs`](../../../src/gui/map_view/consumers/minimap.rs) stays as fallback adapter per [`map_view_spine.json`](../../../tools/orchestrator/knowledge/map_view_spine.json) `minimap_consumer`.

---

## 5. `UiStressState` presentation hooks — sim read-only

**New resource (M1 stub, M1.1 tint):** presentation-only aggregates for HUD chrome — **not** compositor sim writers.

```rust
/// Presentation mirror — written by sync_ui_stress_from_sim_system only.
pub struct UiStressState {
    pub fire_pressure: f32,      // 0..1 from SharedOverlayFieldBuffers peak / policy cap
    pub ecology_stress: f32,     // 0..1 from EcologyVisualSnapshot macro band (optional M1: 0)
    pub revision: u64,
}
```

| Hook | Reader | Effect |
|:---|:---|:---|
| `sync_ui_stress_from_sim_system` | `SharedOverlayFieldBuffers`, `EcologyVisualSnapshot`, `RepresentationResult.overlay_matrix` | Updates `UiStressState` |
| `apply_minimap_stress_chrome_system` | `UiStressState`, `MinimapChromeRoot` | Border / subtle tint on chrome `BorderColor` — fire warm, ecology cool |
| Compositor WGSL | `MinimapOverlayMask` only M1 | Stress tints affect **chrome**, not fire heat layer (heat already in overlay buffer) |

**Rules:** Systems in `SimulationShellPhase2Plugin` or `MapViewPlugin` PostUpdate presentation lane. **No** writes to sim, `ViewManager`, or overlay buffers. If `EcologyVisualSnapshot` empty, `ecology_stress = 0`.

---

## 6. Risks

| Risk | Severity | Mitigation |
|:---|:---|:---|
| **Viewport drift** — RT extent from `ResolvedViewports.minimap_panel` ≠ Bevy `ImageNode` layout / egui rect | HIGH | Single extent authority: viewport resolve → resize queue → compositor → `last_image_rect` from Bevy node; witness `extent_match_px ≤ 1` in live JSON |
| **Dual minimap during bridge** — egui image + `ImageNode` both visible | HIGH | Mutual exclusion on `presentation_source` in `hud_root_tick` + hide egui image widget when GPU active; witness `dual_minimap_present: false` |
| **Preview RT aliasing** — reusing `WorldPreviewRenderTargetRegistry` | HIGH | Separate `MinimapRenderTargetRegistry` (§1 gap) |
| **VT-4 overlay agreement** — compositor reads stale or view-culled overlay vs full sim snapshot | HIGH | Composite after `SyncOverlayField`; include `overlay_revision` in projection hash; regression `stage5` + `vt4_ok` in readiness |
| **VT-5 spatial invariants** — minimap camera pose leaks to WorldMain | HIGH | Keep `apply_minimap_camera_intent` → `ViewSurfaceId::Minimap` only; live proof `minimap_shell_wrote_map_camera_desired: false` |
| **VM-08 / infra isolation** | MED | No new extract; per-view fire via existing projection; refresh `infrastructure_view_isolation_live.json` |
| **Map-view spine flicker** | MED | Do not fold global `resolved.revision` into minimap projection (existing comment contract) |
| **Stage 5 FULL_APP regression** | MED | `cargo test -p proc_A_dine01 --lib stage5` every slice; `--test visual` before default flip |
| **Perf shell budget** | LOW | Cadence gate 10 Hz default; track in `perf_attribution_60s.md` (<0.5 ms median M1 target) |

---

## 7. @coder handoff — task 3.1 (numbered)

Implement **M1 foundation only**. One PR slice; no M2 layers.

### 3.1.1 — Minimap render-target registry

**Files:** new `src/render/minimap_compositor/mod.rs`, `src/render/minimap_compositor/render_target.rs`; wire in `src/render/mod.rs`, `src/gui/view_representation.rs` (`RenderTargets` set).

**Do:** Mirror `WorldPreviewRenderTargetRegistry` / bind barrier pattern with minimap-specific resources. Resize from `ResolvedViewports.minimap_panel.physical_extent`.

**Accept:** Unit test — deferred bind after request frame; handle ≠ default when committed.

### 3.1.2 — WGSL compositor pass

**Files:** `assets/shaders/minimap/minimap_composite.wgsl`, `src/render/minimap_compositor/pass.rs`.

**Do:** Sample terrain fallback color field + `SharedOverlayFieldBuffers` fire heat; respect `MinimapOverlayMask.fire_heat` uniform; write minimap RT.

**Accept:** `cargo check` green; no new ECS extract systems.

### 3.1.3 — Projection / backend wiring

**Files:** `src/gui/map_view/backend/mod.rs` (`resolve_minimap_texture_source`), `src/gui/map_view/projection/mod.rs`.

**Do:** Point `SharedRenderTargetImage` at **minimap** registry, not preview registry. Update `minimap_projection` hash to include compositor revision.

**Accept:** Lib test or harness asserts preview handle ≠ minimap handle when both allocated.

### 3.1.4 — Bevy `ImageNode` + P3 chrome

**Files:** `src/gui/hud/simulation_shell_phase2.rs` (spawn `MinimapGpuImageNode`, `sync_minimap_gpu_image_node_system`), `src/gui/hud/mod.rs`.

**Do:** Child of `MinimapChromeRoot`; bind compositor image; publish `last_image_rect` for chrome sync.

**Accept:** `debug_runs/ui_shell_migration_live.json` → `minimap_chrome_aligned: true` with GPU path enabled.

### 3.1.5 — egui bridge gate

**Files:** `src/gui/hud/hud_root_tick.rs`, `src/gui/map_view/consumers/minimap.rs`.

**Do:** Skip world `egui::Image` when `SharedRenderTargetImage`; keep controls/legend path.

**Accept:** Witness `dual_minimap_present: false`.

### 3.1.6 — `UiStressState` stub

**Files:** new `src/gui/hud/ui_stress_state.rs`; register in `SimulationShellPhase2Plugin`.

**Do:** Resource + read-only sync from overlay/ecology; optional chrome tint system behind `MINIMAP_STRESS_CHROME=1`.

**Accept:** Resource present; no sim writes (grep / witness flag `ui_stress_wrote_sim: false`).

### 3.1.7 — Live witness + diagnostics

**Files:** new `src/render/minimap_compositor/live_proof.rs`; extend `src/render/full_render_diagnostic.rs`, `src/render/stage5_full_app_harness.rs`.

**Do:** Write `debug_runs/minimap_compositor_live.json` with `_agent_meta`, fields: `composite_ok`, `stamp`, `extent`, `compositor_revision`, `presentation_source`, `dual_minimap_present`, `extent_match_px`, `overlay_revision`.

**Accept:** JSON written in sim session; indexed in `debug_runs/agent_debug_index.json`.

### 3.1 exit criteria (all required)

1. GPU compositor RT bound; fire toggle works via existing overlay mask.
2. `cargo test -p proc_A_dine01 --lib stage5` green.
3. `infrastructure_view_isolation_live.json` — `minimap_shell_wrote_map_camera_desired: false`, VM-08 green.
4. `stage5_full_app_live.json` — minimap block reports RT + non-zero compositor revision (when flag on).
5. `ui_shell_migration_live.json` — P3 chrome aligned on GPU path.
6. No new extract systems; `RepresentationResult` / spine consumers unchanged in authority.

### 3.2+ (out of 3.1 scope)

- Default flip `MINIMAP_GPU_COMPOSITOR=1`
- CPU raster hot-path removal
- M2 logistics / construction layers
- Detached/native window GPU parity

---

## 8. Agent routing

| Role | Action |
|:---|:---|
| **@coder** | Execute §7 task 3.1 |
| **@designer** | Confirm P3 chrome + stress tint tokens against `design_theme.md` |
| **@sim-steward** | VT-4/VT-5 + VM-08 witness review before default flip |
| **@planner** | M2 slice after 3.1 witness lands |

**Playbook:** [`tools/orchestrator/agents/ui_layout_agent.md`](../../../tools/orchestrator/agents/ui_layout_agent.md) · **Skill:** bevy-simulation-grade (`ViewRepresentationSystemSet`, single overlay producer).
