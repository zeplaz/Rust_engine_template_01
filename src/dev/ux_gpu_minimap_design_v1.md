# UX-A — GPU minimap compositor design `v1`

**Status:** **M1+M2 SIGNED** — [`minimap_d_m1_signoff_v1.md`](minimap_d_m1_signoff_v1.md) · [`minimap_d_m2_signoff_v1.md`](minimap_d_m2_signoff_v1.md) (**D-MINIMAP-M2** 2026-05-24)  
**Parent brief:** [`prompts/guides/experience_layer_ux_hud_designer_brief_v1.md`](../../prompts/guides/experience_layer_ux_hud_designer_brief_v1.md) §2  
**Post–Stage 6 board:** [`post_stage6_active_todos.md`](post_stage6_active_todos.md) · **UX-E01**  
**Active M1 coder handoff (numbered tasks + witnesses):** [`prompts/guides/ui/ui_phase3_gpu_minimap_m1_planner_v1.md`](../../prompts/guides/ui/ui_phase3_gpu_minimap_m1_planner_v1.md) §7

---

## 1. Problem

The simulation minimap today is an **egui-hosted CPU texture** (`resolve_minimap_egui_texture` → `draw_simulation_minimap_egui` in [`src/gui/hud/hud_root_tick.rs`](../gui/hud/hud_root_tick.rs)). That path is acceptable for tooling and early Stage 5, but it is the wrong long-term owner for:

- pan/zoom at tactical refresh rates  
- multi-layer overlays (fire, logistics, construction, EW)  
- detachable / fullscreen presentation modes  
- GPU compositing beside the main map (no duplicate world extract)

**Rule:** egui hosts **chrome** (dock, toggles, drag handles); it does **not** own per-frame world rasterization.

---

## 2. North star

```text
WorldRepresentationFrame + domain snapshots
        ↓
SharedOverlayFieldBuffers + RenderProjectionGraph (fire / logistics / ecology)
        ↓
MinimapCompositorPass (WGSL, single RenderTarget::Image)
        ↓
egui::Image / dock shell (display only)
```

Same spine as World Main and World Preview — **no** `MinimapOnlyExtract` ECS queries.

---

## 3. Presentation modes (UX-A)

| Mode | `MinimapPresentationMode` | Host |
|------|---------------------------|------|
| Embedded HUD | `Embedded` | [`HudDockRegistry`](../gui/hud/shell_framework.rs) slot `Minimap` |
| Detached window | `Detached` | Native / egui floating (`MinimapShellState::detached`) |
| Fullscreen strategic | `Fullscreen` | Shell transition; same GPU target, different layout contract |

**Source enum (existing):** [`MinimapPresentationSource`](../gui/minimap_shell.rs) — today `SharedCpuRaster` default; target **`SharedRenderTargetImage`** (BQ-124).

---

## 4. Compositor inputs (layer stack)

| Layer | Authority | Today | M1 target |
|-------|-----------|-------|-----------|
| Terrain / tiles | `TileWorldFallbackState` + chunk residency | CPU tint into shared raster | Sample residency-colored low-res field |
| Fire heat | `SharedOverlayFieldBuffers` + `FireVisualFrame` | CPU path | Overlay channel from projection graph |
| Logistics stress | `LogisticsVisualSnapshot` → projection `log_rows` | Policy gate `overlay_matrix.logistics` | Heat strip per corridor row |
| Ecology | `EcologyVisualSnapshot` | Macro band | Optional tint M2 |
| Construction | `ConstructionPhaseGpuChannel` | Phase tiles on instanced path | Corridor/site glyphs M2 |
| Units / markers | Stage 7+ | — | M3 |

**Per-view caps:** reuse [`PerViewRepresentationPolicy`](../render/view_runtime/per_view_policy.rs) — minimap fire cap already lower than WorldMain (VM-11).

---

## 5. Authority & scheduling (must not break)

| Concern | Owner |
|---------|--------|
| Viewport rect / DPR | [`ViewportPipelineSet::Resolve`](../render/viewport_pipeline.rs) → minimap panel id |
| Camera pose (follow main) | [`MinimapShellState`](../gui/minimap_shell.rs) + [`MapViewInstances::minimap`](../gui/map_view/view_state.rs) |
| Overlay mask VM-08 | [`ViewManager`](../gui/view_authority.rs) ↔ `MapViewInstances` (witness: `infrastructure_view_isolation_live.json`) |
| Projection order | After `FireVisualFrameSet::ProjectGpu`; compositor in `ViewRepresentationSystemSet::WorldRender` |
| Input routing | [`MapViewInteractionByView`](../gui/map_view/presentation/mod.rs) — preview/minimap never write `MapCameraDesired` for WorldMain |

**Deferred (DQ-POST-04):** construction ghosts stay on `SimulationMapViewport` until VM-09 + Wave P stable; minimap compositor **reads** construction overlay only.

---

## 6. Resource contract (implementation sketch)

