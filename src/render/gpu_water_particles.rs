//! FX-WATER-PARTICLE-002 — water particle emission from [`WaterSurfaceVisualCatalog`] (D-W06–D-W09).
//!
//! Policy-only lane: builds [`WorldWaterParticleFrame`] for future WGSL upload (D-W10 A).
//! **No** terrain extract; reads catalog + [`FireParticleCameraScale`] only.

use std::collections::HashMap;

use bevy::math::Vec4;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bytemuck::{Pod, Zeroable};

use super::gpu_particles::FireParticleCameraScale;
use super::tile_world_fallback::RASTER_CHUNK_TILES;
use super::water_surface_visual::{RiverPolylineSegment, WaterSurfaceVisualCatalog};

/// Strategic zoom cutoff — particles only (shader motion stays on — D-W09 A).
pub const WATER_PARTICLE_STRATEGIC_ZOOM_ALPHA: f32 = 0.35;

/// Per-chunk caps at **tactical** zoom (§7); scaled down at operational.
pub const WATER_LAKE_GLINTS_PER_CHUNK: usize = 8;
pub const WATER_RIVER_STREAKS_PER_CHUNK: usize = 24;
pub const WATER_RIVER_FOAM_PER_CHUNK: usize = 12;
pub const WATER_OCEAN_FOAM_PER_CHUNK: usize = 16;

/// Water particle profile (D-W06–D-W08).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum WaterParticleProfile {
    LakeGlint = 0,
    RiverStreak = 1,
    RiverFoam = 2,
    OceanFoam = 3,
}

impl WaterParticleProfile {
    #[inline]
    const fn as_f32(self) -> f32 {
        self as u32 as f32
    }
}

/// Instanced water particle row (WGSL expand consumes this — W2-A).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct GpuWaterParticleInstance {
    /// World `xy`, `z` = profile ordinal, `w` = anim phase seed.
    pub world_xyz_profile: Vec4,
    /// Flow direction `xy`, `z` = world half-edge, `w` = stretch (streak) or twinkle.
    pub flow_half_twinkle: Vec4,
}

/// Expanded instanced-quad vertex row for water particle raster pass.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct GpuWaterParticleQuadVertex {
    pub world_xy_profile_phase: Vec4,
    pub uv_stretch_twinkle: Vec4,
}

/// §7 density scale per profile at current zoom band.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WaterParticleDensityScale {
    pub lake: f32,
    pub river: f32,
    pub ocean: f32,
}

impl WaterParticleDensityScale {
    #[must_use]
    pub fn from_zoom_alpha(zoom_alpha: f32) -> Self {
        let za = zoom_alpha.clamp(0.0, 1.0);
        if za < WATER_PARTICLE_STRATEGIC_ZOOM_ALPHA {
            return Self::default();
        }
        if za >= 0.65 {
            return Self {
                lake: 1.0,
                river: 1.0,
                ocean: 1.0,
            };
        }
        Self {
            lake: 0.4,
            river: 0.6,
            ocean: 0.5,
        }
    }

    #[inline]
    fn cap(base: usize, scale: f32) -> usize {
        if scale <= 0.0 {
            0
        } else {
            ((base as f32) * scale).round() as usize
        }
    }

    fn lake_cap(&self) -> usize {
        Self::cap(WATER_LAKE_GLINTS_PER_CHUNK, self.lake)
    }

    fn river_streak_cap(&self) -> usize {
        Self::cap(WATER_RIVER_STREAKS_PER_CHUNK, self.river)
    }

    fn river_foam_cap(&self) -> usize {
        Self::cap(WATER_RIVER_FOAM_PER_CHUNK, self.river)
    }

    fn ocean_foam_cap(&self) -> usize {
        Self::cap(WATER_OCEAN_FOAM_PER_CHUNK, self.ocean)
    }
}

