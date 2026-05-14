# After the plan: where to go next (`base_visual_dev01` synthesis)

**North star:** **Camera intent drives visual representation selection** — implemented as **representation domain resolution**: one policy answers *what form each world datum should take right now* given gameplay, camera, importance, zones, and budgets (`base_visual_dev01_plan_status.md` § *Representation domain resolution*).

**Strict GPU / representation spine (order + DONE language):** [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md) — **theme-3 → phase-e (E1) → VT-4 → phase-d → phase-f**; policy **before** stamped snapshots and GPU upload; single fire GPU path. **Practical convergence:** [`base_visual_dev01_plan_status.md`](base_visual_dev01_plan_status.md) § *Stage staging* — **authoritative `WorldRepresentationResolver` before** particles, preview world pixels, or atmosphere incremental as primary work.

**Prerequisites:** P0-A–D and P1-E/G **✓**; Gates **1–5** policy/upload spine **~ in-tree** (`representation_policy.rs`, `render_projection_graph.rs`, `visual_snapshot_commit.rs`, `gpu_representation_metrics.rs`). **Primary expansion track:** VT integration → Phase D pixels → Phase F draw → P2-H reconcile → Stage 5 domains (IDE todos `next-09` … `next-06`).

**Full design prose:** [`base_visual_dev01.md`](base_visual_dev01.md)

**Code anchors (2026-05-10):** `src/gui/view_representation.rs` (`CameraIntent`, `CameraVisualState`, `VisualCadence`, `OverlayFieldFrame` incl. `fire_heat_overlay_revision`, `WorldFireFx` / `AtmosphereFx`, `FireVisualLod`, `OverlayChannel`, `VisualBudgetSettings`, `SwapImageBuffers`); `src/render/extraction/fire_visual_extract.rs` (`FireVisualExtractPlan`, `CLUSTERED_FIRE_INSTANCE_CAP`, LOD before extract); `MapCameraSystemSet` + `InputFrame` in `src/gui/map_camera.rs` / `src/gui/input_frame.rs`.

**How to execute the IDE todo list:** see [`base_visual_dev01_plan_status.md`](base_visual_dev01_plan_status.md) § **How to process a top-level todo** (pick card → ladder → slice → verify → update card / single IDE item).

**Engineering ladder (non-negotiable for major work):** each roadmap item is executed as **PHASE → CONTRACTS → DATA OWNERSHIP → SCHEDULE → TEST SURFACE → DEBUG VISUALIZATION → KNOWN FAILURE MODES → OPTIMIZATION** — *not* “feature then implementation”. **Tracker shape:** **one top-level todo per phase/theme**; put `[D-1]` style work as **SUBTASKS inside that card** (not as separate top-level todos) — see [`base_visual_dev01_plan_status.md`](base_visual_dev01_plan_status.md) § *Major engineering ladder* (includes universal template, **BLOCKED BY**, recommended IDE list, `visual-test-matrix-upgrade`).

---

## 1. What you are building (two layers, one game)

1. **Strategic world layer** — gameplay truth: terrain, logistics, fire/smoke tied to chunks, units, overlays. Must scale with zoom, stay readable, stay ECS-authoritative.
2. **Atmospheric macro layer** — “outside the frame”: plumes, haze, distant glow, cinematic embers. **Emotional + strategic read**, not authoritative sim. **Camera-reactive** (zoom out = epic; zoom in = tactical).

The parent doc’s point: this split is **valuable** — formalize it in types (`WorldFireFx` vs `AtmosphereFx`), update rates, and ownership so systems stop fighting the same frame budget.

---

## 2. Next real milestone (Stage 5 unlock)

**Primary track (2026-05-13):** app-level **VT-4/VT-5** (CI matrix + readiness) → **Phase D** GPU preview authority → **Phase F** LOD proof + instanced draw → **P2-H** hybrid reconcile ✓ → **Stage 5** logistics/ecology projection on the unified registry graph.

**Done in-tree (Gates 1–5 + stage5-08):** `RepresentationResult` policy spine; projection-owned bursts; `CommittedVisualSnapshotFence`; VT harness + CI test step; GPU metrics HUD; zone policy boosts; GPU preview authority scaffold + CPU raster gate; particle **upload** via registry; **Phase F LOD proof** resource; atmosphere authoritative partial GPU field; logistics/ecology snapshot publish; `AppStage5ReadinessReport` + headless profile.

