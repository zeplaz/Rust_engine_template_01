//! PixelWorld3D camera: a tilted perspective rig over the same map pose the top-down view uses.
//!
//! Pose truth stays [`crate::render::view_runtime::ViewProjectionAuthority`] (WorldMain): pan and
//! zoom input write it exactly as in the top-down view (`map_camera_apply_input`), and this rig
//! reads its mirror ([`MapCameraDesiredRes`]) like the 2D ortho does. The rig adds two things the
//! flat view has no use for:
//! - a free yaw and pitch ([`PixelWorldCameraRig`]): middle-mouse drag orbits (left/right) and
//!   tilts (up/down); Q / E turn a quarter ([`crate::gui::InputBindings`]); the camera eases to it;
//! - a focus height sampled from the voxel field, so the camera orbits the ground under it.
//!
//! The camera renders into the same RTT image ([`SimulationMapTexture`]) the top-down camera
//! uses; exactly one of the two is active, decided by [`crate::gui::WorldPresentationMode`].
//! [`PixelWorldCameraFrame`] records the projection actually rendered, which picking reads.
//!
//! **LGT-06 / tonemap_hdr:** PixelWorld keeps `Tonemapping::None` (display-referred palette on
//! the shared SDR RTT — fire-at-night readability; voxels in scope). See
//! [`crate::render::view_runtime::tonemap_hdr`].

use bevy::camera::{ClearColorConfig, RenderTarget};
use bevy::camera::visibility::RenderLayers;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::field::PixelWorldTerrainField;
use super::{tile_to_world3, PIXEL_WORLD_RENDER_LAYER};
use crate::engine::states::BaseState;
use crate::gui::tactical::map_camera::map_camera_rtt_view_pixels;
use crate::gui::tactical::sim_map_rtt::simulation_map_texture_extent;
use crate::gui::{MapCameraDesiredRes, SimulationMapTexture, SimulationMapViewport, WorldPresentationMode};

/// Zoom (screen px per tile at the focus) the voxel view accepts. Below the floor the whole world
/// is already in view; above the ceiling a single column fills the screen.
pub const PIXEL_WORLD_ZOOM_RANGE: (f32, f32) = (1.5, 64.0);

/// Default camera pitch below the horizon.
const PITCH_DEG: f32 = 52.0;
/// Pitch range a middle-drag tilts within (degrees below the horizon).
pub const PITCH_RANGE_DEG: (f32, f32) = (25.0, 85.0);
/// Middle-drag orbit: radians of yaw per pixel of horizontal drag.
const ORBIT_RAD_PER_PX: f32 = 0.006;
/// Middle-drag tilt: degrees of pitch per pixel of vertical drag.
const TILT_DEG_PER_PX: f32 = 0.15;
/// Vertical field of view.
const FOV_DEG: f32 = 32.0;
/// Yaw easing rate (1/s).
const YAW_EASE: f32 = 10.0;
/// Focus-height easing rate (1/s).
const FOCUS_EASE: f32 = 5.0;

/// Sky / haze colour: clear colour and distance fog share it so the far edge dissolves.
pub const PIXEL_WORLD_SKY: Color = Color::srgb(0.62, 0.71, 0.78);

/// Marker for the voxel-view camera (renders [`PIXEL_WORLD_RENDER_LAYER`] into the map RTT).
#[derive(Component)]
pub struct PixelWorldCamera;

/// Marker for the voxel-view sun.
#[derive(Component)]
pub struct PixelWorldSun;

/// Where the voxel camera should look (the rendered camera eases to it). Writers: the Q / E and
/// middle-drag input systems in this module (one domain: voxel camera input).
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct PixelWorldCameraRig {
    /// Radians; 0 = looking north, positive turns the view counter-clockwise.
    pub yaw: f32,
    /// Degrees below the horizon, within [`PITCH_RANGE_DEG`].
    pub pitch_deg: f32,
}

impl Default for PixelWorldCameraRig {
    fn default() -> Self {
        Self { yaw: 0.0, pitch_deg: PITCH_DEG }
    }
}

impl PixelWorldCameraRig {
    #[must_use]
    pub fn yaw_radians(self) -> f32 {
        self.yaw
    }

    /// Quarter turn from the nearest quarter (Q / E): +1 counter-clockwise, -1 clockwise.
    pub fn quarter_turn(&mut self, dir: f32) {
        let q = std::f32::consts::FRAC_PI_2;
        self.yaw = ((self.yaw / q).round() + dir.signum()) * q;
    }

