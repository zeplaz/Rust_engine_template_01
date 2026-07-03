//! View authority spine — **one `ViewInstance` per `ViewId`** (bridge from legacy resources).
//!
//! **View projection authority:** treat [`ViewManager`] as the read spine for per-view
//! [`ViewCameraState`], [`ViewInstance::viewport_rect`], and [`ViewInstance::visible_world_rect`].
//! **TRIAGE-VM-09-v2:** [`MapCameraDesired`] is a **read-only compatibility mirror** — RTS input and shell
//! jumps commit [`ViewProjectionAuthority`] first; [`derive_map_camera_desired_from_view_authority`]
//! is the sole `ResMut<MapCameraDesired>` writer in production (see [`crate::gui::map_camera`]).
//! **VM-06:** [`sync_view_manager_bridge`] is the **sole** `ResMut<ViewManager>` writer — it rebuilds the
//! read model from authority after viewport resolve. [`sync_view_manager_world_main_from_authority`] is
//! a test/helper partial sync only (not scheduled).
//! Minimap shell camera jumps are committed in [`ViewRepresentationSystemSet::ResolveViewport`](crate::gui::view_representation::ViewRepresentationSystemSet)
//! **before** [`ViewAuthoritySystemSet::SyncViewManager`] so the bridge does not lag one frame.
//!
//! Short-term: `sync_view_manager_bridge` mirrors committed contracts so ownership boundaries
//! can migrate incrementally.
//!
//! **vm-06:** [`ViewIsolationDiagnostics`] flags suspicious **minimap ↔ main** camera lockstep when
//! the minimap is not in [`crate::gui::MinimapFollowMode::FollowCamera`] (possible cross-view bleed).

use std::collections::HashMap;

use bevy::math::{Rect, Vec2};
use bevy::prelude::*;

use crate::gui::map_camera::{MainWorldCamera, MapCameraDesired, MapCameraSystemSet, MAIN_WORLD_CAMERA_Z};
use crate::gui::map_view::{MapViewInstances, MapViewPresentationStates};
use crate::gui::{MapViewState, MinimapFollowMode};
use crate::gui::MinimapOverlayMask;
use crate::gui::world_representation::WorldLodBand;
use crate::render::{ResolvedViewports, ViewportPipelineSet};

/// Per-view camera pose in world/tile space (authoritative for projection after bridge sync).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewCameraState {
    pub translation: Vec2,
    pub zoom: f32,
    pub rotation: f32,
}

impl Default for ViewCameraState {
    fn default() -> Self {
        Self {
            translation: Vec2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
        }
    }
}

impl ViewCameraState {
    #[must_use]
    pub fn to_projection(&self) -> ViewProjection {
        ViewProjection {
            camera_center_world: self.translation,
            zoom: self.zoom,
        }
    }
}

/// Core view buckets (aligns with prior `ViewId` / `MapViewInstanceId` subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewId {
    WorldMain,
    WorldPreview,
    Minimap,
    SimulationMap,
}

/// Where this view rasterizes (Bevy camera target abstraction).
#[derive(Clone, Debug, PartialEq)]
pub enum ViewRenderTarget {
    PrimaryWindow,
    Image(Handle<Image>),
    None,
}

#[derive(Clone, Debug)]
pub struct ViewProjection {
    pub camera_center_world: Vec2,
    pub zoom: f32,
}

