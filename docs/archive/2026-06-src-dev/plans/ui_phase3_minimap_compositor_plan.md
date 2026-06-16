# UX-A M1 — GPU minimap compositor plan (authority + @coder 3.1)

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@planner` (read-only) |
| **Status** | **3.1 landed** — reference contract for M2+ edits |
| **Lane index** | [`ui_overhaul_plan.md`](ui_overhaul_plan.md) · [`ux_gpu_minimap_m1_architecture_v1.md`](ux_gpu_minimap_m1_architecture_v1.md) |
| **Active queue** | [`ui_phase3_coder_queue_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_coder_queue_v1.md) §3.4 M2 |

**Entry:** P3 [`ui_phase0_panel_mocks_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase0_panel_mocks_v1.md) · spine [`map_view_spine.json`](../tools/orchestrator/knowledge/map_view_spine.json) · design [`ux_gpu_minimap_design_v1.md`](ux_gpu_minimap_design_v1.md)

---

## 1. Authority map

Single-writer rule — no dual pixel paths on the GPU lane.

```text
Simulation extract                Policy (read-only)              Compositor              Display
─────────────────                ──────────────────              ──────────              ───────
FireVisualFrameSet               RepresentationResult            MinimapCompositorPlugin  MinimapGpuImageNode
       │                         VisualCadence / overlay_matrix         │                        │
       ▼                                  │ read                       ▼                        ▼
SharedOverlayFieldBuffers ───────────────┼──────────► run_minimap_compositor_pass ──► MinimapRenderTargetRegistry
       ▲                                  │              │ gpu_compute dispatch              │
       │                         PerViewRepresentationPolicy          ▼                        │
LogisticsVisualSnapshot (M2)             │         committed_image ◄── ImageNode bind          │
TileWorldFallbackState (terrain)         │                                   sync_minimap_gpu_image_node_system
                                         │                                   sync_minimap_chrome_root_system (wire only)
MinimapShellState (UX toggles) ──────────┘                                   resolve_minimap_egui_texture (fallback only)
ResolvedViewports.minimap_panel (extent)
```

| Layer | Authority | May write | Must not |
|:---|:---|:---|:---|
| **Fire / overlay sim** | `FireVisualFrameSet` → `SharedOverlayFieldBuffers` | Chunk fire heat map | — |
| **Policy** | `RepresentationResult` (+ resolver inputs) | Band, overlay matrix, budgets | Compositor RT |
| **Per-view fire cap** | `FireVisualFramesByView` + `view_fire_projection.rs` | Minimap tactical frame | Second minimap extract |
| **Compositor pixels** | `run_minimap_compositor_pass` + `gpu_compute.rs` | `MinimapRenderTargetRegistry.committed_image` | Sim state, `ViewManager` |
| **RT lifecycle** | `queue_minimap_render_target_resize` / bind barrier | Deferred GPU image alloc | egui layout |
| **Frame resolution** | `sync_resolved_map_view_frames` | `ResolvedMapViewFrames.minimap` | Raster content |
| **Bevy world image** | `MinimapGpuImageNode` | `ImageNode` handle bind | Composite shaders |
| **P3 chrome** | `sync_minimap_chrome_root_system` | `MinimapChromeRoot` rect / stroke | Pixels |
| **Shell UX** | `MinimapShellState` | zoom, follow, toggles, rects | Extract / overlay buffers |
| **egui fallback** | `resolve_minimap_egui_texture` | egui texture id (CPU path only) | GPU path world image |

**HudShellSync vs render extract:** shell systems publish **geometry** (`last_image_rect`, chrome pad). **Render extract owns pixels** when `MinimapPresentationSource::SharedRenderTargetImage`.

**Separate from preview:** minimap RT uses `MinimapRenderTargetRegistry` — not `WorldPreviewRenderTargetRegistry`.

---

## 2. Inputs — `SharedOverlayFieldBuffers`, `FireVisualFrame`, `RepresentationResult`

### Direct compositor reads (M1)

| Input | Resource / type | Read in | Role |
|:---|:---|:---|:---|
| **SharedOverlayFieldBuffers** | `overlay_field_buffers.rs` | `composite.rs` → `upload_minimap_heat_textures` | `chunk_fire_heat` → fire storage texture |
| **TileWorldFallbackState** | fallback raster | `pass.rs` | Terrain storage sync source |
| **LogisticsVisualSnapshot** | M2+ | `composite.rs` | Corridor heat (optional layer) |
| **MinimapOverlayMask** | `MapViewInstances.minimap.overlays` | `pass.rs` → GPU uniforms | `fire_heat`, `logistics_heat` toggles |
| **ResolvedViewports.minimap_panel** | viewport pipeline | `pass.rs`, `render_target.rs` | RT extent |
| **VisualCadence.minimap_hz** | from budget policy | `pass.rs` + `on_visual_cadence_minimap` | Multirate throttle |

### Indirect inputs (no duplicate extract)

| Input | Upstream producer | How compositor consumes | Compositor must not |
|:---|:---|:---|:---|
| **FireVisualFrame** | `FireVisualFrameSet::BuildProfiles` | **Not read directly.** Heat copied into `SharedOverlayFieldBuffers` in `SyncOverlayField` lane before compositor runs | Query fire entities; build minimap-only frame |
| **FireVisualFrame (minimap view)** | `view_fire_projection.rs` / `FireVisualFramesByView` | Caps/tactical slice **already folded** into shared overlay publish path | Per-view second extract in compositor |
| **RepresentationResult** | `WorldRepresentationResolver` | **Not read directly in compositor.** Drives `overlay_matrix`, `VisualCadence`, extract plan upstream; compositor inherits cadence + published buffers only | Write policy; branch on tactical cull inside pass |

**Invariant:** One fire overlay producer → one buffer → minimap + world preview + fallback raster consumers.

**Revision coupling:** `ResolvedMapViewFrames.minimap.projection_revision` hashes registry + overlay revision + panel extent + compositor stamp — **exclude** global `ResolvedViewports.revision` (flicker guard; see `map_view/projection/mod.rs`).

### Forbidden (hard)

- `MinimapOnlyExtract` or any minimap-scoped ECS fire query
- Shell / egui writing `SharedOverlayFieldBuffers`
- Aliasing minimap RT to world-preview registry
- Compositor reading `ViewManager` or mutating `MapCameraDesired`

---

## 3. Schedule slot

Plugin: `MinimapCompositorPlugin` — registered from `ViewRepresentationPlugin`.

```text
Update · ViewRepresentationSystemSet (chained spine):

  ViewportPipelineSet::Resolve
       │
       ├─ RenderTargets ─ queue_minimap_render_target_resize
       │                  apply_minimap_gpu_resize_request
       │                  commit_minimap_render_target_bind_system
       │
       ├─ FireVisualFrameSet::BuildProfiles
       │       │
       │       ▼
       ├─ SyncOverlayField  (SharedOverlayFieldBuffers fresh)
       │       │
       │       ▼
       └─ WorldRender ─ sync_minimap_presentation_source
                        run_minimap_compositor_pass  [run_if on_visual_cadence_minimap]
                               │
                               ▼ (ExtractResource → render world)
                        gpu_compute composite dispatch

  ResolveViewport ─ sync_resolved_map_view_frames  (after RT; reads registry)

PostUpdate · SimulationShellPhase2Plugin:
  sync_minimap_gpu_image_node_system  (after hud egui pass)
  sync_minimap_chrome_root_system

PostUpdate · MinimapCompositorPlugin:
  write_minimap_compositor_live_proof_system  (Simulation only)

EguiPrimaryContextPass:
  hud_root_tick → resolve_minimap_egui_texture  (gated when SharedRenderTargetImage)
```

| System | Phase | Set | Notes |
|:---|:---|:---|:---|
| Compositor pass | **Update** | `WorldRender` | **Not** PostUpdate |
| `update_minimap_view` | PostUpdate | MapViewPlugin | Presentation pan/zoom only |
| `apply_minimap_camera_intent` | Update | `ResolveViewport` | Minimap pose — never WorldMain |
| Live proof | PostUpdate | Simulation | `debug_runs/minimap_compositor_live.json` |

**Env gate:** `minimap_gpu_compositor_env_enabled()` — unset env → GPU **on**; `MINIMAP_GPU_COMPOSITOR=0` → CPU fallback.

---

## 4. @coder 3.1 — file list + acceptance

Implement / maintain M1 foundation. Tasks map to landed modules (2026-05-24).

### 3.1.1 — Minimap render-target registry

| File | Role |
|:---|:---|
| `src/render/minimap_compositor/render_target.rs` | `MinimapRenderTargetRegistry`, bind barrier, deferred commit |
| `src/render/minimap_compositor/mod.rs` | Plugin registration, RT systems in `RenderTargets` |
| `src/render/mod.rs` | Re-exports |
| `src/gui/view_representation.rs` | `MinimapCompositorPlugin` hook |

**Accept:** `minimap_commit_waits_until_frame_after_resize_request` test green.

### 3.1.2 — Compositor pass + GPU dispatch

| File | Role |
|:---|:---|
| `src/render/minimap_compositor/pass.rs` | Cadence, dispatch queue, `MinimapCompositorState` |
| `src/render/minimap_compositor/composite.rs` | Heat upload from `SharedOverlayFieldBuffers` |
| `src/render/minimap_compositor/gpu_compute.rs` | Render-graph compute dispatch |
| `assets/shaders/minimap/minimap_composite.wgsl` | WGSL composite kernel |

**Accept:** `composite_path: GpuCompute` in live JSON; no new ECS extract.

### 3.1.3 — Projection / map-view backend

| File | Role |
|:---|:---|
| `src/gui/map_view/backend/mod.rs` | `resolve_minimap_texture_source` → minimap registry |
| `src/gui/map_view/projection/mod.rs` | `sync_resolved_map_view_frames` revision hash |
| `src/gui/map_view/presentation/stability.rs` | Minimap registry consumer |

**Accept:** `minimap_and_preview_handles_differ_when_both_allocated` test green.

### 3.1.4 — Bevy `ImageNode` + P3 chrome

| File | Role |
|:---|:---|
| `src/gui/hud/simulation_shell_phase2.rs` | `MinimapGpuImageNode`, `sync_minimap_gpu_image_node_system`, chrome sync |
| `src/gui/in_game_hud.rs` | Spawn hierarchy under `MinimapChromeRoot` |
| `src/gui/hud/mod.rs` | Public types |

**Accept:** `ui_shell_migration_live.json` → `minimap_chrome_aligned: true`.

### 3.1.5 — egui bridge gate

| File | Role |
|:---|:---|
| `src/gui/hud/hud_root_tick.rs` | Skip world egui image on GPU path |
| `src/gui/map_view/consumers/minimap.rs` | Fallback texture resolver |
| `src/gui/minimap_shell.rs` | `MinimapPresentationSource`, overlay mask |

**Accept:** `dual_minimap_present: false`.

### 3.1.6 — `UiStressState` (chrome only)

| File | Role |
|:---|:---|
| `src/gui/hud/ui_stress_state.rs` | Read-only sim mirror |
| `src/gui/hud/simulation_shell_phase2.rs` | Register + chrome tint systems |

**Accept:** `ui_stress_wrote_sim: false` in compositor witness.

### 3.1.7 — Live witness + diagnostics

| File | Role |
|:---|:---|
| `src/render/minimap_compositor/live_proof.rs` | `minimap_compositor_live.json` writer |
| `src/render/full_render_diagnostic.rs` | Minimap diagnostic block |
| `src/render/stage5_full_app_harness.rs` | FULL_APP minimap fields + proof sidecar |

**Accept:** `debug_runs/minimap_compositor_live.json` → `composite_ok`, `stamp`, `extent`, `rt_bound`.

### 3.1 exit criteria (regression)

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor stage5
cargo run -p proc_A_dine01 --release -- --test visual
```

| Witness | Required |
|:---|:---|
| `minimap_compositor_live.json` | `composite_ok: true`, `dual_minimap_present: false` |
| `infrastructure_view_isolation_live.json` | `minimap_shell_wrote_map_camera_desired: false` |
| `stage5_full_app_live.json` | minimap RT + compositor revision |
| `ui_shell_migration_live.json` | P3 chrome aligned |

---

## 5. Risks (M1)

| Risk | Guard |
|:---|:---|
| Viewport drift | Extent from `ResolvedViewports.minimap_panel` → RT → Bevy node; witness `extent_match_px` |
| Dual minimap | egui world image gated; witness `dual_minimap_present` |
| VT-4 overlay agreement | Composite after `SyncOverlayField`; hash includes `overlay_revision` |
| VT-5 pose leak | `apply_minimap_camera_intent` → `ViewSurfaceId::Minimap` only |
| Preview RT aliasing | Separate `MinimapRenderTargetRegistry` |

---

## 6. Agent routing

| Role | When |
|:---|:---|
| **@coder** | M2+ overlay edits — start §3.1 file list, touch ≤3 files per slice |
| **@sim-steward** | VT-4/VT-5 regression before default-path changes |
| **@planner** | M3 operational overlays |

**Next slice (post-M1):** [`ui_phase3_coder_queue_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_coder_queue_v1.md) **UI-P3-M2-001** — logistics heat (`logistics_rows > 0`).