    /// Middle-drag: orbit by the horizontal pixels, tilt by the vertical (drag up = steeper).
    pub fn drag(&mut self, delta_px: Vec2) {
        self.yaw -= delta_px.x * ORBIT_RAD_PER_PX;
        self.pitch_deg = (self.pitch_deg - delta_px.y * TILT_DEG_PER_PX).clamp(PITCH_RANGE_DEG.0, PITCH_RANGE_DEG.1);
    }
}

/// Screen-right and screen-up as tactical XY directions for a yaw (x east, y north).
#[must_use]
pub fn pixel_world_pan_axes(yaw: f32) -> (Vec2, Vec2) {
    let (s, c) = yaw.sin_cos();
    (Vec2::new(c, s), Vec2::new(-s, c))
}

/// Projection rendered last frame by [`PixelWorldCamera`] (written in PostUpdate before transform
/// propagation, read by picking in the next Update — the frame on screen).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct PixelWorldCameraFrame {
    /// The camera rendered this frame (PixelWorld3D active and presentable).
    pub active: bool,
    pub eye: Vec3,
    pub rotation: Quat,
    pub tan_half_fov: f32,
    /// RTT width / height (the projection aspect).
    pub aspect: f32,
    /// Map fill rect the RTT is stretched over (logical px).
    pub screen_min: Vec2,
    pub screen_max: Vec2,
    /// Screen px per tile at the focus point (the clamped map zoom).
    pub px_per_tile: f32,
    /// Ground point the camera orbits (tactical XY) and its height.
    pub focus: Vec2,
    pub focus_height: f32,
    /// Tactical-XY bounds of the ground the screen corners see (at the focus height).
    pub ground_min: Vec2,
    pub ground_max: Vec2,
    /// Displayed yaw (eases toward the rig).
    pub yaw: f32,
    /// Displayed pitch below the horizon, radians (eases toward the rig).
    pub pitch: f32,
}