**Still open:** GPU preview layer parity (height/moisture/ecology overlays on GPU); particle draw pass; domain projection nodes at strict DONE; idle CPU guard + visible-chunk cache.

---

## 3. Sequencing after P0 is stable

| Phase | Goal | Outcome |
|-------|------|---------|
| **A** | P0 raster + egui handle + preview throttle + HUD dirty | Even frametimes; egui stops masquerading as a 60 Hz renderer |
| **B** | P1 shared fire proxy + overlay buffers | One extract; **sim** minimap + world preview + lights agree on chunk heat (`SharedOverlayFieldBuffers`); editor road minimap still terrain-only |
| **C** | P1 camera intent + smoothing | No jitter; optional `run_if` on apply |
| **D** | Preview architecture pivot | Prefer **Bevy render target → egui image** over huge CPU RGBA long-term |
| **E** | P2 atmosphere incremental + FixedUpdate sim | Determinism and scale; **high coupling** — schedule design doc first |
| **F** | P2 GPU particles | Registry upload slice ✓; **instanced draw after resolver + VT-4**; Hanabi only if art requires |

---

## 4. Camera “north star”

- **Strategic:** edge pan (optional), minimap, overlays, smooth zoom — default RTS.
- **Tactical:** tighter zoom, emphasize **world** particles and local smoke/audio.
- **Cinematic / macro:** emphasize **atmospheric** layer; can reduce UI chrome for shots/replay later.
- **Follow / free:** explicit ownership so the player always knows what owns the view (parent §4 “three camera modes” A/B/C).

Cameras must **never** block on: worldgen completion, minimap CPU raster, fire extract, or full egui tree rebuild.

---

## 5. Technical risks called out in the parent (do not forget)

- **Incremental atmosphere:** diffusion/advection boundaries — use hybrid “local every frame + full every N sec” if full incremental is too risky.
- **FixedUpdate sim:** extraction and **interpolation** must be defined for render; UI expectations shift.
- **egui scope:** tooling + panels yes; **full world** raster should shrink over time.

---

## 6. Success criteria (when to declare “visual dev 01 wave” done)