/// Zoom band used when re-stamping tactical witness during `--test visual` proof.
pub const WATER_TACTICAL_WITNESS_ZOOM_ALPHA: f32 = 0.75;

/// Strategic + tactical particle witness snapshots (WATER-WITNESS-001 / WATER-STRATEGIC-001).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WaterVfxWitnessBands {
    pub strategic: WaterParticleWitness,
    pub tactical: WaterParticleWitness,
}

/// Re-stamp particles at strategic and tactical zoom without moving the live camera (D-W09).
#[must_use]
pub fn evaluate_water_vfx_witness_bands(
    catalog: &WaterSurfaceVisualCatalog,
    live_tactical_zoom_alpha: f32,
    time_secs: f32,
) -> WaterVfxWitnessBands {
    let mut strategic_frame = WorldWaterParticleFrame::default();
    update_world_water_particles_from_catalog(
        catalog,
        &mut strategic_frame,
        FireParticleCameraScale {
            camera_zoom: 1.0,
            zoom_alpha: WATER_PARTICLE_STRATEGIC_ZOOM_ALPHA * 0.5,
        },
        time_secs,
    );
    let tactical_zoom = live_tactical_zoom_alpha.max(WATER_TACTICAL_WITNESS_ZOOM_ALPHA);
    let mut tactical_frame = WorldWaterParticleFrame::default();
    update_world_water_particles_from_catalog(
        catalog,
        &mut tactical_frame,
        FireParticleCameraScale {
            camera_zoom: 1.0,
            zoom_alpha: tactical_zoom,
        },
        time_secs,
    );
    WaterVfxWitnessBands {
        strategic: strategic_frame.witness,
        tactical: tactical_frame.witness,
    }
}

/// WATER-STRATEGIC-001 — particles culled, shader motion flag stays on below strategic cutoff.
#[must_use]
pub fn water_strategic_001_green(bands: &WaterVfxWitnessBands) -> bool {
    bands.strategic.zoom_alpha < WATER_PARTICLE_STRATEGIC_ZOOM_ALPHA
        && bands.strategic.strategic_culled
        && bands.strategic.rows == 0
        && bands.strategic.shader_motion_always_on
        && bands.tactical.shader_motion_always_on
}

/// True when any ocean tile borders land, lake, river, or map edge (D-W08 eligibility).
#[must_use]
pub fn catalog_has_coast_ocean(catalog: &WaterSurfaceVisualCatalog) -> bool {
    catalog
        .ocean_tiles
        .iter()
        .any(|&(tx, ty)| is_coast_ocean_tile(catalog, tx, ty))
}

/// True when consecutive same-path segments turn enough for bend foam (D-W07).
#[must_use]
pub fn catalog_has_river_bend(catalog: &WaterSurfaceVisualCatalog) -> bool {
    if catalog.river_segments.len() < 2 {
        return false;
    }
    for window in catalog.river_segments.windows(2) {
        let prev = &window[0];
        let next = &window[1];
        if !river_segments_are_consecutive(prev, next) {
            continue;
        }
        let a = prev.flow_dir.normalize_or_zero();
        let b = next.flow_dir.normalize_or_zero();
        if a.length_squared() < 1e-6 || b.length_squared() < 1e-6 {
            continue;
        }
        let bend = a.dot(b).clamp(-1.0, 1.0).acos();
        if bend >= 0.2 {
            return true;
        }
    }
    false
}

/// WATER-W2-FOAM-001 — tactical coast + bend foam when catalog geometry requires it.
#[must_use]
pub fn water_w2_foam_001_green(
    catalog: &WaterSurfaceVisualCatalog,
    bands: &WaterVfxWitnessBands,
) -> bool {
    if !water_strategic_001_green(bands) {
        return false;
    }
    let coast_required = catalog_has_coast_ocean(catalog);
    let coast_ok = !coast_required || bands.tactical.coast_foam > 0;
    let bend_required = catalog_has_river_bend(catalog);
    let river_ok = !bend_required || bands.tactical.river_foam > 0;
    let parity = !catalog.w1_green()
        || bands.tactical.river_streaks > 0
        || bands.tactical.river_foam > 0;
    coast_ok && river_ok && parity
}

