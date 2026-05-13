# After the plan: where to go next (`base_visual_dev01` synthesis)

**Prerequisites:** Work through [`base_visual_dev01_plan_status.md`](base_visual_dev01_plan_status.md) at least through **P0-A–D** before investing heavily in GPU particles or cinematic polish.

**Full design prose:** [`base_visual_dev01.md`](base_visual_dev01.md)

---

## 1. What you are building (two layers, one game)

1. **Strategic world layer** — gameplay truth: terrain, logistics, fire/smoke tied to chunks, units, overlays. Must scale with zoom, stay readable, stay ECS-authoritative.
2. **Atmospheric macro layer** — “outside the frame”: plumes, haze, distant glow, cinematic embers. **Emotional + strategic read**, not authoritative sim. **Camera-reactive** (zoom out = epic; zoom in = tactical).

The parent doc’s point: this split is **valuable** — formalize it in types (`WorldFireFx` vs `AtmosphereFx`), update rates, and ownership so systems stop fighting the same frame budget.

---

## 2. Sequencing after P0 is stable

| Phase | Goal | Outcome |
|-------|------|---------|
| **A** | P0 raster + egui handle + preview throttle + HUD dirty | Even frametimes; egui stops masquerading as a 60 Hz renderer |
| **B** | P1 shared fire proxy + overlay buffers | One extract; **sim** minimap + world preview + lights agree on chunk heat (`SharedOverlayFieldBuffers`); editor road minimap still terrain-only |
| **C** | P1 camera intent + smoothing | No jitter; optional `run_if` on apply |
| **D** | Preview architecture pivot | Prefer **Bevy render target → egui image** over huge CPU RGBA long-term |
| **E** | P2 atmosphere incremental + FixedUpdate sim | Determinism and scale; **high coupling** — schedule design doc first |
| **F** | P2 GPU particles | When counts force it; instancing before Hanabi unless art pipeline wants Hanabi |

---

## 3. Camera “north star”

- **Strategic:** edge pan (optional), minimap, overlays, smooth zoom — default RTS.
- **Tactical:** tighter zoom, emphasize **world** particles and local smoke/audio.
- **Cinematic / macro:** emphasize **atmospheric** layer; can reduce UI chrome for shots/replay later.
- **Follow / free:** explicit ownership so the player always knows what owns the view (parent §4 “three camera modes” A/B/C).

Cameras must **never** block on: worldgen completion, minimap CPU raster, fire extract, or full egui tree rebuild.

---

## 4. Technical risks called out in the parent (do not forget)

- **Incremental atmosphere:** diffusion/advection boundaries — use hybrid “local every frame + full every N sec” if full incremental is too risky.
- **FixedUpdate sim:** extraction and interpolation must be defined for render; UI expectations shift.
- **egui scope:** tooling + panels yes; **full world** raster should shrink over time.

---

## 5. Success criteria (when to declare “visual dev 01 wave” done)

- [ ] No full-map CPU rebuild on idle frames in sim + editor preview paths you care about.
- [x] Fire/smoke/light/**CPU minimap + preview** read from one agreed path: **`FireVisualFrame`** → `SharedOverlayFieldBuffers` (chunk heat **derived from frame only**); lights cluster from `FireVisualFrame::instances`; smoke aggregate `FireAtmosphereAggregate` for UI label.
- [x] Camera recenter / frame-world / edge toggle exist and are discoverable (HUD MAP line + keybindings options).
- [ ] Macro FX visibly respond to zoom without breaking world-anchored tactical fire.

---

## 6. Doc maintenance

When a **P0/P1** row in the plan file flips to `✓`, add one line under that item in **this** file’s “Changelog” (optional subsection you append):

```text
2026-05-10 — P0-A–D: dirty raster revision + stable egui texture ids + preview partial throttle (~12 Hz) + HUD ops strip / build / narrative string caches (`tile_world_fallback.rs`, `world_preview/*`, `in_game_hud.rs`).
2026-05-10 — P1-E~ / P1-F ✓ / P1-G~: `FireVisualProxy` type alias; `MapCameraDesired` + smoothing + keybinds + `SharedOverlayFieldBuffers` stub plugin (`map_camera.rs`, `input_bindings.rs`, `overlay_field_buffers.rs`, engine plugin list).
2026-05-10 — P1-E ✓ / P1-G ✓ / minimap: `FireVisualFrame` + `chunk_heat`; overlay **only** from frame; world preview + sim tile fallback; `TileWorldFallbackAfterFireExtract` after `FireVisualFrameSet::BuildProfiles`; MAP ops strip + `MapCameraMode` cycle (M).
2026-05-10 — Latency: map editor stable egui texture + `MapEditorMinimapRasterDirty` epoch raster; world preview overlay-only throttle; `InputFrame` (`input_frame.rs`) for camera delta read surface.
2026-05-10 — Fire GPU path: `ExtractResourcePlugin<FireVisualFrame>`; `FireVisualGpuInstanceStorage` + `weather_fire_field.wgsl` `@group(1)`; uniforms `fire_instance_count`; smoke-only `publish_sim_visual_extract`; `SimFireEmitterVisualExtract` not extracted to render.
2026-05-10 — **Canonical contract:** `FireVisualFrame` = sole CPU fire snapshot (`instances` + `chunk_heat`); `SharedOverlayFieldBuffers` derived from frame only; `FireVisualFrameSet` / `FireVisualFramePlugin`.
```

Keeps the long `base_visual_dev01.md` stable while status lives in the split files.

---

## 7. Root cause map (latency)

| Area | What was wrong | Mitigation in repo | Still open |
|------|----------------|-------------------|------------|
| **A. egui + texture churn** | Map editor minimap registered a new egui texture every frame (`add_image` per pass). | `MapEditorMapTexture::egui_texture_cache` — reuse `TextureId` until the `Handle<Image>` changes; cache cleared in `map_editor_sync_map_texture_size` when the image is recreated. | Scan other egui panels for the same pattern. |
| **B. World preview dirty key** | Fire overlay revision could trigger a **full** CPU pass every `Update` tick. | Treat overlay-only invalidation like partials: same ~12 Hz gate as chunk dirty (`world_preview/render_raster.rs`). World raster still driven by material epoch + dirty queue + layers + tex size (not egui viewport zoom). | Visible-chunk slice cache (P0-2) not built yet. |
| **C. Input coalescing** | Event backlog risk on camera paths. | Bevy’s `AccumulatedMouseMotion` is already per-frame summed; `InputFrame` (`gui/input_frame.rs`) snapshots it in `PreUpdate`; `map_camera` grip pan reads `InputFrame::pointer_delta`. | Optional: merge other pointer sources into `InputFrame`. |
| **D. Fire visual consistency** | Many consumers re-query sim fire. | Single path: **`FireVisualFrame`** (ECS once) → overlay from `chunk_heat` only → cluster/lights from `instances`; `SharedOverlayFieldBuffers` is a **view**, not a second sim scan. | Particle/smoke stubs should read only frame/overlay; multi-rate rasters still future. |
