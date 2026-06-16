# Map View + Render Contract

**Repo:** `src/gui/view_representation_snapshot.rs`, `src/gui/map_view/`, `src/gui/view_representation.rs`

## Purpose

Bridge committed view authority → **immutable snapshot** → GPU/CPU raster and extraction.

## Snapshot (GPU contract)

```rust
// ViewRepresentationSnapshot — built after SyncViewManager
pub struct ViewRepresentationSnapshot {
    pub per_view: HashMap<ViewId, CameraVisualState>,
}
```

| System | Set |
|--------|-----|
| `build_view_representation_snapshot` | `ViewRepresentationSystemSet::CameraSync` |
| `validate_view_representation_snapshot` | `ViewRepresentationSystemSet::PostFX` |

**Consumers:** tile fallback, minimap compositor, fire extract visibility — read snapshot / `CameraVisualState`, not stale `ViewManager` alone.

## MapViewInstance

Presentation components under `src/gui/map_view/` — separate from authority resources.

## WorldRepresentationFrame

Registered via `world_representation::register_world_representation_frame` — tactical/world LOD bands; ordered relative to fire extract (see 07).

## Rules

1. **After** `ViewAuthoritySystemSet::SyncViewManager` for any per-view camera math used in render.
2. Do not mutate sim state from map view presentation systems.
3. Stage 5: one authoritative path to `RepresentationResult` — no duplicate LOD extractors.

## Sim map tactical projection (construction + egui overlays)

GPU camera and manual egui math share **`SimMapProjectionFrame`** (`src/gui/map_camera.rs`):

- `screen_rect` — map hole or full window (from live `Camera::viewport`, not latch alone)
- `visible_w/h` — world span for manual inverse/project (≈ `MainWorldCameraOrthoTrace.view_pixels`)
- `fixed_w/h` — Bevy `ScalingMode::Fixed` params (= view_px / zoom); **not** the manual span

See **[09-sim-map-projection-placement.md](09-sim-map-projection-placement.md)** for pick/ghost debug metrics and scissor-heal rules.

## Anti-patterns

- Reading `MapCameraDesired` as truth — use `ViewProjectionAuthority` / snapshot
- GPU upload driven by egui layout without resolved viewport revision check
- Sizing ortho `view_px` from sim map hole while `camera.viewport` is full window