```rust
/// Shell UX — not render owner (see brief §2.3).
#[derive(Resource)]
pub struct MinimapCompositorState {
    pub mode: MinimapPresentationMode,
    pub source: MinimapPresentationSource,
    pub render_target: Option<Handle<Image>>,
    pub revision: u64,
    pub last_composite_stamp: SimStepStamp,
}

/// Extends today’s MinimapShellState — presentation flags only.
// zoom, follow_mode, overlays: MinimapOverlayMask → extend with logistics bit when VIS-05 lands
```

**New render module (target path):** `src/render/minimap_compositor.rs` + `assets/shaders/minimap/minimap_composite.wgsl`

**Plugin hook:** register in [`ViewRepresentationPlugin`](../gui/view_representation.rs) after `RenderProjectionGraph` evaluate.

---

## 7. Phased delivery

### M1 — Foundation (first code slice) — **SIGNED** [`minimap_d_m1_signoff_v1.md`](minimap_d_m1_signoff_v1.md)

**Execute:** [`ui_phase3_minimap_compositor_plan.md`](ui_phase3_minimap_compositor_plan.md) · [`ux_gpu_minimap_m1_architecture_v1.md`](ux_gpu_minimap_m1_architecture_v1.md) · **Coder queue:** [`ui_phase3_coder_queue_v1.md`](../../prompts/guides/ui/ui_phase3_coder_queue_v1.md). **Code:** landed 2026-05-24 — lib tests `stage5` + `minimap_compositor` green; live JSON refresh via `MINIMAP_GPU_COMPOSITOR=1 cargo run -p proc_A_dine01 --release -- --test visual`.

- [x] `RenderTarget::Image` per minimap panel extent — [`MinimapRenderTargetRegistry`](../render/minimap_compositor/render_target.rs)
- [x] Compositor pass (M1 CPU bridge + WGSL stub) — [`minimap_compositor/pass.rs`](../render/minimap_compositor/pass.rs), [`minimap_composite.wgsl`](../../assets/shaders/minimap/minimap_composite.wgsl)
- [x] egui bridge gate + `MinimapGpuImageNode` when `SharedRenderTargetImage` — env `MINIMAP_GPU_COMPOSITOR=1`
- [x] Witness: `debug_runs/minimap_compositor_live.json` (`composite_ok`, `GpuCompute`, `SharedRenderTargetImage`) — **2026-05-24** (`MINIMAP_GPU_COMPOSITOR=1 --test capture`)

### M2 — Strategic shell overlays — **SIGNED** [`minimap_d_m2_signoff_v1.md`](minimap_d_m2_signoff_v1.md)

- [x] Logistics heat from `LogisticsVisualSnapshot` (same rows as FULL_APP `log_rows`)
- [x] Construction phase channel (`CorridorConstructionBook` + `construction_heat` compositor binding)
- [x] Ecology macro band (`EcologyVisualSnapshot` + `ecology_heat` compositor binding)
- [ ] Overlay tray bits → compositor uniforms (tray → policy bridge) — **UI-P3-M2-TRAY-OPT** optional

### M3 — Operational shell

- [ ] Fog-of-war, EW, unit aggregation markers (Stage 7 brief alignment — **out of UI-P3-M3-001**)
- [ ] Replay / intel scrub markers

---

## 8. Migration off egui raster

| Step | Action | Status |
|------|--------|--------|
| 1 | Feature flag / `MinimapPresentationSource` switch | ✅ default GPU (env unset → on) |
| 2 | Run FULL_APP + visual test with GPU source | ✅ 2026-05-24 |
| 3 | Remove duplicate CPU upload in hot path | ☐ after M2 stable |
| 4 | Keep CPU path one release as fallback | ✅ `MINIMAP_GPU_COMPOSITOR=0` opt-out |

---

## 9. Acceptance criteria (M1 done)

1. Minimap panel shows **GPU** texture in sim with fire overlay toggle.  
2. No new ECS extract systems; `cargo test -p proc_A_dine01 --lib` green.  
3. `infrastructure_view_isolation_green` stays true (no minimap → main camera writes).  
4. `stage5_full_app_live.json` reports minimap RT bound + non-zero composite revision.  
5. Perf: minimap composite &lt; 0.5 ms median on reference machine (track in `perf_attribution_60s.md`).

---

## 10. Dependencies & blockers

| Dependency | Why |
|------------|-----|
| Wave P green | Preview RT + viewport contract patterns |
| VM-08 / VM-11 | Per-view overlay masks and fire caps |
| LOG-E01 | Logistics rows in projection — feeds M2 heat layer |
| BQ-124 | `SharedRenderTargetImage` consumer path documented in minimap shell |

**Not blocking M1:** industrial activation, construction Round 3.

---

## 11. Agent routing

| Role | First read |
|------|------------|
| **designer** | This doc + experience brief §2 |
| **coder** | [`ui_phase3_gpu_minimap_m1_planner_v1.md`](../../prompts/guides/ui/ui_phase3_gpu_minimap_m1_planner_v1.md) §7 · then `minimap.rs`, `viewport_pipeline.rs`, `gpu_tile_debug_draw` |
| **sim-steward** | VM-08 witness + `ViewManager` overlay sync |

---

## 12. Explicit non-goals (M1)

- Campaign transmission widget (UX-B)  
- egui-owned strategic map editor  
- Second logistics sim or solver  
- Reopening Stage 5/6 operational sign-offs  
