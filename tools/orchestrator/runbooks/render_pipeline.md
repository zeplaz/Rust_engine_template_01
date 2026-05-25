# Render pipeline runbook

## File map

| Node | File |
|------|------|
| Projection graph | `src/render/extraction/render_projection_graph.rs` |
| Fire extract | `src/render/extraction/fire_visual_extract.rs` |
| View extract | `src/render/extraction/fire_view_extract.rs` |
| Resolved viewports | `src/render/viewport_pipeline.rs` (ViewportPipelinePlugin) |
| Visual diagnostics | `src/render/visual_diagnostics.rs` |
| Tile fallback | `src/render/tile_world_fallback.rs` |

## Spine

```text
RepresentationResult + WorldLodMap
    → RenderProjectionGraph (CPU nodes)
    → FireVisualFrame / buffers
    → ResolvedViewports (follows GUI semantic authority)
    → GPU upload / draw
```

## Coordination

Render agents treat GUI viewport authority as upstream. Never re-derive simulation map geometry from window chrome.

## STAGE5

- TODO-06–11 — frame fence, fire alignment, GPU spine (`STAGE5_TODOS`)