/// Foam and/or ocean channel proved (W2 foam or W1 ocean catalog).
#[must_use]
pub fn water_witness_foam_or_ocean_green(
    catalog: &WaterSurfaceVisualCatalog,
    tactical: &WaterParticleWitness,
) -> bool {
    catalog.w1_ocean_green()
        || tactical.river_foam > 0
        || tactical.coast_foam > 0
}

/// WATER-WITNESS-001 rollup for stage5 JSON gates.
#[must_use]
pub fn water_witness_001_green(
    catalog: &WaterSurfaceVisualCatalog,
    bands: &WaterVfxWitnessBands,
) -> bool {
    if !water_strategic_001_green(bands) {
        return false;
    }
    if catalog.w1_green() && bands.tactical.rows == 0 {
        return false;
    }
    if catalog.w1_green() && bands.tactical.river_streaks == 0 && !catalog.river_segments.is_empty() {
        return false;
    }
    water_witness_foam_or_ocean_green(catalog, &bands.tactical)
}

fn water_particle_witness_json(w: &WaterParticleWitness) -> serde_json::Value {
    serde_json::json!({
        "rows": w.rows,
        "river_streaks": w.river_streaks,
        "river_foam": w.river_foam,
        "lake_glints": w.lake_glints,
        "coast_foam": w.coast_foam,
        "zoom_alpha": w.zoom_alpha,
        "shader_motion_always_on": w.shader_motion_always_on,
        "strategic_culled": w.strategic_culled,
    })
}

/// JSON block for `stage5_full_app_live.json` / agent proofs.
#[must_use]
pub fn water_vfx_witness_json(
    catalog: &WaterSurfaceVisualCatalog,
    bands: &WaterVfxWitnessBands,
) -> serde_json::Value {
    serde_json::json!({
        "water_strategic_001_green": water_strategic_001_green(bands),
        "water_witness_001_green": water_witness_001_green(catalog, bands),
        "water_witness_foam_or_ocean_green": water_witness_foam_or_ocean_green(catalog, &bands.tactical),
        "water_w2_foam_001_green": water_w2_foam_001_green(catalog, bands),
        "catalog_has_coast_ocean": catalog_has_coast_ocean(catalog),
        "catalog_has_river_bend": catalog_has_river_bend(catalog),
        "strategic_band": water_particle_witness_json(&bands.strategic),
        "tactical_band": water_particle_witness_json(&bands.tactical),
        "strategic_zoom_cutoff": WATER_PARTICLE_STRATEGIC_ZOOM_ALPHA,
        "tactical_witness_zoom_alpha": WATER_TACTICAL_WITNESS_ZOOM_ALPHA,
    })
}

/// Witness for W2 / stage5 JSON (FX-WATER-PARTICLE-003 fields subset on W2-B).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WaterParticleWitness {
    pub rows: usize,
    pub river_streaks: usize,
    pub river_foam: usize,
    pub lake_glints: usize,
    pub coast_foam: usize,
    pub zoom_alpha: f32,
    pub shader_motion_always_on: bool,
    pub strategic_culled: bool,
}

/// CPU-side water particle snapshot (single upload spine — D-W10 A policy half).
#[derive(Resource, Clone, Debug, Default, ExtractResource)]
pub struct WorldWaterParticleFrame {
    pub anim_time_secs: f32,
    pub instances: Vec<GpuWaterParticleInstance>,
    pub witness: WaterParticleWitness,
}

#[inline]
fn chunk_key(tx: u32, ty: u32) -> (u32, u32) {
    (tx / RASTER_CHUNK_TILES, ty / RASTER_CHUNK_TILES)
}

