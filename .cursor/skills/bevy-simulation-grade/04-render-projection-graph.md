# Render Projection Graph + Fire Extract

**Repo:** `src/render/extraction/`, `src/render/extraction/render_projection_graph.rs`, `src/render/extraction/fire_visual_extract.rs`

## Purpose

Sim + committed snapshots → `RenderProjectionContext` → `FireVisualFrame` → GPU project.

## Core resources

```rust
pub struct RenderProjectionContext {
    pub representation: RepresentationResult,
}

#[derive(Resource)]
pub struct RenderProjectionGraph { /* nodes */ }
```

`RepresentationResult` is the Stage 5 convergence contract — extend via graph nodes, not parallel ad-hoc extractors.

## FireVisualFrameSet (Update)

| Phase | Role |
|-------|------|
| `BuildProfiles` | Sim scan, snapshots, view visibility — **after `SyncViewManager`** |
| `BuildClusters` … `EmitDomainOverlays` | Derived fire/logistics/ecology visuals |
| `ProjectGpu` | GPU projection graph execution |

Typical dependents:

```rust
.after(FireVisualFrameSet::BuildProfiles)
.before(FireVisualFrameSet::ProjectGpu)  // when inserting between phases
```

## Projection nodes

Implement `ProjectionNodeTrait` / graph registration in extraction module — **read-only** sim and snapshot inputs.

## Rules

- Extraction is **read-only** for gameplay resources.
- Do not write `ViewManager` or `ViewProjectionAuthority` from extract systems.
- `SyncOverlayField` runs **after** `BuildProfiles` (shared fire buffers).

## Anti-patterns

- `Box<dyn ProjectionNode>` in hot path without matching repo pattern
- Tactical-only assumptions that bypass `ViewId`
- Second extract path for same visual layer (Stage 5 triage)