impl PixelWorldCameraFrame {
    /// World-space ray through a logical cursor position, or `None` outside the map rect.
    #[must_use]
    pub fn ray(&self, cursor: Vec2) -> Option<(Vec3, Vec3)> {
        if !self.active {
            return None;
        }
        let size = self.screen_max - self.screen_min;
        if size.x < 1.0 || size.y < 1.0 {
            return None;
        }
        let uv = (cursor - self.screen_min) / size;
        if !(0.0..=1.0).contains(&uv.x) || !(0.0..=1.0).contains(&uv.y) {
            return None;
        }
        let ndc = Vec2::new(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
        let dir_view = Vec3::new(
            ndc.x * self.tan_half_fov * self.aspect,
            ndc.y * self.tan_half_fov,
            -1.0,
        );
        Some((self.eye, (self.rotation * dir_view).normalize()))
    }

    /// World point → logical screen position over the map rect (`None` behind the camera).
    #[must_use]
    pub fn world_to_screen(&self, p: Vec3) -> Option<Vec2> {
        if !self.active {
            return None;
        }
        let v = self.rotation.inverse() * (p - self.eye);
        if v.z > -1e-3 {
            return None;
        }
        let ndc = Vec2::new(
            v.x / (-v.z * self.tan_half_fov * self.aspect),
            v.y / (-v.z * self.tan_half_fov),
        );
        let uv = Vec2::new((ndc.x + 1.0) * 0.5, (1.0 - ndc.y) * 0.5);
        Some(self.screen_min + uv * (self.screen_max - self.screen_min))
    }

    /// Cursor → tactical XY on the voxel surface.
    #[must_use]
    pub fn pick(&self, field: &PixelWorldTerrainField, cursor: Vec2) -> Option<Vec2> {
        let (o, d) = self.ray(cursor)?;
        field.ray_hit(o, d)
    }
}

/// Clamp a map zoom into the voxel view's range.
#[inline]
#[must_use]
pub fn clamp_pixel_world_zoom(zoom: f32) -> f32 {
    if zoom.is_finite() {
        zoom.clamp(PIXEL_WORLD_ZOOM_RANGE.0, PIXEL_WORLD_ZOOM_RANGE.1)
    } else {
        PIXEL_WORLD_ZOOM_RANGE.0
    }
}

/// Q / E quarter turns (PixelWorld3D only). Runs in `MapCameraSystemSet::ApplyInput` before pan so
/// a pan in the same frame already uses the new axes.
pub fn pixel_world_rotate_input_system(
    mode: Option<Res<WorldPresentationMode>>,
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<crate::gui::InputBindings>,
    active_map_surface: Res<crate::gui::ActiveMapViewInput>,
    mut rig: ResMut<PixelWorldCameraRig>,
) {
    if !mode.is_some_and(|m| m.is_pixel_world()) {
        return;
    }
    if active_map_surface.blocks_main_world_map_camera_input() {
        return;
    }
    if keys.just_pressed(bindings.pixel_world_rotate_left) {
        rig.quarter_turn(1.0);
    }
    if keys.just_pressed(bindings.pixel_world_rotate_right) {
        rig.quarter_turn(-1.0);
    }
}

/// Middle-mouse drag orbits (left/right) and tilts (up/down) the voxel camera, smoothly (does not
/// pan — pan stays on right-drag / WASD).
pub fn pixel_world_mmb_orbit_system(
    mode: Option<Res<WorldPresentationMode>>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    input_frame: Res<crate::gui::InputFrame>,
    active_map_surface: Res<crate::gui::ActiveMapViewInput>,
    input_routing: Option<Res<crate::render::view_runtime::ViewInputRoutingState>>,
    mut rig: ResMut<PixelWorldCameraRig>,
) {
    if !mode.is_some_and(|m| m.is_pixel_world()) {
        return;
    }
    if active_map_surface.blocks_main_world_map_camera_input() {
        return;
    }
    if input_routing.is_some_and(|r| r.world_pointer_input_disabled()) {
        return;
    }
    if !mouse_btn.pressed(MouseButton::Middle) || input_frame.pointer_delta == Vec2::ZERO {
        return;
    }
    rig.drag(input_frame.pointer_delta);
}

/// Spawn the voxel camera + sun once the map RTT exists. Both start inactive / dark.
pub fn spawn_pixel_world_camera(
    mut commands: Commands,
    tex: Option<Res<SimulationMapTexture>>,
    existing: Query<(), With<PixelWorldCamera>>,
) {
    if !existing.is_empty() {
        return;
    }
    let Some(tex) = tex else {
        return;
    };
    if tex.0 == Handle::default() {
        return;
    }
    let layer = RenderLayers::layer(PIXEL_WORLD_RENDER_LAYER);
    commands.spawn((
        PixelWorldCamera,
        Camera3d::default(),
        Camera {
            order: 0,
            is_active: false,
            clear_color: ClearColorConfig::Custom(PIXEL_WORLD_SKY),
            ..default()
        },
        RenderTarget::from(tex.0.clone()),
        Projection::Perspective(PerspectiveProjection {
            fov: FOV_DEG.to_radians(),
            near: 0.5,
            far: 8000.0,
            ..default()
        }),
        Msaa::Off,
        // LGT-06: sole map/voxel policy → Bevy Tonemapping (CQ-57j).
        match crate::render::view_runtime::tonemap_hdr::PIXEL_WORLD_TONE_MAP_POLICY {
            crate::render::view_runtime::tonemap_hdr::ViewToneMapPolicy::DisplayReferredNone => {
                bevy::core_pipeline::tonemapping::Tonemapping::None
            }
            crate::render::view_runtime::tonemap_hdr::ViewToneMapPolicy::HdrFloatTarget => {
                bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface
            }
        },
        layer.clone(),
        DistanceFog {
            color: PIXEL_WORLD_SKY,
            directional_light_color: Color::NONE,
            directional_light_exponent: 8.0,
            falloff: FogFalloff::Linear { start: 400.0, end: 1200.0 },
        },
        AmbientLight {
            color: Color::srgb(0.80, 0.86, 0.95),
            brightness: 300.0,
            affects_lightmapped_meshes: true,
        },
        Transform::from_xyz(0.0, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        PixelWorldSun,
        DirectionalLight {
            color: Color::srgb(1.0, 0.96, 0.88),
            // Default exposure (EV100 9.7): ~0.7 of albedo on a sunlit top face, ambient ~0.3.
            illuminance: 2_900.0,
            // Ground, trees and buildings cast and receive: relief (valley walls, voxel steps,
            // dam) reads from its shadows, not only from face shading.
            shadow_maps_enabled: true,
            ..default()
        },
        // Cascades over the voxel view's draw range (fog ends at 1200 display units).
        bevy::light::CascadeShadowConfigBuilder {
            num_cascades: 3,
            minimum_distance: 1.0,
            first_cascade_far_bound: 120.0,
            maximum_distance: 1_200.0,
            ..default()
        }
        .build(),
        // Mid-morning sun from the south-west, low enough (40 deg) that walls and steps throw
        // visible shadows; east and north faces fall into shade.
        Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, (-35f32).to_radians(), (-40f32).to_radians(), 0.0)),
        layer,
    ));
}