#[inline]
fn hash_u32(x: u32, y: u32, salt: u32) -> u32 {
    x.wrapping_mul(73856093)
        .wrapping_add(y.wrapping_mul(19349663))
        .wrapping_add(salt)
}

#[inline]
fn water_particle_half_world(profile: WaterParticleProfile, zoom_alpha: f32, camera_zoom: f32) -> f32 {
    let z = camera_zoom.max(0.06);
    let za = zoom_alpha.clamp(0.0, 1.0);
    let screen_half = match profile {
        WaterParticleProfile::LakeGlint => 0.5 + za * 1.0,
        WaterParticleProfile::RiverStreak => 0.6 + za * 1.2,
        WaterParticleProfile::RiverFoam => 0.45 + za * 0.8,
        WaterParticleProfile::OceanFoam => 0.55 + za * 0.9,
    };
    (screen_half / z).clamp(0.012, 1.2)
}

fn push_instance(
    out: &mut Vec<GpuWaterParticleInstance>,
    world: Vec2,
    profile: WaterParticleProfile,
    flow: Vec2,
    half: f32,
    twinkle: f32,
    phase: f32,
) {
    out.push(GpuWaterParticleInstance {
        world_xyz_profile: Vec4::new(world.x, world.y, profile.as_f32(), phase),
        flow_half_twinkle: Vec4::new(flow.x, flow.y, half, twinkle),
    });
}

fn emit_river_streaks_for_segment(
    out: &mut Vec<GpuWaterParticleInstance>,
    seg: &RiverPolylineSegment,
    slots: usize,
    cam: FireParticleCameraScale,
    salt: u32,
) {
    if slots == 0 {
        return;
    }
    let delta = seg.end - seg.start;
    let len = delta.length().max(0.25);
    let flow = if seg.flow_dir.length_squared() > 1e-6 {
        seg.flow_dir.normalize()
    } else {
        delta / len
    };
    for i in 0..slots {
        let t = (i as f32 + 0.5) / slots as f32;
        let along = seg.start + delta * t;
        let cross = Vec2::new(-flow.y, flow.x);
        let jitter = (hash_u32(seg.path_id, i as u32, salt) & 0xff) as f32 / 255.0 - 0.5;
        let world = along + cross * jitter * seg.half_width;
        let half = water_particle_half_world(WaterParticleProfile::RiverStreak, cam.zoom_alpha, cam.camera_zoom);
        push_instance(
            out,
            world,
            WaterParticleProfile::RiverStreak,
            flow,
            half,
            3.0,
            t,
        );
    }
}

#[inline]
fn river_segments_are_consecutive(a: &RiverPolylineSegment, b: &RiverPolylineSegment) -> bool {
    a.path_id == b.path_id && (a.end - b.start).length_squared() <= 1.01
}

fn emit_river_foam_at_bends(
    out: &mut Vec<GpuWaterParticleInstance>,
    segments: &[RiverPolylineSegment],
    chunk_foam: &mut HashMap<(u32, u32), usize>,
    density: &WaterParticleDensityScale,
    cam: FireParticleCameraScale,
) -> usize {
    if segments.len() < 2 {
        return 0;
    }
    let mut emitted = 0usize;
    for window in segments.windows(2) {
        let prev = &window[0];
        let next = &window[1];
        if !river_segments_are_consecutive(prev, next) {
            continue;
        }
        let a = prev.flow_dir.normalize_or_zero();
        let b = next.flow_dir.normalize_or_zero();
        if a.length_squared() < 1e-6 || b.length_squared() < 1e-6 {
            continue;
        }
        let bend = a.dot(b).clamp(-1.0, 1.0).acos();
        if bend < 0.2 {
            continue;
        }
        let ck = chunk_key(next.start.x as u32, next.start.y as u32);
        let used = chunk_foam.entry(ck).or_insert(0);
        let cap = density.river_foam_cap();
        if *used >= cap {
            continue;
        }
        let world = next.start;
        let half = water_particle_half_world(WaterParticleProfile::RiverFoam, cam.zoom_alpha, cam.camera_zoom);
        push_instance(
            out,
            world,
            WaterParticleProfile::RiverFoam,
            b,
            half,
            1.0,
            bend,
        );
        *used += 1;
        emitted += 1;
    }
    emitted
}

