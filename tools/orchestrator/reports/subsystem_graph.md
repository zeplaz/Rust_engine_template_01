# Subsystem graph

## Viewport authority spine

```text
viewport authority
    ├── semantic viewport (viewport_layout_solver)
    ├── ui measured rect (authoritative_viewport)
    ├── layout solver (commit_authority_from_semantic)
    ├── camera viewport (map_camera / MainWorldCamera)
    ├── minimap shell (MinimapShellState)
    ├── world preview (map_view / world_preview)
    ├── render sync (ResolvedViewports)
    └── debug tracing (sim_view_sync_debug, viewport_authority_debug)
```

## Map presentation spine

```text
map_view
    ├── backend / texture cache
    ├── presentation state
    ├── projection / resolved frames
    ├── minimap consumer
    └── world_preview consumer
```

## Render extraction spine

```text
render
    ├── RenderProjectionGraph
    ├── fire visual extract
    └── viewport resolve sets
```