/// Fit the voxel camera to the map pose and publish [`PixelWorldCameraFrame`].
///
/// PostUpdate, `SimulationViewportSyncSet::ApplyCameraProjection`, before transform propagation —
/// the same slot the top-down ortho fit uses, so both read the measured map fill of this frame.
#[allow(clippy::too_many_arguments)]
pub fn sync_pixel_world_camera(
    time: Res<Time>,
    base: Option<Res<State<BaseState>>>,
    mode: Option<Res<WorldPresentationMode>>,
    rig: Res<PixelWorldCameraRig>,
    desired: Option<Res<MapCameraDesiredRes>>,
    fill: Option<Res<SimulationMapViewport>>,
    tex: Option<Res<SimulationMapTexture>>,
    images: Res<Assets<Image>>,
    field: Res<PixelWorldTerrainField>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut frame: ResMut<PixelWorldCameraFrame>,
    mut cams: Query<
        (
            &mut Camera,
            &mut Transform,
            &mut GlobalTransform,
            &mut Projection,
            &mut RenderTarget,
            &mut DistanceFog,
        ),
        With<PixelWorldCamera>,
    >,
) {
    let Ok((mut camera, mut tf, mut global, mut proj, mut target, mut fog)) = cams.single_mut() else {
        frame.active = false;
        return;
    };
    let in_map_state = base
        .as_deref()
        .is_some_and(|s| matches!(s.get(), BaseState::Simulation | BaseState::Editor));
    let presentable = windows
        .single()
        .is_ok_and(|w| crate::render::primary_window_logical_presentable(w.width(), w.height()));
    let want = in_map_state && presentable && mode.as_deref().is_some_and(|m| m.is_pixel_world());
    let (Some(desired), Some(fill), Some(tex)) = (desired, fill, tex) else {
        camera.is_active = false;
        frame.active = false;
        return;
    };
    if !want || !field.is_ready() || tex.0 == Handle::default() {
        if camera.is_active {
            camera.is_active = false;
        }
        frame.active = false;
        return;
    }
    let bound = matches!(&*target, RenderTarget::Image(img) if img.handle == tex.0);
    if !bound {
        *target = RenderTarget::from(tex.0.clone());
    }
    camera.is_active = true;

    let tex_extent = simulation_map_texture_extent(tex.as_ref(), images.as_ref());
    let view_px = map_camera_rtt_view_pixels(fill.as_ref(), tex_extent);
    let zoom = clamp_pixel_world_zoom(desired.scale.x);
    let focus = desired.translation.truncate();

    // Yaw eases toward the rig along the short way round; pitch eases at the same rate.
    let target_yaw = rig.yaw_radians();
    let dt = time.delta_secs().clamp(0.0, 0.1);
    let first = !frame.active;
    let yaw = if first {
        target_yaw
    } else {
        let mut d = (target_yaw - frame.yaw).rem_euclid(std::f32::consts::TAU);
        if d > std::f32::consts::PI {
            d -= std::f32::consts::TAU;
        }
        let step = d * (1.0 - (-YAW_EASE * dt).exp());
        if d.abs() < 1e-3 { target_yaw } else { frame.yaw + step }
    };
    let target_pitch = rig.pitch_deg.clamp(PITCH_RANGE_DEG.0, PITCH_RANGE_DEG.1).to_radians();
    let pitch = if first || frame.pitch <= 0.0 {
        target_pitch
    } else {
        frame.pitch + (target_pitch - frame.pitch) * (1.0 - (-YAW_EASE * dt).exp())
    };

    // Focus height: mean surface over a small cross around the focus, eased.
    let view_h_tiles = view_px.y.max(1.0) / zoom;
    let spread = (view_h_tiles * 0.08).max(1.0);
    let mut sum = 0.0;
    let mut cnt = 0.0;
    for (dx, dy) in [(0.0, 0.0), (1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
        if let Some(h) = field.surface_at(focus + Vec2::new(dx, dy) * spread) {
            sum += h;
            cnt += 1.0;
        }
    }
    let sampled = if cnt > 0.0 { sum / cnt } else { field.min_h };
    let focus_height = if first {
        sampled
    } else {
        frame.focus_height + (sampled - frame.focus_height) * (1.0 - (-FOCUS_EASE * dt).exp())
    };

    let tan_half_fov = (FOV_DEG.to_radians() * 0.5).tan();
    let distance = view_h_tiles / (2.0 * tan_half_fov);
    let (_, up_axis) = pixel_world_pan_axes(yaw);
    let forward = Vec3::new(up_axis.x, 0.0, -up_axis.y);
    let focus3 = tile_to_world3(focus, focus_height);
    let eye = focus3 - forward * (distance * pitch.cos()) + Vec3::Y * (distance * pitch.sin());
    *tf = Transform::from_translation(eye).looking_at(focus3, Vec3::Y);
    // Root entity: write the global pose too, so the frame rendered is the one recorded below
    // whatever order this runs in relative to transform propagation.
    *global = GlobalTransform::from(*tf);

    if let Projection::Perspective(p) = proj.as_mut() {
        p.fov = FOV_DEG.to_radians();
        p.near = (distance * 0.01).clamp(0.2, 5.0);
        p.far = distance * 6.0 + 600.0;
    }
    // The top screen edge meets the ground ~1.35 focus distances out; haze starts past it.
    fog.falloff = FogFalloff::Linear {
        start: distance * 1.5,
        end: distance * 4.0 + 200.0,
    };

    let aspect = (tex_extent.x / tex_extent.y.max(1.0)).max(1e-3);
    let mut next = PixelWorldCameraFrame {
        active: true,
        eye,
        rotation: tf.rotation,
        tan_half_fov,
        aspect,
        screen_min: fill.min,
        screen_max: fill.max,
        px_per_tile: zoom,
        focus,
        focus_height,
        ground_min: focus,
        ground_max: focus,
        yaw,
        pitch,
    };
    // Ground footprint: the four screen-corner rays against the focus-height plane.
    let (mut lo, mut hi) = (focus, focus);
    for c in [next.screen_min, Vec2::new(next.screen_max.x, next.screen_min.y), next.screen_max, Vec2::new(next.screen_min.x, next.screen_max.y)] {
        let Some((o, d)) = next.ray(c) else {
            continue;
        };
        // Pitch exceeds half the FOV, so every corner ray descends; cap the reach anyway.
        let t = if d.y < -1e-3 { ((o.y - focus_height) / -d.y).min(distance * 8.0) } else { distance * 8.0 };
        let p = o + d * t;
        let xy = Vec2::new(p.x, -p.z);
        lo = lo.min(xy);
        hi = hi.max(xy);
    }
    next.ground_min = lo;
    next.ground_max = hi;
    *frame = next;
}

/// Voxel sun / sky for one instant of the sim day (pure; see [`voxel_day_pose`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoxelDayPose {
    pub sun_rotation: Quat,
    pub sun_illuminance: f32,
    pub sun_color: Color,
    pub ambient_brightness: f32,
    pub sky: Color,
}