fn is_coast_ocean_tile(
    catalog: &WaterSurfaceVisualCatalog,
    tx: u32,
    ty: u32,
) -> bool {
    if !catalog.ocean_tiles.contains(&(tx, ty)) {
        return false;
    }
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let nx = tx as i32 + dx;
        let ny = ty as i32 + dy;
        if nx < 0 || ny < 0 {
            return true;
        }
        let ux = nx as u32;
        let uy = ny as u32;
        if catalog.lake_tiles.contains(&(ux, uy)) || catalog.river_tiles.contains(&(ux, uy)) {
            return true;
        }
        if !catalog.ocean_tiles.contains(&(ux, uy)) {
            return true;
        }
    }
    false
}

fn emit_coast_foam_at(
    out: &mut Vec<GpuWaterParticleInstance>,
    tx: u32,
    ty: u32,
    cam: FireParticleCameraScale,
) {
    let half = water_particle_half_world(WaterParticleProfile::OceanFoam, cam.zoom_alpha, cam.camera_zoom);
    push_instance(
        out,
        Vec2::new(tx as f32 + 0.5, ty as f32 + 0.5),
        WaterParticleProfile::OceanFoam,
        Vec2::X,
        half,
        0.5,
        0.0,
    );
}

fn emit_lake_glint_at(
    out: &mut Vec<GpuWaterParticleInstance>,
    tx: u32,
    ty: u32,
    cam: FireParticleCameraScale,
) -> bool {
    let h = hash_u32(tx, ty, 17);
    if h % 5 != 0 {
        return false;
    }
    let half = water_particle_half_world(WaterParticleProfile::LakeGlint, cam.zoom_alpha, cam.camera_zoom);
    push_instance(
        out,
        Vec2::new(tx as f32 + 0.5, ty as f32 + 0.5),
        WaterParticleProfile::LakeGlint,
        Vec2::ZERO,
        half,
        (h & 0xff) as f32 / 255.0,
        0.0,
    );
    true
}

