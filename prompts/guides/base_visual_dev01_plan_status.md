# Visual dev plan & status (split from `base_visual_dev01.md`)

**Parent doc:** [`base_visual_dev01.md`](base_visual_dev01.md) — full rationale, sketches, and code examples stay there.

**Companion:** [`base_visual_dev01_roadmap_next.md`](base_visual_dev01_roadmap_next.md) — what to do after this checklist, north-star sequencing.

**How to use:** Keep items **rough**; flip status when behavior matches the intent (not when every sub-bullet exists). Update dates in the status column when you touch an item.

---

## Three major themes (map to the parent doc)

| Part | Parent doc focus | Engineering center |
|------|------------------|----------------------|
| **1st** | Strategic world layer vs atmospheric “outside world” layer; camera UX, modes, recenter/edge | **Cameras + ownership + fun outside the frame** (macro VFX reacts to zoom/camera, not sim truth) |
| **2nd** | Multi-rate pipelines, input vs UI vs extraction, dirty flags, minimap/preview, world vs screen particles | **Latency + invalidation** (what runs every frame vs 5–20 Hz) |
| **3rd** | Single fire visual proxy, shared overlays, P2 refactors, GPU path | **One extract → many consumers** + scale-out |

---

## Master priority order (from parent tail + “HIGHEST PRIORITY FIX ORDER”)

Status: `○` not started · `~` in progress · `✓` done enough for current milestone · `!` blocked / needs design

| ID | Item | Status | Note (rough) |
|----|------|--------|--------------|
| P0-A | **Dirty raster** (`tile_world_fallback` + world preview invalidation), not `(w,h,tile_count)` only | ✓ | `TileWorldFallbackRasterDirty` + ECS `Added`/`Changed` + `TerrainRegistriesHandles::is_changed`; revision skip in raster |
| P0-B | **Stable egui texture handles** (preview/minimap); avoid per-frame `add_image` churn | ✓ | `Local<Option<(Handle<Image>, TextureId)>>` in minimap + world preview window |
| P0-C | **Preview `run_if` / throttle** — no full CPU raster at 60 Hz when nothing changed | ✓ | Partial ~12 Hz; **overlay-only** full passes throttled same rate (`render_raster.rs`); keys = epoch / dirty / layers / tex / overlay rev (not egui zoom) |
| P0-D | **HUD dirty flags** — cached strings; no continuous `format!` for static labels | ✓ | Ops strip fingerprints + text write only on change; build line fp; narrative `is_changed` + line cache |
| P1-E | **Single CPU fire snapshot + GPU upload** (proxy row + frame + overlay derive) | ✓ | **Two CPU concepts:** [`FireVisualFrame`](../../src/render/extraction/fire_visual_extract.rs) (`FireVisualProxy` / [`FireVisualGpuInstance`](../../src/render/sim_visual_extract.rs) rows + [`ChunkFireHeat`](../../src/render/sim_visual_extract.rs)); [`SharedOverlayFieldBuffers`](../../src/render/overlay_field_buffers.rs) **only** from `FireVisualFrame::chunk_heat`. GPU: [`FireVisualGpuInstanceStorage`](../../src/render/gpu_weather_fire_field.rs). ECS `ChunkSurfaceFire` **only** in `extract_fire_visual_frame`. |
| P1-F | **Camera smoothing + `run_if`** (`CameraTarget` vs current `Transform`) | ✓ | `MapCameraDesired` + chained lerp; edge toggle / recenter / Z reset / B frame-world; ScrollLock edge; `focus_main_camera` syncs desired |
| P1-G | **Shared minimap + preview overlay** (derived view model) | ✓ | `SharedOverlayFieldBuffers::chunk_fire_heat` **from `FireVisualFrame` only** (no second ECS fire scan); world preview + sim tile fallback / egui minimap via `apply_shared_fire_heat_to_rgba`; dirty on overlay `revision`; raster after `FireVisualFrameSet::BuildProfiles`. |
| P2-H | **Incremental atmosphere** (dirty rects + periodic full refresh) | ○ | High risk — design + tests before code |
| P2-I | **FixedUpdate sim** (fire/ecology/logistics) vs Update input/camera/UI | ○ | Large ordering refactor |
| P2-J | **GPU particles** (instanced quads → Hanabi/compute) | ○ | Only after P0 raster + egui churn tamed |

---

## Consolidated backlog — P1 through end (execution order)

Rough dependency order; flip rows in the master table above when done.

**P1 — visuals & camera polish**