/// Highest sun elevation at noon (degrees).
const NOON_ELEVATION_DEG: f32 = 62.0;
/// The light never drops below this elevation: at night it is the moon (dim, blue).
const MIN_LIGHT_ELEVATION_DEG: f32 = 12.0;
const NIGHT_SKY: Color = Color::srgb(0.07, 0.09, 0.15);

/// Sun pose from sim time on the same day clock as the tactical map
/// ([`crate::gui::tactical::sim_map_rtt::SIM_MAP_DAY_CYCLE_MICROS`] / `sim_map_daylight_factor`):
/// one day for both views. The sun rises in the east, crosses the south and sets in the west,
/// so shadows sweep across the valley; at night a low dim moon keeps the ground readable.
#[must_use]
pub fn voxel_day_pose(sim_time_micros: u64) -> VoxelDayPose {
    use crate::gui::tactical::sim_map_rtt::{sim_map_daylight_factor, SIM_MAP_DAY_CYCLE_MICROS};
    let phase = (sim_time_micros % SIM_MAP_DAY_CYCLE_MICROS) as f32 / SIM_MAP_DAY_CYCLE_MICROS as f32;
    let daylight = sim_map_daylight_factor(sim_time_micros);
    // sin(phase) > 0 is day (noon at phase 0.25). Sun elevation follows it; azimuth sweeps
    // east (sunrise, phase 0) -> south (noon) -> west (sunset, phase 0.5).
    let s = (phase * std::f32::consts::TAU).sin();
    let day = s.max(0.0);
    let elevation = (NOON_ELEVATION_DEG * day).max(MIN_LIGHT_ELEVATION_DEG);
    let azimuth = if s >= 0.0 {
        // 90 deg (east) -> 180 (south) -> 270 (west) across the day.
        90.0 + 360.0 * phase
    } else {
        // Moon: crosses the sky through the night on the same path.
        90.0 + 360.0 * (phase - 0.5)
    };
    // Bevy light shines along its -Z: yaw to the azimuth, pitch down by the elevation.
    let sun_rotation = Quat::from_euler(
        EulerRot::YXZ,
        (180.0 - azimuth).to_radians(),
        (-elevation).to_radians(),
        0.0,
    );
    // Warm and low at dawn/dusk, white at noon; dim blue moonlight at night.
    let warm = 1.0 - day;
    let sun_color = if s >= 0.0 {
        Color::srgb(1.0, 0.96 - 0.22 * warm, 0.88 - 0.38 * warm)
    } else {
        Color::srgb(0.62, 0.72, 1.0)
    };
    let sun_illuminance = if s >= 0.0 { 600.0 + 2_600.0 * day.sqrt() } else { 260.0 };
    let ambient_brightness = 70.0 + 230.0 * daylight;
    let sky = NIGHT_SKY.mix(&PIXEL_WORLD_SKY, daylight);
    VoxelDayPose { sun_rotation, sun_illuminance, sun_color, ambient_brightness, sky }
}