- [x] **Representation policy spine (Gates 1–5)** — `RepresentationResult` consumed by projection/compute/HUD; burst hints projection-only; snapshot fence before projection (`plan_status` § *Stage staging*).
- [ ] **Representation policy authoritative in app** — no per-subsystem LOD in runtime paths; CI VT green.
- [ ] No full-map CPU rebuild on idle frames in sim + editor preview paths you care about.
- [x] Fire/smoke/light/**CPU minimap + preview** read from one agreed path: **`FireVisualFrame`** → `SharedOverlayFieldBuffers` (chunk heat **derived from frame only**); lights cluster from `FireVisualFrame::instances`; smoke aggregate `FireAtmosphereAggregate` for UI label.
- [x] Camera recenter / frame-world / edge toggle exist and are discoverable (HUD MAP line + keybindings options).
- [ ] Macro FX visibly respond to zoom without breaking world-anchored tactical fire.

---

## 7. Doc maintenance

When a **P0/P1** row in the plan file flips to `✓`, add one line under that item in **this** file’s “Changelog” (optional subsection you append):

```text
2026-05-10 — P0-A–D: dirty raster revision + stable egui texture ids + preview partial throttle (~12 Hz) + HUD ops strip / build / narrative string caches (`tile_world_fallback.rs`, `world_preview/*`, `in_game_hud.rs`).
2026-05-10 — P1-E~ / P1-F ✓ / P1-G~: `FireVisualProxy` type alias; `MapCameraDesired` + smoothing + keybinds + `SharedOverlayFieldBuffers` stub plugin (`map_camera.rs`, `input_bindings.rs`, `overlay_field_buffers.rs`, engine plugin list).
2026-05-10 — P1-E ✓ / P1-G ✓ / minimap: `FireVisualFrame` + `chunk_heat`; overlay **only** from frame; world preview + sim tile fallback; `TileWorldFallbackAfterFireExtract` after `FireVisualFrameSet::BuildProfiles`; MAP ops strip + `MapCameraMode` cycle (M).
2026-05-10 — Latency: map editor stable egui texture + `MapEditorMinimapRasterDirty` epoch raster; world preview overlay-only throttle; `InputFrame` (`input_frame.rs`) for camera delta read surface.
2026-05-10 — Fire GPU path: `ExtractResourcePlugin<FireVisualFrame>`; `FireVisualGpuInstanceStorage` + `weather_fire_field.wgsl` `@group(1)`; uniforms `fire_instance_count`; smoke-only `publish_sim_visual_extract`; `SimFireEmitterVisualExtract` not extracted to render.
2026-05-10 — **Canonical contract:** `FireVisualFrame` = sole CPU fire snapshot (`instances` + `chunk_heat`); `SharedOverlayFieldBuffers` derived from frame only; `FireVisualFrameSet` / `FireVisualFramePlugin`.
2026-05-10 — View Representation scaffolding: `view_representation.rs`, `MapCameraSystemSet`, `InputFrame` scroll/drag/frame counter, `FxVisibilitySettings` synced from map camera mode + zoom.
2026-05-10 — View Rep wiring: GPU `WeatherFireFieldUniforms` + `AtmosphereRenderLayers` use `CameraVisualState` / `FxVisibilitySettings`; world preview partial interval from `VisualBudgetSettings::preview_hz`; HUD MAP line shows CAM + weights.
2026-05-10 — Phase D scaffold: `world_preview/preview_render_contract.rs` (preview ownership + `PreviewRenderTarget` + `PreviewRenderBudget` + debug stub); sync before texture resize/raster; throttle uses `PreviewRenderBudget` + `preview_partial_min_interval_from_hz`.
2026-05-13 — Gates 1–5 + post-gate scaffolds: `representation_policy.rs`, `CommittedVisualSnapshotFence`, `visual_agreement.rs`, `gpu_representation_metrics.rs`, zone policy, Phase D contract/`gpu_preview.rs`, particle registry upload, **P2-H** dirty queue + GPU partial uploads + partial compute dispatch, `visual_domain_snapshots.rs`; `cargo test --lib` green.
2026-05-13 — **stage5-08:** `stage5_readiness.rs` + headless default / full-app when preview loads; MAP REP + F3 readiness; `phase_f_lod_proof.rs`; Phase D `PreviewPathAuthority`; CI `cargo test --lib`.
```

Keeps the long `base_visual_dev01.md` stable while status lives in the split files.

---

## 8. Root cause map (latency)

| Area | What was wrong | Mitigation in repo | Still open |
|------|----------------|-------------------|------------|
| **A. egui + texture churn** | Map editor minimap registered a new egui texture every frame (`add_image` per pass). | `MapEditorMapTexture::egui_texture_cache` — reuse `TextureId` until the `Handle<Image>` changes; cache cleared in `map_editor_sync_map_texture_size` when the image is recreated. | Scan other egui panels for the same pattern. |
| **B. World preview dirty key** | Fire overlay revision could trigger a **full** CPU pass every `Update` tick. | Treat overlay-only invalidation like partials: same ~12 Hz gate as chunk dirty (`world_preview/render_raster.rs`). World raster still driven by material epoch + dirty queue + layers + tex size (not egui viewport zoom alone). | Visible-chunk slice cache (P0-2) not built yet. |
| **C. Input coalescing** | Event backlog risk on camera paths. | Bevy’s `AccumulatedMouseMotion` is already per-frame summed; `InputFrame` (`gui/input_frame.rs`) snapshots it in `PreUpdate`; `map_camera` grip pan reads `InputFrame::pointer_delta` and scroll blend. | Optional: merge other pointer sources into `InputFrame`. |
| **D. Fire visual consistency** | Many consumers re-query sim fire. | Single path: **`FireVisualFrame`** (ECS once) → overlay from `chunk_heat` only → cluster/lights from `instances`; `SharedOverlayFieldBuffers` is a **view**, not a second sim scan. | **Resolver authority** + particle draw after VT-4; fold `fx_burst_request` side path |