1. **P1-E** — Canonical **`FireVisualFrame`**: proxy rows + `chunk_heat`; overlay derived from frame; GPU upload is prepare stage of that frame. **No** extra `ChunkSurfaceFire` in render consumers (only `extract_fire_visual_frame` + `infer_*`). *(✓ 2026-05-10; frame contract 2026-05-10.)*
2. **P1-F** — `MapCameraDesired` + lerp apply; edge-scroll toggle; Home / Z / frame-world; double-Space & double–middle-mouse recenter; optional `run_if` when idle. *(✓ core: desired + smooth + bindings + focus sync; double middle recenter.)*
3. **P1-G** — `SharedOverlayFieldBuffers` filled from extract; minimap + world preview sample same chunk heat map (roads/editor minimap still terrain-only). *(✓)*
4. **Theme 1st** — `MapCameraMode` + ops strip MAP line + cycle keybind (M); optional `CameraControlState` / Follow / FreePan and richer HUD still open.
5. **Theme 2nd** — Particle markers `WorldFireFx` / `AtmosphereFx`; preview double-buffer (if tearing persists); input coalescing audit.
6. **Theme 3rd** — Zoom LOD rules for fire visibility; overlay channel matrix vs stubs.

**P2 — sim cadence & GPU scale**

7. **P2-H** — Atmosphere dirty rects + periodic full refresh (design doc + tests before code).
8. **P2-I** — `FixedUpdate` sim slice + interpolation for render (fire/ecology/logistics ordering).
9. **P2-J** — Instanced quads / Hanabi path once CPU particle budget fails.

**Injection queue (misc)**

10. PresentMode / mailbox pass; egui scissor for heavy panels; optional `InputFrameState`; transport event cadence notes.

---

## 1st theme — layers & cameras (planned todos not fully in table above)

| Todo | Status | Note |
|------|--------|------|
| **Resource:** `CameraControlSettings` or `CameraControlState` — edge scroll toggle, default zoom, mode | ~ | Edge + zoom live on `MapCameraSettings` / `MapCameraDesired`; consolidated `CameraControlState` not added |
| **Actions:** Recenter (Home / Space×2 / double middle), reset zoom (Z / Shift+Home), frame world (`fit_world_bounds`) | ~ | Home / Z / B + double middle ✓; Space×2 not wired |
| **Enum:** `CameraMode` — Strategic / Tactical / Cinematic / FollowEntity / FreePan | ~ | `MapCameraMode`: STRAT / TACT / CINE + cycle key; Follow / FreePan still ○ |
| **HUD strip:** tiny camera state (edge on/off, zoom, mode, follow target) | ~ | Bevy ops strip MAP line (mode, edge, zoom, yaw); follow target N/A |
| **Clarify “two cameras” mentally:** (A) Bevy `MainWorldCamera` gameplay view (B) egui preview = texture, not a second game camera yet | ○ | Long-term: render target inside egui (parent §9) |
| **Outside-frame fun:** macro layer stays **camera-reactive**, **not sim-authoritative**; document in code near atmosphere drivers | ○ | Zoom out → plumes/haze; zoom in → tactical smoke |

---

## 2nd theme — pipelines, particles, UI perf

| Todo | Status | Note |
|------|--------|------|
| **Stages:** input collect → camera → UI intent → visual invalidation flags → low-rate extract → render | ○ | Align with `SystemSet` names in parent; incremental adoption |
| **Minimap:** cached texture; dirty on camera chunk / fire / overlay change | ~ | Sim minimap: stable egui texture + `TileWorldFallbackRasterDirty` + **fire tint** from `SharedOverlayFieldBuffers`; camera-chunk dirty not granular yet |
| **Preview:** double-buffer RGBA or swap handles to avoid upload tear | ○ | Parent “bars” hypothesis: timing + empty tiles |
| **Split particles:** `WorldFireFx` vs `AtmosphereFx` (world-scaled vs screen/macro) | ○ | Fixes zoom-scaling wrong class bug |
| **Event coalescing:** latest mouse only per frame | ○ | Where cursor readers still loop events |
| **Camera never waits on:** worldgen, minimap raster, smoke extract, heavy egui | ○ | Ordering audit |

---

## 3rd theme — extraction, overlays, GPU path

| Todo | Status | Note |
|------|--------|------|
| **Formalize** `FireVisualProxy` / single consumer buffer from `ChunkSurfaceFire` once per tick | ~ | Extract + `SharedOverlayFieldBuffers` + preview/minimap/light path; aggregate `FireAtmosphereAggregate` still separate row |
| **LOD visibility rules** by zoom: strategic blobs vs mid vs close flame sprites | ○ | Design table in parent §6 / fire visibility |
| **Overlay channel list** height/temp/moisture/ecology/smoke/fire/wind/mobility/pressure — shared sampler | ○ | Many today preview-only or stub |
| **GPU weather-fire field** — keep as visual-only; no sim readback without gate | ○ | Already directionally right |

---

## Bottom injection queue (misc from parent not mapped above)

Add here when you pull new bullets from `base_visual_dev01.md`:

- [ ] PresentMode / mailbox review vs “feels like input lag”
- [ ] Viewport scissoring for large egui regions
- [ ] `InputFrameState`-style fast path (optional resource)
- [ ] Transport/logistics: event-driven cadence called out in parent timing table

---

*Last template edit: 2026-05-10 — P1-E/P1-G closed out for current milestone; theme 1 partial; minimap fire tint wired.*