/// Drive the voxel sun, ambient and sky from the sim day clock (render reads sim time only).
pub fn sync_pixel_world_day_cycle(
    sim_time: Option<Res<crate::systems::sim_control::SimTimeMicros>>,
    mut sun: Query<(&mut Transform, &mut DirectionalLight), With<PixelWorldSun>>,
    mut cam: Query<(&mut Camera, &mut AmbientLight, &mut DistanceFog), With<PixelWorldCamera>>,
) {
    let pose = voxel_day_pose(sim_time.map_or(0, |t| t.0));
    for (mut tf, mut light) in &mut sun {
        tf.rotation = pose.sun_rotation;
        light.illuminance = pose.sun_illuminance;
        light.color = pose.sun_color;
    }
    for (mut camera, mut ambient, mut fog) in &mut cam {
        camera.clear_color = ClearColorConfig::Custom(pose.sky);
        ambient.brightness = pose.ambient_brightness;
        fog.color = pose.sky;
    }
}

#[cfg(test)]
mod day_cycle_tests {
    use super::*;
    use crate::gui::tactical::sim_map_rtt::SIM_MAP_DAY_CYCLE_MICROS;

    fn light_dir(p: &VoxelDayPose) -> Vec3 {
        p.sun_rotation * Vec3::NEG_Z
    }

    #[test]
    fn noon_is_high_and_bright_midnight_is_dim_moon() {
        let noon = voxel_day_pose(SIM_MAP_DAY_CYCLE_MICROS / 4);
        let night = voxel_day_pose(SIM_MAP_DAY_CYCLE_MICROS * 3 / 4);
        assert!(light_dir(&noon).y < -0.8, "noon sun shines steeply down: {:?}", light_dir(&noon));
        assert!(noon.sun_illuminance > 3.0 * night.sun_illuminance);
        assert!(noon.ambient_brightness > night.ambient_brightness);
        assert!(light_dir(&night).y < 0.0, "the moon still lights the ground from above");
    }

    #[test]
    fn sun_moves_east_to_west_through_the_day() {
        let morning = light_dir(&voxel_day_pose(SIM_MAP_DAY_CYCLE_MICROS / 16));
        let evening = light_dir(&voxel_day_pose(SIM_MAP_DAY_CYCLE_MICROS * 7 / 16));
        // Morning sun is in the east, so its light travels west (-X); evening the reverse.
        assert!(morning.x < -0.3, "morning light {morning:?}");
        assert!(evening.x > 0.3, "evening light {evening:?}");
    }
}
