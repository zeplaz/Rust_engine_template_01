# View Authority + ViewManager

**Repo:** `src/gui/view_authority.rs` · **Schedule:** [07-repo-authority-map.md](07-repo-authority-map.md)

## Purpose

One read spine per `ViewId` after committed projection and viewport resolve.

## ViewId (repo)

```rust
pub enum ViewId {
    WorldMain,
    WorldPreview,
    Minimap,
    SimulationMap,
}
```

## ViewManager — frame-rebuilt read model

```rust
#[derive(Resource, Default)]
pub struct ViewManager {
    pub views: HashMap<ViewId, ViewInstance>,
}
```

### Production writer

| Writer | When |
|--------|------|
| **`sync_view_manager_bridge`** | Every frame in `ViewAuthoritySystemSet::SyncViewManager` |

Rebuilds from `ViewProjectionAuthority` + `ResolvedViewports` + per-view components — **not** from UI ad-hoc mutation.

### Anti-patterns

- Any other `ResMut<ViewManager>` in production
- UI writing `ViewInstance` transforms directly
- Render systems adjusting authoritative camera pose
- Using `ViewManager` for GPU upload without `ViewRepresentationSnapshot`

## ViewProjectionAuthority (pose commit)

RTS / minimap / session code **commits** poses here first:

```rust
authority.commit_pose(ViewSurfaceId::WorldMain, cam, ViewAuthorityWriter::MapCameraInput);
```

`MapCameraDesired` is updated only by **`derive_map_camera_desired_from_view_authority`** (compatibility mirror).

## ViewCameraTag

At most one Bevy camera entity per `ViewId` (debug-asserted). World main: `MainWorldCamera` + `ViewCameraTag(WorldMain)`.

## Correct flow

```text
Input → ViewProjectionAuthority.commit_pose
     → ViewportPipelineSet::Resolve (ResolvedViewports)
     → sync_view_manager_bridge (ViewManager)
     → build_view_representation_snapshot
     → FireVisualFrameSet / raster consumers
```

## Ordering rule

```rust
.after(ViewportPipelineSet::Resolve)
.after(MapCameraSystemSet::ApplyInput)
.in_set(ViewAuthoritySystemSet::SyncViewManager)
```