/// Build particle rows from catalog + zoom band (§7).
pub fn update_world_water_particles_from_catalog(
    catalog: &WaterSurfaceVisualCatalog,
    frame: &mut WorldWaterParticleFrame,
    cam: FireParticleCameraScale,
    time_secs: f32,
) {
    frame.anim_time_secs = time_secs;
    frame.instances.clear();

    let density = WaterParticleDensityScale::from_zoom_alpha(cam.zoom_alpha);
    let strategic = cam.zoom_alpha < WATER_PARTICLE_STRATEGIC_ZOOM_ALPHA;

    let has_water_motion =
        catalog.w1_green() || catalog.w1_ocean_green() || !catalog.lake_tiles.is_empty();
    if strategic || !has_water_motion {
        frame.witness = WaterParticleWitness {
            zoom_alpha: cam.zoom_alpha,
            shader_motion_always_on: true,
            strategic_culled: strategic,
            ..Default::default()
        };
        return;
    }

    let mut chunk_river_streaks: HashMap<(u32, u32), usize> = HashMap::new();
    let mut chunk_river_foam: HashMap<(u32, u32), usize> = HashMap::new();
    let mut chunk_lake: HashMap<(u32, u32), usize> = HashMap::new();
    let mut chunk_ocean: HashMap<(u32, u32), usize> = HashMap::new();

    let mut river_streaks = 0usize;
    let mut river_foam = 0usize;
    let mut lake_glints = 0usize;
    let mut coast_foam = 0usize;

    for seg in &catalog.river_segments {
        let ck = chunk_key(seg.start.x as u32, seg.start.y as u32);
        let used = chunk_river_streaks.entry(ck).or_insert(0);
        let cap = density.river_streak_cap();
        if *used >= cap {
            continue;
        }
        let slots = (cap - *used).min(3);
        emit_river_streaks_for_segment(&mut frame.instances, seg, slots, cam, *used as u32);
        *used += slots;
        river_streaks += slots;
    }

    river_foam += emit_river_foam_at_bends(
        &mut frame.instances,
        &catalog.river_segments,
        &mut chunk_river_foam,
        &density,
        cam,
    );

    for &(tx, ty) in &catalog.lake_tiles {
        let ck = chunk_key(tx, ty);
        let used = chunk_lake.entry(ck).or_insert(0);
        let cap = density.lake_cap();
        if *used >= cap {
            continue;
        }
        if emit_lake_glint_at(&mut frame.instances, tx, ty, cam) {
            *used += 1;
            lake_glints += 1;
        }
    }

    for &(tx, ty) in &catalog.ocean_tiles {
        if !is_coast_ocean_tile(catalog, tx, ty) {
            continue;
        }
        let ck = chunk_key(tx, ty);
        let used = chunk_ocean.entry(ck).or_insert(0);
        let cap = density.ocean_foam_cap();
        if *used >= cap {
            continue;
        }
        emit_coast_foam_at(&mut frame.instances, tx, ty, cam);
        *used += 1;
        coast_foam += 1;
    }

    if coast_foam == 0 && catalog_has_coast_ocean(catalog) {
        if let Some(&(tx, ty)) = catalog
            .ocean_tiles
            .iter()
            .find(|&&(tx, ty)| is_coast_ocean_tile(catalog, tx, ty))
        {
            emit_coast_foam_at(&mut frame.instances, tx, ty, cam);
            coast_foam = 1;
        }
    }

    if river_foam == 0 && catalog_has_river_bend(catalog) {
        river_foam += emit_river_foam_at_bends(
            &mut frame.instances,
            &catalog.river_segments,
            &mut chunk_river_foam,
            &WaterParticleDensityScale {
                lake: 1.0,
                river: 1.0,
                ocean: 1.0,
            },
            cam,
        );
    }

    frame.witness = WaterParticleWitness {
        rows: frame.instances.len(),
        river_streaks,
        river_foam,
        lake_glints,
        coast_foam,
        zoom_alpha: cam.zoom_alpha,
        shader_motion_always_on: true,
        strategic_culled: false,
    };
}

pub fn emit_world_water_particles_from_catalog(
    time: Res<Time>,
    catalog: Option<Res<WaterSurfaceVisualCatalog>>,
    cam: Res<FireParticleCameraScale>,
    mut frame: ResMut<WorldWaterParticleFrame>,
) {
    let Some(catalog) = catalog else {
        frame.instances.clear();
        frame.witness = WaterParticleWitness {
            shader_motion_always_on: true,
            zoom_alpha: cam.zoom_alpha,
            ..Default::default()
        };
        return;
    };
    update_world_water_particles_from_catalog(catalog.as_ref(), frame.as_mut(), *cam, time.elapsed_secs());
}

pub struct GpuWaterParticlesPlugin;