impl Default for ViewProjection {
    fn default() -> Self {
        Self {
            camera_center_world: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ViewInteractionState {
    pub pan_delta: Vec2,
    pub zoom_factor: f32,
    pub hovered_tile: Option<UVec2>,
}

#[derive(Clone, Debug)]
pub struct OverlayMask {
    pub bits: MinimapOverlayMask,
}

impl Default for OverlayMask {
    fn default() -> Self {
        Self {
            bits: MinimapOverlayMask {
                fire_heat: true,
                logistics_heat: false,
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DebugFlags {
    pub show_lod_grid: bool,
    pub show_viewport_outline: bool,
}

/// Per-view presentation filters (bitset); reserved for map filter isolation (Phase 1).
/// **vm-08:** overlay bitfields for preview/minimap live in [`crate::gui::MapViewInstances`]; UI uses
/// [`crate::gui::map_overlay_controls_ui`] with a distinct `id_prefix` per surface — do not write one
/// view's `MapViewState` from another surface's widgets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ViewFilterMask(pub u32);

/// Per-view render policy: LOD band, overlays, filters, debug flags (isolates minimap vs main).
#[derive(Clone, Debug)]
pub struct ViewRenderPolicy {
    pub lod_band: WorldLodBand,
    pub overlays: OverlayMask,
    pub filter_mask: ViewFilterMask,
    pub debug_flags: DebugFlags,
}

impl Default for ViewRenderPolicy {
    fn default() -> Self {
        Self {
            lod_band: WorldLodBand::Strategic,
            overlays: OverlayMask::default(),
            filter_mask: ViewFilterMask::default(),
            debug_flags: DebugFlags::default(),
        }
    }
}

/// Single owned snapshot for one view (no shared mutation across ids).
#[derive(Clone, Debug)]
pub struct ViewInstance {
    pub id: ViewId,
    pub camera_entity: Entity,
    pub render_target: ViewRenderTarget,
    /// Authoritative per-view camera; [`projection`] mirrors this after bridge sync.
    pub camera: ViewCameraState,
    pub projection: ViewProjection,
    pub interaction_state: ViewInteractionState,
    /// Logical rect in **window** space when known; may be invalid (zero) before first layout.
    pub viewport_rect: Rect,
    pub render_policy: ViewRenderPolicy,
}

impl ViewInstance {
    /// World XY rectangle visible in this view (center/zoom from [`ViewCameraState`], size from viewport).
    #[must_use]
    pub fn visible_world_rect(&self) -> Rect {
        let c = self.camera.translation;
        let z = self.camera.zoom.max(1e-5);
        let w = self.viewport_rect.width().max(1.0);
        let h = self.viewport_rect.height().max(1.0);
        let half_w = w / (2.0 * z);
        let half_h = h / (2.0 * z);
        Rect::from_center_half_size(c, Vec2::new(half_w, half_h))
    }
}

#[derive(Resource, Default, Debug)]
pub struct ViewManager {
    pub views: HashMap<ViewId, ViewInstance>,
}

impl ViewManager {
    #[must_use]
    pub fn view(&self, id: ViewId) -> Option<&ViewInstance> {
        self.views.get(&id)
    }
}

/// WorldMain camera center + zoom (authority-only; [`ViewManager`] / [`MapCameraDesired`] are read mirrors).
#[inline]
#[must_use]
pub fn tactical_camera_world_pose(
    authority: Option<&crate::render::view_runtime::ViewProjectionAuthority>,
    manager: &ViewManager,
    desired: &MapCameraDesired,
) -> (Vec2, f32) {
    use crate::render::view_runtime::ViewSurfaceId;

    if let Some(auth) = authority {
        if let Some(surface) = auth.surface(ViewSurfaceId::WorldMain) {
            return (
                surface.camera.translation,
                surface.camera.zoom.max(1e-4),
            );
        }
    }
    if let Some(view) = manager.view(ViewId::WorldMain) {
        return (view.camera.translation, view.camera.zoom.max(1e-4));
    }
    (desired.translation.truncate(), desired.scale.x.max(1e-4))
}

/// Build [`MapCameraDesired`] from authoritative WorldMain pose (**TRIAGE-VM-09-v2** derive shim).
#[inline]
#[must_use]
pub fn map_camera_desired_from_view_authority(
    authority: &crate::render::view_runtime::ViewProjectionAuthority,
) -> MapCameraDesired {
    use crate::render::view_runtime::ViewSurfaceId;

    let Some(cam) = authority
        .surface(ViewSurfaceId::WorldMain)
        .map(|s| s.camera)
    else {
        return MapCameraDesired::default();
    };
    MapCameraDesired {
        translation: Vec3::new(cam.translation.x, cam.translation.y, MAIN_WORLD_CAMERA_Z),
        scale: Vec3::splat(cam.zoom.max(1e-4)),
        rotation: Quat::from_rotation_z(cam.rotation),
    }
}

/// Build [`ViewCameraState`] from gameplay [`MapCameraDesired`] (2D map plane; yaw lives in Quat but
/// [`ViewCameraState::rotation`] stays 0 until per-view rotation is wired through the bridge).
#[inline]
#[must_use]
pub fn view_camera_state_from_map_camera_desired(desired: &MapCameraDesired) -> ViewCameraState {
    ViewCameraState {
        translation: desired.translation.truncate(),
        zoom: desired.scale.x,
        rotation: 0.0,
    }
}

/// VM-C C1: RTS input commits pose to [`ViewProjectionAuthority`] (authoritative write).
pub fn commit_map_camera_pose_to_view_authority(
    authority: &mut crate::render::view_runtime::ViewProjectionAuthority,
    trace: &mut crate::render::view_runtime::ViewRuntimeTrace,
    desired: &MapCameraDesired,
) {
    let cam = view_camera_state_from_map_camera_desired(desired);
    let writer = crate::render::view_runtime::ViewAuthorityWriter::MapCameraInput;
    if trace.enabled {
        authority.commit_pose_traced(
            crate::render::view_runtime::ViewSurfaceId::WorldMain,
            cam,
            writer,
            Some(trace),
        );
        authority.commit_pose_traced(
            crate::render::view_runtime::ViewSurfaceId::SimulationMap,
            cam,
            writer,
            Some(trace),
        );
    } else {
        authority.commit_pose(
            crate::render::view_runtime::ViewSurfaceId::WorldMain,
            cam,
            writer,
        );
        authority.commit_pose(
            crate::render::view_runtime::ViewSurfaceId::SimulationMap,
            cam,
            writer,
        );
    }
}

/// Commit a world XY focus jump on WorldMain (shell intel — authority before desired mirror).
pub fn commit_world_main_map_focus(
    authority: &mut crate::render::view_runtime::ViewProjectionAuthority,
    trace: &mut crate::render::view_runtime::ViewRuntimeTrace,
    world_xy: Vec2,
) {
    let mut desired = map_camera_desired_from_view_authority(authority);
    desired.translation.x = world_xy.x;
    desired.translation.y = world_xy.y;
    commit_map_camera_pose_to_view_authority(authority, trace, &desired);
}

/// Partial read-model sync for tests — production uses [`sync_view_manager_bridge`] full rebuild.
pub fn sync_view_manager_world_main_from_authority(
    manager: &mut ViewManager,
    authority: &crate::render::view_runtime::ViewProjectionAuthority,
) {
    let Some(cam) = authority
        .surface(crate::render::view_runtime::ViewSurfaceId::WorldMain)
        .map(|s| s.camera)
    else {
        return;
    };
    let projection = cam.to_projection();
    match manager.views.get_mut(&ViewId::WorldMain) {
        Some(inst) => {
            inst.camera = cam;
            inst.projection = projection;
        }
        None => {
            manager.views.insert(
                ViewId::WorldMain,
                ViewInstance {
                    id: ViewId::WorldMain,
                    camera_entity: Entity::PLACEHOLDER,
                    render_target: ViewRenderTarget::PrimaryWindow,
                    camera: cam,
                    projection,
                    interaction_state: ViewInteractionState::default(),
                    viewport_rect: Rect::default(),
                    render_policy: ViewRenderPolicy::default(),
                },
            );
        }
    }
}

/// Latest world-representation LOD band mirrored for each [`ViewId`] (vm-10 hook for per-view policy).
#[derive(Resource, Clone, Debug, Default)]
pub struct PerViewLodHints {
    pub by_view: HashMap<ViewId, WorldLodBand>,
}

/// Per-frame **heuristic** checks for view isolation (Stage 5 / `base_finsh_5.md` §1).
///
/// A `true` flag is a **suspected** authority bleed — confirm with follow mode and user intent.
#[derive(Resource, Clone, Debug, Default)]
pub struct ViewIsolationDiagnostics {
    /// Minimap camera center/zoom matches main map while follow mode is not `FollowCamera`.
    pub minimap_main_lockstep_suspect: bool,
    /// World preview matches main tactical camera while preview is initialized (often false-positive at map center).
    pub preview_main_lockstep_suspect: bool,
    /// [`ViewId::SimulationMap`] intentionally mirrors main today — informational for audits.
    pub simulation_map_shares_main_camera: bool,
    /// `MapViewInstances::world_preview.overlays.fire_heat` (vm-08 snapshot).
    pub preview_overlay_fire_heat: bool,
    /// `MapViewInstances::minimap.overlays.fire_heat` (vm-08 snapshot).
    pub minimap_overlay_fire_heat: bool,
    /// `ViewManager` WorldMain overlay flag matches [`MapViewInstances`] sim overlays.
    pub world_main_overlay_fire_heat: bool,
    /// VM-08: preview/minimap overlay masks match independent `MapViewState` (no cross-write).
    pub vm08_overlay_masks_aligned: bool,
}

#[inline]
#[must_use]
pub(crate) fn minimap_main_lockstep_suspect(
    minimap: &MapViewState,
    main_translation: Vec2,
    main_zoom: f32,
) -> bool {
    if matches!(
        minimap.follow_mode,
        MinimapFollowMode::FollowCamera | MinimapFollowMode::FollowBookmark
    ) {
        return false;
    }
    const EPS_POS: f32 = 0.75;
    const EPS_Z: f32 = 0.02;
    (minimap.camera_center - main_translation).length() < EPS_POS
        && (minimap.zoom - main_zoom).abs() < EPS_Z
}

#[inline]
#[must_use]
pub(crate) fn preview_main_lockstep_suspect(
    preview: &MapViewState,
    main_translation: Vec2,
    main_zoom: f32,
) -> bool {
    if !preview.camera_initialized {
        return false;
    }
    const EPS_POS: f32 = 0.75;
    const EPS_Z: f32 = 0.02;
    (preview.camera_center - main_translation).length() < EPS_POS
        && (preview.zoom - main_zoom).abs() < EPS_Z
}

/// Sentinel when no camera entity is bound yet.
pub const VIEW_NO_ENTITY: Entity = Entity::PLACEHOLDER;

/// Bevy camera entity registered for a [`ViewId`] (at most **one** entity per id — enforced in debug).
///
/// Today only [`ViewId::WorldMain`] is tagged on the gameplay [`MainWorldCamera`]. Other views may
/// still use [`VIEW_NO_ENTITY`] until dedicated raster cameras exist (preview GPU path is separate).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ViewCameraTag(pub ViewId);

/// Runs in [`Update`] after [`ViewportPipelineSet::Resolve`](crate::render::ViewportPipelineSet) so
/// [`ViewManager`] is fresh before [`crate::render::extraction::FireVisualFrameSet::BuildProfiles`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ViewAuthoritySystemSet {
    /// Tag / spawn view-bound cameras before [`ViewAuthoritySystemSet::SyncViewManager`].
    RegisterViewCameras,
    SyncViewManager,
}

pub struct ViewAuthorityPlugin;

impl Plugin for ViewAuthorityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewManager>()
            .init_resource::<PerViewLodHints>()
            .init_resource::<ViewIsolationDiagnostics>()
            .configure_sets(
                Update,
                (
                    ViewAuthoritySystemSet::RegisterViewCameras
                        .after(ViewportPipelineSet::Resolve)
                        .before(ViewAuthoritySystemSet::SyncViewManager),
                    ViewAuthoritySystemSet::SyncViewManager
                        .after(ViewportPipelineSet::Resolve)
                        .after(MapCameraSystemSet::DeriveDesired),
                ),
            )
            .add_systems(
                Update,
                (
                    ensure_main_world_view_camera_tag,
                    ApplyDeferred,
                    debug_assert_view_camera_registry,
                )
                    .chain()
                    .in_set(ViewAuthoritySystemSet::RegisterViewCameras),
            )
            .add_systems(
                Update,
                (
                    sync_view_manager_bridge,
                    sync_view_isolation_diagnostics,
                )
                    .chain()
                    .in_set(ViewAuthoritySystemSet::SyncViewManager),
            )
            .add_systems(
                Update,
                sync_per_view_lod_hints.after(ViewAuthoritySystemSet::SyncViewManager),
            );
    }
}

/// Assign [`ViewCameraTag`](ViewId::WorldMain) to the primary [`MainWorldCamera`] (first entity only).
fn ensure_main_world_view_camera_tag(
    mut commands: Commands,
    q: Query<Entity, (With<MainWorldCamera>, Without<ViewCameraTag>)>,
) {
    if let Some(entity) = q.iter().next() {
        commands
            .entity(entity)
            .insert(ViewCameraTag(ViewId::WorldMain));
    }
}

fn debug_assert_view_camera_registry(
    #[allow(unused_variables)] tags: Query<&ViewCameraTag>,
    #[allow(unused_variables)] main_cams: Query<(), With<MainWorldCamera>>,
) {
    #[cfg(debug_assertions)]
    {
        use std::collections::HashMap;

        let main_count = main_cams.iter().count();
        assert!(
            main_count <= 1,
            "expected at most one MainWorldCamera entity, found {main_count}"
        );

        let mut per_view: HashMap<ViewId, u32> = HashMap::new();
        for t in tags.iter() {
            *per_view.entry(t.0).or_insert(0) += 1;
        }
        for (id, n) in per_view {
            assert!(
                n <= 1,
                "ViewCameraTag: at most one camera entity per ViewId (found {n} for {id:?})"
            );
        }
    }
}

fn sync_view_manager_bridge(
    mut manager: ResMut<ViewManager>,
    mut authority: ResMut<crate::render::view_runtime::ViewProjectionAuthority>,
    resolved: Res<ResolvedViewports>,
    lod_frame: Option<Res<crate::gui::world_representation::WorldRepresentationFrame>>,
    map_views: Res<MapViewInstances>,
    map_presentation: Res<MapViewPresentationStates>,
    desired: Res<MapCameraDesired>,
    main_cams: Query<(Entity, Option<&ViewCameraTag>), With<MainWorldCamera>>,
) {
    crate::render::view_runtime::sync_view_authority_bridge(
        manager.as_mut(),
        authority.as_mut(),
        resolved.as_ref(),
        lod_frame.as_deref(),
        map_views.as_ref(),
        map_presentation.as_ref(),
        desired.as_ref(),
        &main_cams,
    );
}

fn sync_per_view_lod_hints(manager: Res<ViewManager>, mut out: ResMut<PerViewLodHints>) {
    out.by_view.clear();
    for (id, inst) in &manager.views {
        out.by_view.insert(*id, inst.render_policy.lod_band);
    }
}

fn sync_view_isolation_diagnostics(
    mut out: ResMut<ViewIsolationDiagnostics>,
    manager: Res<ViewManager>,
    map_views: Res<MapViewInstances>,
    authority: Res<crate::render::view_runtime::ViewProjectionAuthority>,
    desired: Res<MapCameraDesired>,
) {
    let (main_t, main_z) = tactical_camera_world_pose(Some(authority.as_ref()), manager.as_ref(), desired.as_ref());
    out.minimap_main_lockstep_suspect =
        minimap_main_lockstep_suspect(&map_views.minimap, main_t, main_z);
    out.preview_main_lockstep_suspect =
        preview_main_lockstep_suspect(&map_views.world_preview, main_t, main_z);
    out.simulation_map_shares_main_camera = true;
    out.preview_overlay_fire_heat = map_views.world_preview.overlays.fire_heat;
    out.minimap_overlay_fire_heat = map_views.minimap.overlays.fire_heat;
    out.world_main_overlay_fire_heat = manager
        .view(ViewId::WorldMain)
        .map(|v| v.render_policy.overlays.bits.fire_heat)
        .unwrap_or(false);
    out.vm08_overlay_masks_aligned =
        crate::render::view_runtime::overlay_masks_aligned_with_map_views(
            manager.as_ref(),
            map_views.as_ref(),
        );
}

/// **VM-10-MINIMAP-LOCKSTEP** — lib witness: follow modes exempt; free+match flags suspect.
#[must_use]
pub fn vm10_minimap_lockstep_diagnostics_green() -> bool {
    vm10_minimap_lockstep_self_check().is_ok()
}

fn vm10_minimap_lockstep_self_check() -> Result<(), &'static str> {
    let main_t = Vec2::new(3.0, 4.0);
    let main_z = 2.0;
    let mut free = MapViewState::default();
    free.follow_mode = MinimapFollowMode::Free;
    free.camera_center = main_t;
    free.zoom = main_z;
    if !minimap_main_lockstep_suspect(&free, main_t, main_z) {
        return Err("free_matching_main");
    }
    let mut follow = MapViewState::default();
    follow.follow_mode = MinimapFollowMode::FollowCamera;
    follow.camera_center = main_t;
    follow.zoom = main_z;
    if minimap_main_lockstep_suspect(&follow, main_t, main_z) {
        return Err("follow_camera_exempt");
    }
    let mut bookmark = MapViewState::default();
    bookmark.follow_mode = MinimapFollowMode::FollowBookmark;
    bookmark.camera_center = main_t;
    bookmark.zoom = main_z;
    if minimap_main_lockstep_suspect(&bookmark, main_t, main_z) {
        return Err("follow_bookmark_exempt");
    }
    Ok(())
}

#[cfg(test)]
mod isolation_tests {
    use super::*;

    #[test]
    fn vm10_minimap_lockstep_diagnostics_witness_green() {
        assert!(vm10_minimap_lockstep_diagnostics_green());
    }

    #[test]
    fn minimap_follow_camera_never_suspect() {
        let mut mm = MapViewState::default();
        mm.follow_mode = MinimapFollowMode::FollowCamera;
        mm.camera_center = Vec2::new(10.0, 20.0);
        mm.zoom = 1.5;
        assert!(!minimap_main_lockstep_suspect(&mm, Vec2::new(10.0, 20.0), 1.5));
    }

    #[test]
    fn minimap_free_matching_main_is_suspect() {
        let mut mm = MapViewState::default();
        mm.follow_mode = MinimapFollowMode::Free;
        mm.camera_center = Vec2::new(3.0, 4.0);
        mm.zoom = 2.0;
        assert!(minimap_main_lockstep_suspect(&mm, Vec2::new(3.0, 4.0), 2.0));
    }
}
