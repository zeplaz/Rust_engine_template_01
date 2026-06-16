# Viewport Authority Pipeline

**Repo:** `src/gui/viewport_authority.rs`, `src/render/viewport_pipeline.rs`

## Purpose

UI submits **requests**; resolve phase commits **ResolvedViewports** once per frame.

## Request side (UI / HUD)

```rust
// src/gui/viewport_authority.rs
pub struct ViewportRequest {
    pub logical_rect: egui::Rect,
    pub priority: u8,
    pub world_extent: UVec2,
}

pub struct ViewportAuthority {
    pub pending: Vec<ViewportRequest>,
    pub requested: Option<ViewportRequest>,
    pub resolved: Option<ResolvedViewport>,
    pub revision: u64,
}
```

Submit via `submit_viewport_request`; do not write `ResolvedViewports` from UI systems.

## Resolved side (commit)

```rust
// src/render/viewport_pipeline.rs
pub struct ResolvedViewports {
    pub world_preview: ResolvedViewport,
    pub minimap_panel: ResolvedViewport,
    pub simulation_map: ResolvedViewport,
    pub primary_window: ResolvedViewport,
    pub revision: u64,
}
```

## Resolve chain (production)

All in **`ViewportPipelineSet::Resolve`** (`ViewRepresentationSystemSet::ResolveViewport`):

```text
resolve_preview_viewport_requests
resolve_primary_and_simulation_viewports
resolve_minimap_panel_viewport
commit_resolved_viewports_to_authority
sync_resolved_viewports_from_authority
apply_map_view_extents_from_authority
clear_viewport_requests_after_resolve
```

## Hard rules

- Only the **Resolve** set chain writes committed layout for `ResolvedViewports`.
- **Do not** clear `world_preview` on empty `pending` without reading `viewport_pipeline.rs` comments (readiness flap hazard).
- Minimap camera intent: `apply_minimap_camera_intent` in **ResolveViewport** — updates minimap authority, **not** `MapCameraDesired`.

## Debug

- `ViewportPresentationMismatch` — extent / texture binding flags for VT witnesses.

## Anti-pattern

Illustrative `HashMap<ViewId, URect>`-only resolve — **not** the repo shape; use struct fields above.