impl Plugin for GpuWaterParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldWaterParticleFrame>()
            .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
                WorldWaterParticleFrame,
            >::default())
            .add_systems(
                Update,
                emit_world_water_particles_from_catalog
                    .after(super::gpu_particles::sync_fire_particle_camera_scale)
                    .in_set(super::WaterSurfaceVisualSet),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::generation::hydrology::{compute_hydrology_rect, HydrologyParams};
    use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

    #[test]
    fn water_strategic_001_dual_band_witness() {
        let mut catalog = WaterSurfaceVisualCatalog::default();
        catalog.grid_width = 16;
        catalog.grid_height = 16;
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(8.0, 0.0),
            flow_dir: Vec2::X,
            half_width: 0.42,
        });
        catalog.river_tiles.insert((4, 0));
        catalog.ocean_tiles.insert((5, 5));
        catalog.lake_tiles.insert((0, 0));

        let bands = evaluate_water_vfx_witness_bands(&catalog, 0.8, 0.0);
        assert!(water_strategic_001_green(&bands), "D-W09 strategic cull + shader on");
        assert!(bands.tactical.rows > 0);
        assert!(water_witness_foam_or_ocean_green(&catalog, &bands.tactical));
        assert!(water_witness_001_green(&catalog, &bands));
    }

    #[test]
    fn strategic_zoom_zeroes_water_particle_rows() {
        let mut catalog = WaterSurfaceVisualCatalog::default();
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(1.0, 1.0),
            end: Vec2::new(4.0, 1.0),
            flow_dir: Vec2::X,
            half_width: 0.4,
        });
        catalog.river_tiles.insert((2, 1));
        let mut frame = WorldWaterParticleFrame::default();
        update_world_water_particles_from_catalog(
            &catalog,
            &mut frame,
            FireParticleCameraScale {
                camera_zoom: 1.0,
                zoom_alpha: 0.2,
            },
            0.0,
        );
        assert!(frame.instances.is_empty());
        assert!(frame.witness.strategic_culled);
        assert!(frame.witness.shader_motion_always_on);
    }

    /// P2-VFX-WITNESS-001 W-2 / P2-WATER-WITNESS-002 — tactical zoom witness gates.
    #[test]
    fn p2_tactical_zoom_alpha_08_water_particle_witness_gates() {
        let mut catalog = WaterSurfaceVisualCatalog::default();
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(8.0, 0.0),
            flow_dir: Vec2::X,
            half_width: 0.42,
        });
        catalog.river_tiles.insert((4, 0));
        let mut frame = WorldWaterParticleFrame::default();
        update_world_water_particles_from_catalog(
            &catalog,
            &mut frame,
            FireParticleCameraScale {
                camera_zoom: 1.0,
                zoom_alpha: 0.8,
            },
            1.0,
        );
        assert!(frame.witness.rows > 0, "water_particle_rows at tactical zoom");
        assert!(frame.witness.river_streaks > 0, "river streaks when segments present");
        assert!(!frame.witness.strategic_culled);
        assert!(frame.witness.shader_motion_always_on);
        assert!((frame.witness.zoom_alpha - 0.8).abs() < 1e-4);
    }

    #[test]
    fn tactical_zoom_emits_river_streaks() {
        let mut catalog = WaterSurfaceVisualCatalog::default();
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(8.0, 0.0),
            flow_dir: Vec2::X,
            half_width: 0.42,
        });
        catalog.river_tiles.insert((4, 0));
        let mut frame = WorldWaterParticleFrame::default();
        update_world_water_particles_from_catalog(
            &catalog,
            &mut frame,
            FireParticleCameraScale {
                camera_zoom: 1.0,
                zoom_alpha: 0.8,
            },
            1.0,
        );
        assert!(frame.witness.river_streaks > 0);
        assert!(frame.instances.iter().any(|i| i.world_xyz_profile.z == WaterParticleProfile::RiverStreak.as_f32()));
    }

    #[test]
    fn density_scale_operational_halves_lake_cap() {
        let tactical = WaterParticleDensityScale::from_zoom_alpha(0.9);
        let operational = WaterParticleDensityScale::from_zoom_alpha(0.5);
        assert!(operational.lake_cap() < tactical.lake_cap());
        assert!(operational.river_streak_cap() < tactical.river_streak_cap());
    }

    /// WATER-W2-FOAM-001 / D-W07 — bend foam at path junctions (not per-chunk segment subsets).
    #[test]
    fn water_w2_foam_001_river_bend_emits_foam() {
        let mut catalog = WaterSurfaceVisualCatalog::default();
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(4.0, 0.0),
            flow_dir: Vec2::X,
            half_width: 0.42,
        });
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(4.0, 0.0),
            end: Vec2::new(4.0, 4.0),
            flow_dir: Vec2::Y,
            half_width: 0.42,
        });
        catalog.river_tiles.insert((2, 0));
        let mut frame = WorldWaterParticleFrame::default();
        update_world_water_particles_from_catalog(
            &catalog,
            &mut frame,
            FireParticleCameraScale {
                camera_zoom: 1.0,
                zoom_alpha: 0.8,
            },
            0.0,
        );
        assert!(
            frame.witness.river_foam > 0,
            "90° bend should emit RiverFoam (D-W07)"
        );
        assert!(frame.instances.iter().any(|i| {
            i.world_xyz_profile.z == WaterParticleProfile::RiverFoam.as_f32()
        }));
    }

    /// WATER-W2-FOAM-001 / D-W08 — coast foam only on ocean tiles adjacent to land.
    #[test]
    fn water_w2_foam_001_green_on_dem_ocean_grid() {
        use crate::terrain::generation::hydrology::HydrologyResult;
        use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

        let w = 8u32;
        let h = 8u32;
        let n = (w * h) as usize;
        let hydro = HydrologyResult {
            rivers: Vec::new(),
            lakes: Vec::new(),
            accumulation: vec![0.0; n],
            river_mask: vec![false; n],
            lake_mask: vec![false; n],
            filled_dem: vec![0.05; n],
        };
        let mut params = WorldGenParams::default();
        params.width = w;
        params.height = h;
        let catalog = WaterSurfaceVisualCatalog::from_hydrology(&hydro, &params);
        assert!(catalog_has_coast_ocean(&catalog));
        let bands = evaluate_water_vfx_witness_bands(&catalog, 0.8, 0.0);
        assert!(water_w2_foam_001_green(&catalog, &bands));
        assert!(bands.tactical.coast_foam > 0);
    }

    #[test]
    fn water_w2_foam_001_coast_foam_at_ocean_shore() {
        let mut catalog = WaterSurfaceVisualCatalog::default();
        catalog.grid_width = 16;
        catalog.grid_height = 16;
        catalog.river_tiles.insert((0, 0));
        catalog.ocean_tiles.insert((5, 5));
        let mut frame = WorldWaterParticleFrame::default();
        update_world_water_particles_from_catalog(
            &catalog,
            &mut frame,
            FireParticleCameraScale {
                camera_zoom: 1.0,
                zoom_alpha: 0.8,
            },
            0.0,
        );
        assert!(
            frame.witness.coast_foam > 0,
            "ocean tile beside dry land should emit coast foam (D-W08)"
        );
        assert!(frame.instances.iter().any(|i| {
            i.world_xyz_profile.z == WaterParticleProfile::OceanFoam.as_f32()
        }));
    }

    #[test]
    fn catalog_hydro_builds_w1_green_and_particles() {
        let w = 32u32;
        let h = 32u32;
        let mut dem = vec![0.55f32; (w * h) as usize];
        for x in 4..28 {
            dem[x as usize] = 0.35;
            dem[w as usize + x as usize] = 0.34;
        }
        let hydro = compute_hydrology_rect(w, h, &dem, &HydrologyParams::default(), 4, None);
        let mut params = WorldGenParams::default();
        params.width = w;
        params.height = h;
        let catalog = WaterSurfaceVisualCatalog::from_hydrology(&hydro, &params);
        assert!(catalog.w1_green());
        let mut frame = WorldWaterParticleFrame::default();
        update_world_water_particles_from_catalog(
            &catalog,
            &mut frame,
            FireParticleCameraScale {
                camera_zoom: 1.0,
                zoom_alpha: 0.75,
            },
            0.0,
        );
        assert!(frame.witness.rows > 0 || catalog.river_segments.is_empty());
    }
}
