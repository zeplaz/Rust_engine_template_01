//! PixelWorld3D content layers over the voxel terrain: trees, construction sites, the build ghost
//! and fronts. Each reads an existing representation and nothing else:
//!
//! - trees: [`VegetationExtractFrame`] rows (per-cell [`crate::systems::ecology::ChunkCellStand`]
//!   codes — live stage, burning, burnt snags, regrowth, water) with the flat layer's own site
//!   sampler and zoom ladder ([`VegLod`]); one mesh per sim chunk in view, rebuilt when that
//!   chunk's cells change;
//! - sites: `PlannedSite` / `ConstructionSite` / `SiteFootprint` — footprint columns by archetype,
//!   height by phase and footprint size ([`derive_massing_floors`]);
//! - ghost: [`ConstructionVisualRequests`] footprint tiles + **RO** `massing_columns` /
//!   `massing_instances` (same requests the top-down overlay paints; sole writer = `voxel_massing`;
//!   **RND-09** instance batch when count ≥ threshold);
//! - fronts: [`FrontOverlaySources`] geometry (the top-down layer's builders) draped on the surface.

use std::collections::HashMap;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use super::camera::PixelWorldCameraFrame;
use super::field::PixelWorldTerrainField;
use super::terrain::MeshBuf;
use super::PIXEL_WORLD_RENDER_LAYER;
use crate::construction::{derive_massing_floors, ConstructionVisualRequests, FootprintTileColorKind};
use crate::gui::WorldPresentationMode;
use crate::render::extraction::{VegExtractRow, VegetationExtractFrame};
use crate::render::world_map_front_mesh::FrontOverlaySources;
use crate::render::vegetation_species::{Maturity, VegSpeciesTable};
use crate::render::world_map_vegetation_mesh::{sample_row_instances, Cover, Variation, VegInstance, VegLod};
use crate::combat::{BallisticPresentFrame, Formation, UnitKind};
use crate::strategic::{ConstructionSite, PlannedSite, SiteArchetype, SiteConstructionPhase, SiteFootprint};
use crate::terrain::generation::ChunkCellMatrix;
use crate::terrain::world_scale_contract::{CHUNK_TILES_SIM, TILE_METRES_GAMEPLAY};

#[inline]
fn p3(x: f32, y: f32, h: f32) -> Vec3 {
    Vec3::new(x, h, -y)
}

fn mix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn shade(c: [f32; 3], k: f32, a: f32) -> [f32; 4] {
    [c[0] * k, c[1] * k, c[2] * k, a]
}

/// Box from a tactical-XY rectangle and a height span (no bottom face).
fn column(buf: &mut MeshBuf, x0: f32, y0: f32, x1: f32, y1: f32, lo: f32, hi: f32, top: [f32; 4], side: [f32; 4]) {
    buf.open_box(p3(x0, y0, lo).min(p3(x1, y1, hi)), p3(x0, y0, lo).max(p3(x1, y1, hi)), top, side);
}

fn layer() -> RenderLayers {
    RenderLayers::layer(PIXEL_WORLD_RENDER_LAYER)
}

fn pixel_world_on(mode: &Option<Res<WorldPresentationMode>>, frame: &PixelWorldCameraFrame, field: &PixelWorldTerrainField) -> bool {
    mode.as_deref().is_some_and(|m| m.is_pixel_world()) && frame.active && field.is_ready()
}

// ---------------------------------------------------------------------------------------------
// Trees
// ---------------------------------------------------------------------------------------------

#[derive(Component)]
pub struct PixelWorldTreeChunk;

/// One live chunk: stable entity + mesh handle, content key of what the mesh currently shows.
pub(crate) struct TreeChunkSlot {
    entity: Entity,
    key: u64,
    mesh: Option<Handle<Mesh>>,
}

#[derive(Default)]
pub(crate) struct TreeLayerState {
    material: Option<Handle<StandardMaterial>>,
    chunks: HashMap<IVec2, TreeChunkSlot>,
    /// Reused site buffer for chunk rebuilds.
    scratch: Vec<VegInstance>,
}

/// Chunk meshes rebuilt per frame (nearest first); the rest wait a frame.
const TREE_CHUNKS_PER_FRAME: usize = 48;

/// Crown / trunk sizes scale with the sampling step so a coarse band still reads as forest.
fn step_scale(step: u32) -> f32 {
    match step {
        0 | 1 => 1.0,
        2 => 1.7,
        4 => 3.0,
        _ => 5.5,
    }
}

const TRUNK_COL: [f32; 3] = [0.30, 0.21, 0.13];
const CHAR_COL: [f32; 3] = [0.11, 0.09, 0.08];
const BLEACH_COL: [f32; 3] = [0.58, 0.54, 0.48];
const GRASS_COL: [f32; 3] = [0.50, 0.64, 0.26];
const SAPLING_COL: [f32; 3] = [0.45, 0.78, 0.28];

/// Per-plant placement: ground point, base height and the instance tint (shared by every box of one plant).
struct Plant {
    at: Vec2,
    base: f32,
    tint: f32,
}

impl Plant {
    /// Box centred `(ox, oy)` off the plant point, `w` wide in x and `d` deep in y, spanning `lo..hi` above ground.
    #[allow(clippy::too_many_arguments)]
    fn bx(&self, buf: &mut MeshBuf, (ox, oy): (f32, f32), w: f32, d: f32, lo: f32, hi: f32, c: [f32; 3], crown: bool) {
        let (top, side) = if crown { (1.08, 0.86) } else { (1.0, 0.8) };
        let (cx, cy) = (self.at.x + ox, self.at.y + oy);
        let (hx, hy) = (w * 0.5, d * 0.5);
        column(
            buf,
            cx - hx,
            cy - hy,
            cx + hx,
            cy + hy,
            self.base + lo,
            self.base + hi,
            shade(c, self.tint * top, 1.0),
            shade(c, self.tint * side, 1.0),
        );
    }
}

/// One plant as stacked voxel boxes, every stage always drawn at every band (`k` = zoom-band scale, so a coarse
/// band keeps a trunk and reads as a tree, not a floating cube). Live trees take tiers / trunk / colours from the
/// species table ([`VegSpeciesTable::resolve_shape`]); every stage gets a small deterministic per-instance
/// variation from `inst.rand` ([`Variation`]: size, lean, tint).
fn push_tree(buf: &mut MeshBuf, table: &VegSpeciesTable, inst: &VegInstance, base: f32, k: f32) {
    let v = Variation::of(inst.rand);
    let p = Plant { at: inst.pos, base, tint: 1.0 + v.tint };
    let kj = k * v.size;
    let lean = |sh_w: f32, t: f32| (v.dx * sh_w * kj * t, v.dy * sh_w * kj * t);
    // Fire states use the species' mature envelope; `hs` is its height against the reference oak (1.6).
    let fire_shape = || table.resolve_shape(inst.species, Maturity::Old, inst.rand);
    match inst.cover {
        Cover::OldGrowth | Cover::YoungForest => {
            let sh = table.resolve_shape(inst.species, inst.cover.maturity(), inst.rand);
            let lift = sh.trunk_frac.max(0.15) * sh.height * kj;
            p.bx(buf, (0.0, 0.0), sh.trunk_width * kj, sh.trunk_width * kj, 0.0, lift * 1.3, TRUNK_COL, false);
            let n = sh.tiers().len().max(1) as f32;
            for (i, t) in sh.tiers().iter().enumerate() {
                let off = lean(sh.crown_width, (i + 1) as f32 / n);
                p.bx(buf, off, t.width * kj, t.width * kj, lift + t.lo * kj, lift + t.hi * kj, sh.tier_color(i), true);
            }
        }
        Cover::Shrub => {
            // Low round-ish clump: three stacked boxes, narrowing and brightening upward.
            let sh = table.resolve_shape(inst.species, Maturity::Shrub, inst.rand);
            let (w, h) = (sh.crown_width * kj, sh.height * kj);
            let bands = [(1.0, 0.0, 0.45, 1.0), (0.75, 0.3, 0.8, 1.06), (0.45, 0.6, 1.0, 1.12)];
            for (i, (bw, lo, hi, lit)) in bands.into_iter().enumerate() {
                let c = sh.crown.map(|x| (x * lit).min(1.0));
                p.bx(buf, lean(sh.crown_width, i as f32 * 0.5), w * bw, w * bw, h * lo, h * hi, c, true);
            }
        }
        Cover::Grass => {
            // Low tuft: two thin flat boxes crossed.
            p.bx(buf, (v.dx * kj, v.dy * kj), 0.26 * kj, 0.05 * kj, 0.0, 0.14 * kj, GRASS_COL, true);
            p.bx(buf, (v.dx * kj, v.dy * kj), 0.05 * kj, 0.22 * kj, 0.0, 0.18 * kj, GRASS_COL, true);
        }
        Cover::Burning => {
            let sh = fire_shape();
            let hs = sh.height / 1.6;
            let flick = if inst.burn_phase % 2 == 0 { 1.0 } else { 0.85 };
            let tw = sh.trunk_width * kj;
            p.bx(buf, (0.0, 0.0), tw, tw, 0.0, 0.4 * kj * hs, [0.12, 0.09, 0.07], false);
            let off = lean(sh.crown_width, 0.6);
            let w = 0.9 * sh.crown_width * kj;
            p.bx(buf, off, w, w, 0.3 * kj * hs, (0.3 + 0.85 * flick) * kj * hs, [1.0, 0.42, 0.08], true);
            let w2 = 0.5 * sh.crown_width * kj;
            p.bx(buf, off, w2, w2, (0.3 + 0.85 * flick) * kj * hs, (0.3 + 1.2 * flick) * kj * hs, [1.0, 0.82, 0.30], true);
        }
        Cover::Burnt => {
            // Charred snag: trunk plus two stubs.
            let sh = fire_shape();
            let hs = sh.height / 1.6;
            let tw = 0.9 * sh.trunk_width * kj;
            p.bx(buf, (0.0, 0.0), tw, tw, 0.0, 0.95 * kj * hs, CHAR_COL, false);
            let arm = 0.64 * sh.crown_width * kj;
            let th = 0.1 * kj;
            p.bx(buf, (0.0, 0.0), arm, th, 0.6 * kj * hs, 0.68 * kj * hs, CHAR_COL, false);
            p.bx(buf, (0.0, 0.0), th, 0.7 * arm, 0.42 * kj * hs, 0.5 * kj * hs, CHAR_COL, false);
        }
        Cover::Dead => {
            // Bleached standing dead (vs charred Burnt): grey trunk with bare arms.
            let sh = fire_shape();
            let hs = sh.height / 1.6;
            let tw = 0.85 * sh.trunk_width * kj;
            p.bx(buf, (0.0, 0.0), tw, tw, 0.0, 0.9 * kj * hs, BLEACH_COL, false);
            let arm = 0.7 * sh.crown_width * kj;
            let th = 0.07 * kj;
            p.bx(buf, (0.0, 0.0), arm, th, 0.62 * kj * hs, 0.68 * kj * hs, BLEACH_COL, false);
            p.bx(buf, (0.0, 0.0), th, 0.7 * arm, 0.46 * kj * hs, 0.52 * kj * hs, BLEACH_COL, false);
        }
        Cover::Regrowth => {
            // Small bright sapling: thin stem, light crown.
            p.bx(buf, (0.0, 0.0), 0.05 * kj, 0.05 * kj, 0.0, 0.16 * kj, [0.36, 0.28, 0.16], false);
            p.bx(buf, lean(0.3, 0.5), 0.3 * kj, 0.3 * kj, 0.1 * kj, 0.3 * kj, SAPLING_COL, true);
        }
    }
}

/// Voxel trees for one extract row (one sim chunk). Sites come from the same sampler as the flat layer
/// ([`sample_row_instances`]), so a plant keeps its tile, species and rand across the view toggle.
pub(crate) fn build_tree_chunk(
    row: &VegExtractRow,
    chunk: UVec2,
    lod: VegLod,
    field: &PixelWorldTerrainField,
    table: &VegSpeciesTable,
    scratch: &mut Vec<VegInstance>,
) -> MeshBuf {
    let mut buf = MeshBuf::default();
    // Prefer per-cell stand codes when ready; otherwise sample from row density so trees
    // appear as soon as extract + terrain field are up (not only after stand.render lands).
    let step = match lod {
        VegLod::Wash => 8,
        other => other.step_tiles().max(1),
    };
    let k = step_scale(step);
    scratch.clear();
    sample_row_instances(row, chunk, step, scratch);
    for inst in scratch.iter() {
        let Some(i) = field.index(inst.pos.x.floor() as i32, inst.pos.y.floor() as i32) else {
            continue;
        };
        if field.water[i].is_finite() {
            continue;
        }
        push_tree(&mut buf, table, inst, field.ground[i], k);
    }
    buf
}

fn tree_chunk_key(row: &VegExtractRow, lod: VegLod, field_rev: u64) -> u64 {
    let burn = if row.burn_active { u64::from(row.frame_index / 3) + 1 } else { 0 };
    mix64(
        u64::from(row.cells_rev)
            ^ u64::from(row.species_rev).wrapping_mul(0xD6E8_FEB8_6659_FD93)
            ^ (lod as u64) << 32
            ^ burn << 40
            ^ field_rev.wrapping_mul(0x9E37_79B9)
            ^ ((row.modifiers.mean_density.clamp(0.0, 1.0) * 16.0) as u64) << 56,
    )
}

#[allow(clippy::too_many_arguments)]
/// Voxel camera domain: the frame plus whether layers may cull against it.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct VoxelCamView<'w> {
    frame: Res<'w, PixelWorldCameraFrame>,
    culling: Res<'w, super::PixelWorldCulling>,
}

pub(crate) fn sync_pixel_world_trees(
    mut commands: Commands,
    mode: Option<Res<WorldPresentationMode>>,
    view: VoxelCamView,
    field: Res<PixelWorldTerrainField>,
    veg: Option<Res<VegetationExtractFrame>>,
    table: Res<VegSpeciesTable>,
    matrices: Query<&ChunkCellMatrix>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: Local<TreeLayerState>,
) {
    let cam = &*view.frame;
    if !pixel_world_on(&mode, cam, &field) {
        return;
    }
    let Some(veg) = veg else {
        return;
    };
    if table.is_changed() {
        // New species table: every chunk mesh is stale.
        for (_, slot) in state.chunks.drain() {
            commands.entity(slot.entity).despawn();
        }
    }
    let chunk = matrices
        .iter()
        .next()
        .map(|m| m.size)
        .filter(|s| s.x > 0 && s.y > 0)
        .unwrap_or(UVec2::splat(CHUNK_TILES_SIM));
    let lod = VegLod::for_px_per_tile(cam.px_per_tile);
    let material = state
        .material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.9,
                reflectance: 0.05,
                ..default()
            })
        })
        .clone();

    let csize = Vec2::new(chunk.x as f32, chunk.y as f32);
    // Chunks overlapping the ground the screen sees (plus a two-tile margin for crowns on the edge).
    let (lo, hi) = (cam.ground_min - Vec2::splat(2.0), cam.ground_max + Vec2::splat(2.0));
    let mut wanted: Vec<(f32, &VegExtractRow)> = veg
        .rows
        .iter()
        .filter_map(|r| {
            let c0 = r.coord.as_vec2() * csize;
            let c1 = c0 + csize;
            // Culling off (default, `PixelWorldCulling`): every chunk is kept, nearest first.
            let overlaps = !view.culling.trees
                || (c1.x >= lo.x && c0.x <= hi.x && c1.y >= lo.y && c0.y <= hi.y);
            let d = (c0 + csize * 0.5).distance(cam.focus);
            overlaps.then_some((d, r))
        })
        .collect();
    wanted.sort_by(|a, b| a.0.total_cmp(&b.0));

    let keep: std::collections::HashSet<IVec2> = wanted.iter().map(|(_, r)| r.coord).collect();
    state.chunks.retain(|coord, slot| {
        let k = keep.contains(coord);
        if !k {
            commands.entity(slot.entity).despawn();
        }
        k
    });

    let mut budget = TREE_CHUNKS_PER_FRAME;
    for (_, row) in wanted {
        let key = tree_chunk_key(row, lod, field.revision);
        if state.chunks.get(&row.coord).is_some_and(|s| s.key == key) {
            continue;
        }
        if budget == 0 {
            break;
        }
        budget -= 1;
        let buf = build_tree_chunk(row, chunk, lod, field.as_ref(), &table, &mut state.scratch);
        let prev = state.chunks.remove(&row.coord);
        let slot = publish_tree_chunk(&mut commands, &mut meshes, &material, prev, buf, key);
        state.chunks.insert(row.coord, slot);
    }
}

/// Publish a rebuilt chunk mesh. The chunk keeps its entity and mesh handle: the asset is replaced in
/// place, so a burn-frame rebuild never despawns the old draw before the new one is GPU-prepared.
fn publish_tree_chunk(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &Handle<StandardMaterial>,
    prev: Option<TreeChunkSlot>,
    buf: MeshBuf,
    key: u64,
) -> TreeChunkSlot {
    let empty = buf.is_empty();
    match prev {
        Some(mut slot) => {
            if empty {
                commands.entity(slot.entity).try_insert(Visibility::Hidden);
            } else {
                let mesh = buf.into_mesh();
                match slot.mesh.as_ref().filter(|h| meshes.contains(*h)) {
                    Some(h) => {
                        let _ = meshes.insert(h, mesh);
                        commands.entity(slot.entity).try_insert(Visibility::Inherited);
                    }
                    None => {
                        let h = meshes.add(mesh);
                        commands.entity(slot.entity).try_insert((
                            Mesh3d(h.clone()),
                            MeshMaterial3d(material.clone()),
                            layer(),
                            Pickable::IGNORE,
                            Visibility::Inherited,
                        ));
                        slot.mesh = Some(h);
                    }
                }
            }
            slot.key = key;
            slot
        }
        None if empty => TreeChunkSlot {
            entity: commands.spawn((PixelWorldTreeChunk, Transform::IDENTITY, Visibility::Hidden)).id(),
            key,
            mesh: None,
        },
        None => {
            let h = meshes.add(buf.into_mesh());
            let entity = commands
                .spawn((
                    PixelWorldTreeChunk,
                    Mesh3d(h.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::IDENTITY,
                    Visibility::Inherited,
                    layer(),
                    Pickable::IGNORE,
                ))
                .id();
            TreeChunkSlot { entity, key, mesh: Some(h) }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Construction sites
// ---------------------------------------------------------------------------------------------

#[derive(Component)]
pub struct PixelWorldSiteLayer;

#[derive(Default)]
pub(crate) struct SiteLayerState {
    key: Option<u64>,
    material: Option<Handle<StandardMaterial>>,
    entity: Option<Entity>,
}

/// Wall colour and full height (world units, one tile = one unit) per archetype.
fn archetype_look(a: SiteArchetype) -> ([f32; 3], [f32; 3], f32) {
    match a {
        SiteArchetype::CivilHousing => ([0.80, 0.64, 0.50], [0.58, 0.26, 0.20], 1.8),
        SiteArchetype::Factory => ([0.58, 0.60, 0.62], [0.36, 0.38, 0.42], 2.6),
        SiteArchetype::PowerPlant => ([0.64, 0.62, 0.56], [0.30, 0.30, 0.32], 3.4),
        SiteArchetype::RailDepot => ([0.52, 0.44, 0.36], [0.34, 0.30, 0.28], 1.6),
        SiteArchetype::MilitaryBase => ([0.44, 0.49, 0.34], [0.32, 0.36, 0.25], 1.6),
        SiteArchetype::RadarSite | SiteArchetype::SensorPost => ([0.72, 0.74, 0.76], [0.55, 0.57, 0.60], 1.0),
        SiteArchetype::TrenchLine => ([0.34, 0.27, 0.19], [0.30, 0.24, 0.17], 0.3),
        SiteArchetype::BunkerComplex => ([0.58, 0.58, 0.54], [0.46, 0.46, 0.43], 1.0),
        SiteArchetype::DefensiveWall => ([0.62, 0.60, 0.57], [0.52, 0.50, 0.48], 1.4),
        SiteArchetype::DragonTeeth => ([0.68, 0.68, 0.65], [0.60, 0.60, 0.58], 0.6),
        SiteArchetype::Minefield => ([0.42, 0.32, 0.22], [0.42, 0.32, 0.22], 0.08),
        SiteArchetype::FuelDepot => ([0.78, 0.75, 0.58], [0.62, 0.58, 0.42], 1.6),
        SiteArchetype::WaterPlant => ([0.56, 0.66, 0.74], [0.40, 0.50, 0.60], 1.6),
        SiteArchetype::Dam => ([0.48, 0.50, 0.54], [0.32, 0.34, 0.38], 2.8),
    }
}

/// Height fraction + colour override per phase (`None` = archetype colours).
fn phase_look(p: SiteConstructionPhase) -> (f32, Option<[f32; 3]>, f32) {
    match p {
        SiteConstructionPhase::Planned | SiteConstructionPhase::Surveying => (0.0, Some([0.92, 0.84, 0.36]), 1.0),
        SiteConstructionPhase::Clearing => (0.0, Some([0.48, 0.36, 0.22]), 1.0),
        SiteConstructionPhase::Foundation => (0.0, Some([0.66, 0.66, 0.63]), 1.0),
        SiteConstructionPhase::UnderConstruction => (0.55, Some([0.82, 0.66, 0.32]), 1.0),
        SiteConstructionPhase::Provisioning => (0.9, None, 1.0),
        SiteConstructionPhase::Operational => (1.0, None, 1.0),
        SiteConstructionPhase::Damaged => (0.7, None, 0.45),
        SiteConstructionPhase::Offline => (1.0, None, 0.7),
        SiteConstructionPhase::Abandoned => (0.45, None, 0.35),
    }
}

/// Drawn vertical extent `(lo, hi)` of a site's columns, shared by the mesh and the voxel pick so
/// a click hits exactly what is drawn (OP-1).
pub(crate) fn site_vertical_extent<'a>(
    planned: &PlannedSite,
    site: &ConstructionSite,
    tiles: impl Iterator<Item = &'a IVec2>,
    field: &PixelWorldTerrainField,
) -> Option<(f32, f32)> {
    let mut top = f32::NEG_INFINITY;
    let mut bottom = f32::INFINITY;
    let mut n = 0u32;
    for t in tiles {
        n += 1;
        if let Some(i) = field.index(t.x, t.y) {
            // Ground only: water tops made every flooded building stretch with the flood (CLOSE-W2).
            top = top.max(field.ground[i]);
            bottom = bottom.min(field.ground[i]);
        }
    }
    if !top.is_finite() {
        return None;
    }
    let (_, _, full) = archetype_look(planned.archetype);
    let floors = derive_massing_floors(n, None);
    let full = full * (1.0 + 0.25 * f32::from(floors.saturating_sub(1)));
    let (frac, _, _) = phase_look(site.phase);
    // Slabs (planned / clearing / foundation) sit just over the ground; the rest stand up.
    let height = if frac <= 0.0 {
        match site.phase {
            SiteConstructionPhase::Foundation => 0.3,
            _ => 0.12,
        }
    } else {
        (full * frac).max(0.3)
    };
    Some((bottom - 0.4, top + height))
}

/// One drawn site box for voxel picking (tactical XY bounds + vertical extent).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SitePickBox {
    pub min: Vec2,
    pub max: Vec2,
    pub lo: f32,
    pub hi: f32,
}

/// Drawn site boxes in the voxel view (render publishes, the pointer gate reads; OP-1).
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct PixelWorldSitePickBoxes(pub Vec<SitePickBox>);

/// Publish every site's drawn box so the voxel pick can hit sites standing above the ground
/// (a dam across a river: the ground ray passed over it and landed on the water behind).
pub(crate) fn publish_pixel_world_site_pick_boxes(
    mode: Option<Res<WorldPresentationMode>>,
    cam: Res<PixelWorldCameraFrame>,
    field: Res<PixelWorldTerrainField>,
    sites: Query<(&PlannedSite, &ConstructionSite, &SiteFootprint)>,
    mut out: ResMut<PixelWorldSitePickBoxes>,
) {
    if !pixel_world_on(&mode, &cam, &field) {
        if !out.0.is_empty() {
            out.0.clear();
        }
        return;
    }
    let mut boxes = Vec::new();
    for (planned, site, fp) in &sites {
        let Some((lo, hi)) = site_vertical_extent(planned, site, fp.tiles.iter(), &field) else {
            continue;
        };
        let (mut mn, mut mx) = (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY));
        for t in &fp.tiles {
            mn = mn.min(t.as_vec2());
            mx = mx.max(t.as_vec2() + Vec2::ONE);
        }
        boxes.push(SitePickBox { min: mn, max: mx, lo, hi });
    }
    if out.0 != boxes {
        out.0 = boxes;
    }
}

/// Nearest ray entry into any site box: `(t, tactical xy inside the footprint)`.
#[must_use]
pub fn ray_hit_site_boxes(boxes: &[SitePickBox], origin: Vec3, dir: Vec3) -> Option<(f32, Vec2)> {
    let mut best: Option<(f32, Vec2)> = None;
    for b in boxes {
        // World: x = tile x, y = height, z = -tile y.
        let bmin = Vec3::new(b.min.x, b.lo, -b.max.y);
        let bmax = Vec3::new(b.max.x, b.hi, -b.min.y);
        let (mut t0, mut t1) = (0.0f32, f32::INFINITY);
        for a in 0..3 {
            let (o, d, lo, hi) = (origin[a], dir[a], bmin[a], bmax[a]);
            if d.abs() < 1e-8 {
                if o < lo || o > hi {
                    t0 = f32::INFINITY;
                }
                continue;
            }
            let (ta, tb) = ((lo - o) / d, (hi - o) / d);
            t0 = t0.max(ta.min(tb));
            t1 = t1.min(ta.max(tb));
        }
        if t0 <= t1 && t0.is_finite() && best.is_none_or(|(bt, _)| t0 < bt) {
            let p = origin + dir * t0;
            let xy = Vec2::new(p.x, -p.z).clamp(b.min + 0.01, b.max - 0.01);
            best = Some((t0, xy));
        }
    }
    best
}

fn build_site(
    buf: &mut MeshBuf,
    planned: &PlannedSite,
    site: &ConstructionSite,
    fp: &SiteFootprint,
    field: &PixelWorldTerrainField,
    mask: Option<&super::DestructionMaskBook>,
) {
    if fp.tiles.is_empty() {
        return;
    }
    // DESTROY-T2.1: omit columns on destroyed mask tiles (same book as ground VS).
    let tiles: std::collections::HashSet<IVec2> = fp
        .tiles
        .iter()
        .copied()
        .filter(|t| {
            mask.map_or(true, |m| m.cell(t.x, t.y) < super::MASK_DESTROYED)
        })
        .collect();
    if tiles.is_empty() {
        return;
    }
    let Some((lo, hi)) = site_vertical_extent(planned, site, tiles.iter(), field) else {
        return;
    };
    let (wall, roof, _) = archetype_look(planned.archetype);
    let (frac, tint, dim) = phase_look(site.phase);
    let wall = tint.unwrap_or(wall);
    let roof = tint.map(|c| [c[0] * 0.9, c[1] * 0.9, c[2] * 0.9]).unwrap_or(roof);
    let side = shade(wall, dim * 0.92, 1.0);
    let cap = shade(roof, dim, 1.0);
    for t in &tiles {
        let (x0, y0, x1, y1) = (t.x as f32, t.y as f32, (t.x + 1) as f32, (t.y + 1) as f32);
        // Top.
        buf.quad([p3(x0, y0, hi), p3(x1, y0, hi), p3(x1, y1, hi), p3(x0, y1, hi)], Vec3::Y, [Vec2::ZERO; 4], cap);
        // Outer walls only.
        for (d, n, a, b) in [
            (IVec2::new(0, -1), Vec3::Z, Vec2::new(x0, y0), Vec2::new(x1, y0)),
            (IVec2::new(0, 1), Vec3::NEG_Z, Vec2::new(x0, y1), Vec2::new(x1, y1)),
            (IVec2::new(-1, 0), Vec3::NEG_X, Vec2::new(x0, y0), Vec2::new(x0, y1)),
            (IVec2::new(1, 0), Vec3::X, Vec2::new(x1, y0), Vec2::new(x1, y1)),
        ] {
            if tiles.contains(&(*t + d)) {
                continue;
            }
            buf.quad([p3(a.x, a.y, lo), p3(b.x, b.y, lo), p3(b.x, b.y, hi), p3(a.x, a.y, hi)], n, [Vec2::ZERO; 4], side);
        }
    }
    // Masts / stacks read the archetype at a glance once the shell stands.
    if frac >= 0.9 {
        let o = tiles.iter().copied().min_by_key(|t| (t.y, t.x)).unwrap_or_default();
        let c = Vec2::new(o.x as f32 + 0.5, o.y as f32 + 0.5);
        let mast = |buf: &mut MeshBuf, w: f32, h: f32, col: [f32; 3]| {
            column(buf, c.x - w, c.y - w, c.x + w, c.y + w, hi, hi + h, shade(col, dim, 1.0), shade(col, dim * 0.85, 1.0));
        };
        match planned.archetype {
            SiteArchetype::PowerPlant => mast(buf, 0.18, 3.0, [0.45, 0.42, 0.40]),
            SiteArchetype::RadarSite => mast(buf, 0.08, 3.5, [0.80, 0.82, 0.85]),
            SiteArchetype::SensorPost => mast(buf, 0.06, 2.4, [0.80, 0.82, 0.85]),
            SiteArchetype::Factory => mast(buf, 0.14, 1.6, [0.50, 0.40, 0.34]),
            // Missile pad placeholder: low bunker + angled launcher rail.
            SiteArchetype::MilitaryBase => {
                mast(buf, 0.22, 0.55, [0.38, 0.40, 0.32]);
                let rail = [0.55, 0.52, 0.42];
                column(
                    buf,
                    c.x - 0.08,
                    c.y - 0.35,
                    c.x + 0.08,
                    c.y + 0.55,
                    hi + 0.5,
                    hi + 1.8,
                    shade(rail, dim, 1.0),
                    shade(rail, dim * 0.85, 1.0),
                );
            }
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_pixel_world_sites(
    mut commands: Commands,
    mode: Option<Res<WorldPresentationMode>>,
    cam: Res<PixelWorldCameraFrame>,
    field: Res<PixelWorldTerrainField>,
    mask: Option<Res<super::DestructionMaskBook>>,
    sites: Query<(Entity, &PlannedSite, &ConstructionSite, &SiteFootprint)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: Local<SiteLayerState>,
) {
    if !pixel_world_on(&mode, &cam, &field) {
        return;
    }
    let mut key = mix64(field.revision ^ mask.as_ref().map_or(0, |m| m.revision));
    for (e, planned, site, fp) in &sites {
        key = mix64(
            key ^ e.to_bits()
                ^ (site.phase as u64) << 48
                ^ (planned.archetype as u64) << 56
                ^ (fp.tiles.len() as u64) << 32
                ^ fp.tiles.first().map_or(0, |t| (t.x as u32 as u64) ^ (t.y as u32 as u64) << 16),
        );
    }
    if state.key == Some(key) {
        return;
    }
    state.key = Some(key);
    let mut buf = MeshBuf::default();
    let mask_ref = mask.as_deref();
    for (_, planned, site, fp) in &sites {
        build_site(&mut buf, planned, site, fp, field.as_ref(), mask_ref);
    }
    let material = state
        .material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.8,
                reflectance: 0.15,
                ..default()
            })
        })
        .clone();
    if let Some(e) = state.entity.take() {
        commands.entity(e).despawn();
    }
    if !buf.is_empty() {
        state.entity = Some(
            commands
                .spawn((
                    PixelWorldSiteLayer,
                    Mesh3d(meshes.add(buf.into_mesh())),
                    MeshMaterial3d(material),
                    Transform::IDENTITY,
                    Visibility::Inherited,
                    layer(),
                    Pickable::IGNORE,
                ))
                .id(),
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Build ghost (footprint tiles + massing columns)
// ---------------------------------------------------------------------------------------------

#[derive(Component)]
pub struct PixelWorldGhostLayer;

#[derive(Default)]
pub(crate) struct GhostLayerState {
    material: Option<Handle<StandardMaterial>>,
    mesh: Option<Handle<Mesh>>,
    entity: Option<Entity>,
}

fn ghost_rgb(kind: FootprintTileColorKind) -> [f32; 3] {
    match kind {
        FootprintTileColorKind::Valid => [0.30, 0.92, 0.45],
        FootprintTileColorKind::Risky => [0.98, 0.80, 0.22],
        FootprintTileColorKind::Invalid => [0.98, 0.26, 0.20],
    }
}

/// Ghost tiles cap (a drag can request thousands; the slab reads the same past this).
const GHOST_TILE_CAP: usize = 6000;

/// Paints ghost footprint + massing from [`ConstructionVisualRequests`] (**RO**).
/// RND-09: prefer `massing_instances` when the sole writer packed a batch (≥ threshold).
#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_pixel_world_ghost(
    mut commands: Commands,
    mode: Option<Res<WorldPresentationMode>>,
    cam: Res<PixelWorldCameraFrame>,
    field: Res<PixelWorldTerrainField>,
    requests: Option<Res<ConstructionVisualRequests>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: Local<GhostLayerState>,
) {
    if !pixel_world_on(&mode, &cam, &field) {
        return;
    }
    let Some(requests) = requests else {
        return;
    };
    if !requests.is_changed() && state.entity.is_some() {
        return;
    }
    let mut buf = MeshBuf::default();
    let lock: std::collections::HashSet<IVec2> = requests.footprint_lock_tiles.iter().copied().collect();
    let massing: std::collections::HashSet<IVec2> = if !requests.massing_instances.is_empty() {
        requests.massing_instances.iter().map(|c| c.tile).collect()
    } else {
        requests.massing_columns.iter().map(|c| c.tile).collect()
    };
    for t in requests.footprint_tiles.iter().take(GHOST_TILE_CAP) {
        if massing.contains(&t.tile) {
            continue;
        }
        let Some(s) = field.surface_at_cell(t.tile.x, t.tile.y) else {
            continue;
        };
        let rgb = match t.fill_override {
            Some(c) => [f32::from(c.r) / 255.0, f32::from(c.g) / 255.0, f32::from(c.b) / 255.0],
            None => ghost_rgb(t.color_kind),
        };
        let a = (0.25 + 0.45 * t.weight).min(0.75);
        let lift = if lock.contains(&t.tile) { 0.5 } else { 0.3 };
        let (x0, y0) = (t.tile.x as f32 + 0.04, t.tile.y as f32 + 0.04);
        column(&mut buf, x0, y0, x0 + 0.92, y0 + 0.92, s + 0.02, s + lift, shade(rgb, 1.1, a), shade(rgb, 0.9, a));
    }
    // RND-09 batch path: one mesh, many instance rows (gea4v2 11.3.8.2).
    if !requests.massing_instances.is_empty() {
        for c in requests.massing_instances.iter().take(GHOST_TILE_CAP) {
            let Some(s) = field.surface_at_cell(c.tile.x, c.tile.y) else {
                continue;
            };
            let rgb = ghost_rgb(c.color_kind);
            let a = (0.3 + 0.4 * c.weight).min(0.7);
            let (x0, y0) = (c.tile.x as f32 + 0.03, c.tile.y as f32 + 0.03);
            column(
                &mut buf,
                x0,
                y0,
                x0 + 0.94,
                y0 + 0.94,
                s + 0.02,
                s + f32::from(c.floors.max(1)) * 0.9,
                shade(rgb, 1.1, a),
                shade(rgb, 0.85, a),
            );
        }
    } else {
        for c in requests.massing_columns.iter().take(GHOST_TILE_CAP) {
            let Some(s) = field.surface_at_cell(c.tile.x, c.tile.y) else {
                continue;
            };
            let rgb = ghost_rgb(c.color_kind);
            let a = (0.3 + 0.4 * c.weight).min(0.7);
            let (x0, y0) = (c.tile.x as f32 + 0.03, c.tile.y as f32 + 0.03);
            column(
                &mut buf,
                x0,
                y0,
                x0 + 0.94,
                y0 + 0.94,
                s + 0.02,
                s + f32::from(c.floors.max(1)) * 0.9,
                shade(rgb, 1.1, a),
                shade(rgb, 0.85, a),
            );
        }
    }
    let material = state
        .material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })
        })
        .clone();
    // Empty builds never reach `Assets<Mesh>` (zero-vertex meshes trip the render slab allocator):
    // keep the last asset and hide the layer instead.
    let before = state.mesh.as_ref().map(Handle::id);
    let drawable = !buf.is_empty()
        && crate::render::upload_layer_mesh(&mut meshes, &mut state.mesh, buf.into_mesh());
    let vis = if drawable { Visibility::Inherited } else { Visibility::Hidden };
    let Some(mesh) = state.mesh.clone() else {
        return;
    };
    match state.entity {
        Some(e) => {
            if before != Some(mesh.id()) {
                commands.entity(e).try_insert((Mesh3d(mesh), vis));
            } else {
                commands.entity(e).try_insert(vis);
            }
        }
        None => {
            state.entity = Some(
                commands
                    .spawn((PixelWorldGhostLayer, Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY, vis, layer(), Pickable::IGNORE))
                    .id(),
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Fronts
// ---------------------------------------------------------------------------------------------

#[derive(Component)]
pub struct PixelWorldFrontLayer;

#[derive(Default)]
pub(crate) struct FrontLayerState {
    key: Option<u64>,
    material: Option<Handle<StandardMaterial>>,
    mesh: Option<Handle<Mesh>>,
    entity: Option<Entity>,
}

/// Longest edge (tiles) a draped triangle may keep; longer ones are bisected so they follow the
/// columns instead of cutting through hills.
const DRAPE_MAX_EDGE: f32 = 1.0;
/// Overlay lift above the surface (world units); the 2D layer's z order adds a little more.
const DRAPE_LIFT: f32 = 0.35;
/// Roads / rails sit just over the ground.
const PATH_LIFT: f32 = 0.06;

/// Drape tile-space overlay triangles (`[x, y, z_order]`) on the voxel surface.
/// Deck heights `(a, b)` when the straight segment `a → b` crosses a water cell; `None` for a dry
/// path. Each end sits on its bank (surface + lift); the deck never dips below the water surface.
#[must_use]
pub(crate) fn bridge_deck_heights(field: &PixelWorldTerrainField, a: Vec2, b: Vec2) -> Option<(f32, f32)> {
    let len = (b - a).length();
    let steps = (len * 2.0).ceil().max(2.0) as usize;
    let mut wet = false;
    let mut water_top = f32::NEG_INFINITY;
    for k in 0..=steps {
        let p = a + (b - a) * (k as f32 / steps as f32);
        if let Some(i) = field.index(p.x.floor() as i32, p.y.floor() as i32) {
            if field.water[i].is_finite() {
                wet = true;
                water_top = water_top.max(field.water[i]);
            }
        }
    }
    if !wet {
        return None;
    }
    let ha = field.surface_max_around(a) + PATH_LIFT;
    let hb = field.surface_max_around(b) + PATH_LIFT;
    let clear = water_top + 0.6;
    Some((ha.max(clear), hb.max(clear)))
}

/// Straight bridge deck (top + two side faces) from `a` to `b` at heights `ha` → `hb`.
fn push_bridge_deck(buf: &mut MeshBuf, a: Vec2, b: Vec2, n: Vec2, ha: f32, hb: f32, col: [f32; 4]) {
    let t = 0.25; // deck thickness (display units)
    let (a0, a1, b0, b1) = (a - n, a + n, b - n, b + n);
    let top = [p3(a0.x, a0.y, ha), p3(a1.x, a1.y, ha), p3(b1.x, b1.y, hb), p3(b0.x, b0.y, hb)];
    buf.quad(top, Vec3::Y, [Vec2::ZERO; 4], col);
    let side = shade([col[0], col[1], col[2]], 0.7, 1.0);
    for (u, v) in [(a0, b0), (a1, b1)] {
        let (hu, hv) = (ha, hb);
        buf.quad([p3(u.x, u.y, hu - t), p3(v.x, v.y, hv - t), p3(v.x, v.y, hv), p3(u.x, u.y, hu)], Vec3::Y, [Vec2::ZERO; 4], side);
    }
}

fn drape(positions: &[[f32; 3]], colors: &[[f32; 4]], indices: &[u32], field: &PixelWorldTerrainField, lift: f32) -> MeshBuf {
    let mut buf = MeshBuf::default();
    let mut stack: Vec<([Vec3; 3], [[f32; 4]; 3])> = Vec::new();
    for tri in indices.chunks_exact(3) {
        let v = |k: usize| Vec3::from_array(positions[tri[k] as usize]);
        let c = |k: usize| colors[tri[k] as usize];
        stack.push(([v(0), v(1), v(2)], [c(0), c(1), c(2)]));
        let mut guard = 0u32;
        while let Some((p, c)) = stack.pop() {
            guard += 1;
            let edges = [(p[0] - p[1]).truncate().length(), (p[1] - p[2]).truncate().length(), (p[2] - p[0]).truncate().length()];
            let (mut longest, mut e) = (0usize, edges[0]);
            for (k, len) in edges.iter().enumerate() {
                if *len > e {
                    e = *len;
                    longest = k;
                }
            }
            if e > DRAPE_MAX_EDGE && guard < 4096 {
                let (a, b, o) = (longest, (longest + 1) % 3, (longest + 2) % 3);
                let m = (p[a] + p[b]) * 0.5;
                let mc = [
                    (c[a][0] + c[b][0]) * 0.5,
                    (c[a][1] + c[b][1]) * 0.5,
                    (c[a][2] + c[b][2]) * 0.5,
                    (c[a][3] + c[b][3]) * 0.5,
                ];
                stack.push(([p[a], m, p[o]], [c[a], mc, c[o]]));
                stack.push(([m, p[b], p[o]], [mc, c[b], c[o]]));
                continue;
            }
            let base = buf.positions.len() as u32;
            for k in 0..3 {
                let h = field.surface_max_around(p[k].truncate()) + lift + (p[k].z - 1.0).max(0.0) * 4.0;
                buf.positions.push([p[k].x, h, -p[k].y]);
                buf.normals.push([0.0, 1.0, 0.0]);
                buf.uvs.push([0.0, 0.0]);
                buf.colors.push(c[k]);
            }
            buf.indices.extend([base, base + 1, base + 2]);
        }
    }
    buf
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_pixel_world_fronts(
    mut commands: Commands,
    mode: Option<Res<WorldPresentationMode>>,
    cam: Res<PixelWorldCameraFrame>,
    field: Res<PixelWorldTerrainField>,
    src: FrontOverlaySources,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: Local<FrontLayerState>,
) {
    let on = pixel_world_on(&mode, &cam, &field);
    if !on {
        if let Some(e) = state.entity {
            commands.entity(e).try_insert(Visibility::Hidden);
        }
        state.key = None;
        return;
    }
    let px = cam.px_per_tile;
    let key = mix64(src.content_key(px) ^ field.revision.wrapping_mul(0xA24B_AED4_963E_E407));
    if state.key == Some(key) {
        return;
    }
    state.key = Some(key);
    let geo = src.build_geometry(px);
    let buf = drape(&geo.positions, &geo.colors, &geo.indices, field.as_ref(), DRAPE_LIFT);
    let material = state
        .material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                double_sided: true,
                cull_mode: None,
                depth_bias: 8.0,
                ..default()
            })
        })
        .clone();
    // Empty builds never reach `Assets<Mesh>` (zero-vertex meshes trip the render slab allocator):
    // keep the last asset and hide the layer instead.
    let before = state.mesh.as_ref().map(Handle::id);
    let drawable = !buf.is_empty()
        && crate::render::upload_layer_mesh(&mut meshes, &mut state.mesh, buf.into_mesh());
    let vis = if drawable { Visibility::Inherited } else { Visibility::Hidden };
    let Some(mesh) = state.mesh.clone() else {
        return;
    };
    match state.entity {
        Some(e) => {
            if before != Some(mesh.id()) {
                commands.entity(e).try_insert((Mesh3d(mesh), vis));
            } else {
                commands.entity(e).try_insert(vis);
            }
        }
        None => {
            state.entity = Some(
                commands
                    .spawn((PixelWorldFrontLayer, Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY, vis, layer(), Pickable::IGNORE))
                    .id(),
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Roads / rails / corridor previews (the same path requests the top-down ribbons draw)
// ---------------------------------------------------------------------------------------------

#[derive(Component)]
pub struct PixelWorldPathLayer;

#[derive(Default)]
pub(crate) struct PathLayerState {
    key: Option<u64>,
    material: Option<Handle<StandardMaterial>>,
    mesh: Option<Handle<Mesh>>,
    entity: Option<Entity>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_pixel_world_paths(
    mut commands: Commands,
    mode: Option<Res<WorldPresentationMode>>,
    cam: Res<PixelWorldCameraFrame>,
    field: Res<PixelWorldTerrainField>,
    requests: Option<Res<ConstructionVisualRequests>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: Local<PathLayerState>,
) {
    if !pixel_world_on(&mode, &cam, &field) {
        return;
    }
    let Some(req) = requests else {
        return;
    };
    let q = |v: f32| (v * 32.0).round() as i64 as u64;
    let mut key = mix64(field.revision ^ (req.paths.len() as u64) << 40);
    for p in &req.paths {
        key = mix64(key ^ q(p.start.x) ^ q(p.start.z) << 16 ^ q(p.end.x) << 32 ^ q(p.end.z) << 48);
        key = mix64(key ^ u64::from(p.committed) ^ u64::from(p.valid) << 1 ^ u64::from(p.slope_ok) << 2 ^ (p.kind as u64) << 3);
    }
    if state.key == Some(key) {
        return;
    }
    state.key = Some(key);
    let (mut positions, mut colors, mut indices) = (Vec::new(), Vec::new(), Vec::new());
    let mut decks = MeshBuf::default();
    for p in &req.paths {
        let a = crate::gui::tactical::map_camera::map_plane_horizontal_xy(p.start);
        let b = crate::gui::tactical::map_camera::map_plane_horizontal_xy(p.end);
        let d = b - a;
        if d.length_squared() < 1e-6 {
            continue;
        }
        let n = Vec2::new(-d.y, d.x).normalize() * (crate::render::world_map_network_mesh::world_ribbon_width(p.width) * 0.5);
        let c = LinearRgba::from(crate::render::world_map_network_mesh::path_color(p.kind, p.valid, p.slope_ok, p.committed));
        let col = [c.red, c.green, c.blue, c.alpha.max(0.85)];
        // OP-7: a path that crosses water is a bridge — a straight deck between its bank ends,
        // not a ribbon draped down the banks and across the riverbed.
        if let Some((ha, hb)) = bridge_deck_heights(field.as_ref(), a, b) {
            push_bridge_deck(&mut decks, a, b, n, ha, hb, col);
            continue;
        }
        let base = positions.len() as u32;
        for v in [a - n, a + n, b + n, b - n] {
            positions.push([v.x, v.y, 1.0]);
            colors.push(col);
        }
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    let mut buf = drape(&positions, &colors, &indices, field.as_ref(), PATH_LIFT);
    buf.append(decks);
    let material = state
        .material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 0.9,
                double_sided: true,
                cull_mode: None,
                depth_bias: 4.0,
                ..default()
            })
        })
        .clone();
    // Empty builds never reach `Assets<Mesh>` (zero-vertex meshes trip the render slab allocator):
    // keep the last asset and hide the layer instead.
    let before = state.mesh.as_ref().map(Handle::id);
    let drawable = !buf.is_empty()
        && crate::render::upload_layer_mesh(&mut meshes, &mut state.mesh, buf.into_mesh());
    let vis = if drawable { Visibility::Inherited } else { Visibility::Hidden };
    let Some(mesh) = state.mesh.clone() else {
        return;
    };
    match state.entity {
        Some(e) => {
            if before != Some(mesh.id()) {
                commands.entity(e).try_insert((Mesh3d(mesh), vis));
            } else {
                commands.entity(e).try_insert(vis);
            }
        }
        None => {
            state.entity = Some(
                commands
                    .spawn((PixelWorldPathLayer, Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY, vis, layer(), Pickable::IGNORE))
                    .id(),
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Formations + ballistic placeholders (voxel guys / missile body)
// ---------------------------------------------------------------------------------------------

#[derive(Component)]
pub struct PixelWorldUnitLayer;

#[derive(Default)]
pub(crate) struct UnitLayerState {
    key: Option<u64>,
    material: Option<Handle<StandardMaterial>>,
    mesh: Option<Handle<Mesh>>,
    entity: Option<Entity>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_pixel_world_units(
    mut commands: Commands,
    mode: Option<Res<WorldPresentationMode>>,
    cam: Res<PixelWorldCameraFrame>,
    field: Res<PixelWorldTerrainField>,
    formations: Query<&Formation>,
    ballistics: Option<Res<BallisticPresentFrame>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: Local<UnitLayerState>,
) {
    let on = pixel_world_on(&mode, &cam, &field);
    if !on {
        if let Some(e) = state.entity {
            commands.entity(e).try_insert(Visibility::Hidden);
        }
        state.key = None;
        return;
    }
    let mut key = mix64(field.revision ^ (formations.iter().len() as u64) << 20);
    for f in &formations {
        key = mix64(
            key ^ ((f.tile.x as u64) << 32)
                ^ (f.tile.y as u64)
                ^ ((f.kind as u8 as u64) << 48)
                ^ ((f.strength * 16.0) as u64),
        );
    }
    if let Some(b) = ballistics.as_ref() {
        key = mix64(key ^ b.revision ^ (b.in_flight as u64) << 8);
    }
    if state.key == Some(key) {
        return;
    }
    state.key = Some(key);

    let mut buf = MeshBuf::default();
    for f in &formations {
        let base = field
            .surface_at_cell(f.tile.x, f.tile.y)
            .unwrap_or(field.min_h);
        let (w, h, rgb) = match f.kind {
            UnitKind::Infantry => (0.18, 0.55, [0.22, 0.28, 0.18]),
            UnitKind::AirDefense => (0.28, 0.85, [0.42, 0.38, 0.28]),
            UnitKind::Armor => (0.32, 0.70, [0.36, 0.40, 0.28]),
            UnitKind::Artillery => (0.30, 0.75, [0.40, 0.36, 0.26]),
            _ => (0.22, 0.60, [0.34, 0.36, 0.30]),
        };
        let c = Vec2::new(f.tile.x as f32 + 0.5, f.tile.y as f32 + 0.5);
        column(
            &mut buf,
            c.x - w,
            c.y - w,
            c.x + w,
            c.y + w,
            base,
            base + h * f.strength.max(0.25),
            shade(rgb, 1.05, 1.0),
            shade(rgb, 0.85, 1.0),
        );
    }
    if let Some(b) = ballistics.as_ref() {
        let tm = TILE_METRES_GAMEPLAY.max(1e-3);
        for (a, p) in &b.trail_segments {
            let tile = Vec2::new(p.x / tm, p.y / tm);
            let base = field.surface_at(tile).unwrap_or(field.min_h) + 1.2;
            let c = tile;
            column(
                &mut buf,
                c.x - 0.12,
                c.y - 0.12,
                c.x + 0.12,
                c.y + 0.12,
                base,
                base + 0.55,
                shade([0.85, 0.75, 0.35], 1.0, 1.0),
                shade([0.70, 0.55, 0.25], 1.0, 1.0),
            );
            let _ = a;
        }
        for (p, age) in &b.impacts {
            let tile = Vec2::new(p.x / tm, p.y / tm);
            let base = field.surface_at(tile).unwrap_or(field.min_h);
            let r = (0.55 + (1.0 - age / 2.5).max(0.0) * 0.6).clamp(0.4, 1.2);
            column(
                &mut buf,
                tile.x - r * 0.5,
                tile.y - r * 0.5,
                tile.x + r * 0.5,
                tile.y + r * 0.5,
                base,
                base + 0.35,
                shade([0.95, 0.45, 0.15], 1.0, 0.9),
                shade([0.80, 0.25, 0.10], 1.0, 0.9),
            );
        }
    }

    let material = state
        .material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            })
        })
        .clone();
    let before = state.mesh.as_ref().map(Handle::id);
    let drawable = !buf.is_empty()
        && crate::render::upload_layer_mesh(&mut meshes, &mut state.mesh, buf.into_mesh());
    let vis = if drawable {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let Some(mesh) = state.mesh.clone() else {
        return;
    };
    match state.entity {
        Some(e) => {
            if before != Some(mesh.id()) {
                commands.entity(e).try_insert((Mesh3d(mesh), vis));
            } else {
                commands.entity(e).try_insert(vis);
            }
        }
        None => {
            state.entity = Some(
                commands
                    .spawn((
                        PixelWorldUnitLayer,
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        Transform::IDENTITY,
                        vis,
                        layer(),
                        Pickable::IGNORE,
                    ))
                    .id(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::extraction::VegExtractModifiers;
    use crate::render::pixel_world::field::NO_WATER;
    use crate::render::world_map_vegetation_mesh::build_vegetation_geometry;
    use crate::systems::ecology::{SuccessionTopologyStage, CELL_OLD};

    const CHUNK: u32 = 32;

    fn row(species: impl Fn(u32) -> u8) -> VegExtractRow {
        let n = (CHUNK * CHUNK) as usize;
        VegExtractRow {
            coord: IVec2::ZERO,
            planning_glyph: '#',
            extract_glyph: '#',
            modifiers: VegExtractModifiers { burn_active: false, mean_density: 0.9 },
            variant_key: "veg_topo_patch".to_owned(),
            succession_stage: SuccessionTopologyStage::OldGrowth,
            burn_active: false,
            frame_index: 0,
            cells: Some(vec![CELL_OLD; n].into()),
            cells_rev: 1,
            species: Some((0..n as u32).map(|i| species(i % CHUNK)).collect()),
            species_rev: 1,
        }
    }

    fn flat_field() -> PixelWorldTerrainField {
        let mut f = PixelWorldTerrainField::default();
        f.width = CHUNK;
        f.height = CHUNK;
        f.ground = vec![0.0; (CHUNK * CHUNK) as usize];
        f.water = vec![NO_WATER; (CHUNK * CHUNK) as usize];
        f
    }

    fn build(row: &VegExtractRow, lod: VegLod) -> (MeshBuf, usize) {
        let mut scratch = Vec::new();
        let buf = build_tree_chunk(row, UVec2::splat(CHUNK), lod, &flat_field(), &VegSpeciesTable::default(), &mut scratch);
        (buf, scratch.len())
    }

    #[test]
    fn burn_tick_does_not_change_key_of_unburnt_chunk() {
        let mut r = row(|_| 0);
        let k0 = tree_chunk_key(&r, VegLod::Fine, 7);
        r.frame_index = 5;
        assert_eq!(k0, tree_chunk_key(&r, VegLod::Fine, 7), "no burn: frame tick is ignored");
        r.burn_active = true;
        let a = tree_chunk_key(&r, VegLod::Fine, 7);
        r.frame_index = 8;
        assert_ne!(a, tree_chunk_key(&r, VegLod::Fine, 7), "burning chunk animates");
    }

    #[test]
    fn chunk_rebuild_keeps_entity_and_mesh_handle() {
        let mut world = World::new();
        world.insert_resource(Assets::<Mesh>::default());
        let material = Handle::<StandardMaterial>::default();
        let table = VegSpeciesTable::default();
        let r = row(|_| 0);
        let mut scratch = Vec::new();
        let field = flat_field();
        let mut publish = |world: &mut World, prev: Option<TreeChunkSlot>, key: u64| {
            let buf = build_tree_chunk(&r, UVec2::splat(CHUNK), VegLod::Fine, &field, &table, &mut scratch);
            let mut meshes = world.remove_resource::<Assets<Mesh>>().unwrap();
            let mut queue = bevy::ecs::world::CommandQueue::default();
            let slot = {
                let mut commands = Commands::new(&mut queue, world);
                publish_tree_chunk(&mut commands, &mut meshes, &material, prev, buf, key)
            };
            queue.apply(world);
            world.insert_resource(meshes);
            slot
        };
        let first = publish(&mut world, None, 1);
        let (e, h) = (first.entity, first.mesh.clone().unwrap());
        let second = publish(&mut world, Some(first), 2);
        assert_eq!(second.entity, e, "entity is reused");
        assert_eq!(second.mesh.as_ref().unwrap().id(), h.id(), "mesh asset replaced in place");
        assert_eq!(second.key, 2);
        assert_eq!(world.resource::<Assets<Mesh>>().len(), 1, "no orphaned mesh assets");
        assert!(world.get_entity(e).is_ok());
    }

    #[test]
    fn voxel_and_tactical_place_the_same_plants() {
        let r = row(|_| 0);
        let (buf, n) = build(&r, VegLod::Fine);
        assert!(n > 100 && !buf.is_empty());
        let (flat, _, _) = build_vegetation_geometry(
            std::slice::from_ref(&r),
            UVec2::splat(CHUNK),
            VegLod::Fine,
            None,
            &VegSpeciesTable::default(),
        );
        assert_eq!(flat.symbols, n, "one sampler: same site count in both views");
    }

    #[test]
    fn species_changes_shape_not_placement() {
        let t = VegSpeciesTable::default();
        let spruce = t.species.iter().position(|s| s.id == "spruce").unwrap() as u8;
        let oak = t.species.iter().position(|s| s.id == "oak").unwrap() as u8;
        let (b_oak, n_oak) = build(&row(|_| oak), VegLod::Fine);
        let (b_spruce, n_spruce) = build(&row(|_| spruce), VegLod::Fine);
        assert_eq!(n_oak, n_spruce);
        // GPU-C: Fine band = trunk quad + crown quad(s) as axis-aligned rects (4 verts each).
        assert!(b_oak.positions.len() >= n_oak * 4);
        assert!(b_spruce.positions.len() >= n_spruce * 4);
        assert_eq!(b_oak.positions.len() % 4, 0);
        assert_eq!(b_spruce.positions.len() % 4, 0);
        // Spruce is taller than oak: its highest vertex is higher.
        let top = |b: &MeshBuf| b.positions.iter().map(|p| p[1]).fold(f32::MIN, f32::max);
        assert!(top(&b_spruce) > top(&b_oak));
    }

    #[test]
    fn mixed_stand_builds_both_shapes_deterministically() {
        let r = row(|x| if x < 16 { 0 } else { 2 });
        let (a, _) = build(&r, VegLod::Fine);
        let (b, _) = build(&r, VegLod::Fine);
        assert_eq!(a.positions, b.positions);
        let (oak, _) = build(&row(|_| 0), VegLod::Fine);
        let (spruce, _) = build(&row(|_| 2), VegLod::Fine);
        assert!(a.positions.len() > oak.positions.len() && a.positions.len() < spruce.positions.len());
    }

    #[test]
    fn coarse_band_keeps_trunks_and_species_tiers() {
        let (fine, nf) = build(&row(|_| 2), VegLod::Fine);
        let (tree, nt) = build(&row(|_| 2), VegLod::Tree);
        assert!(nf > nt && !tree.is_empty());
        // Both bands: trunk box + four spruce tiers per plant (20 verts a box).
        assert_eq!(tree.positions.len(), nt * 5 * 20);
        assert_eq!(fine.positions.len(), nf * 5 * 20);
    }

    fn stage_instance(cover: Cover, rand: f32) -> VegInstance {
        VegInstance { pos: Vec2::new(8.5, 8.5), cover, species: 2, rand, riparian: false, burn_phase: 0 }
    }

    const STAGES: [Cover; 8] = [
        Cover::Grass,
        Cover::Shrub,
        Cover::YoungForest,
        Cover::OldGrowth,
        Cover::Burning,
        Cover::Burnt,
        Cover::Dead,
        Cover::Regrowth,
    ];

    fn one(cover: Cover, rand: f32, step: u32) -> MeshBuf {
        let mut b = MeshBuf::default();
        push_tree(&mut b, &VegSpeciesTable::default(), &stage_instance(cover, rand), 0.0, step_scale(step));
        b
    }

    fn height(b: &MeshBuf) -> f32 {
        b.positions.iter().map(|p| p[1]).fold(f32::MIN, f32::max)
    }

    #[test]
    fn every_stage_draws_at_every_band_deterministically_with_variation() {
        for cover in STAGES {
            for step in [1, 2, 4, 8] {
                let (a, b) = (one(cover, 0.3, step), one(cover, 0.3, step));
                assert!(!a.is_empty(), "{cover:?} step {step} draws");
                assert_eq!(a.positions, b.positions, "{cover:?} deterministic");
                assert!(height(&one(cover, 0.05, step)) != height(&one(cover, 0.95, step)), "{cover:?} size varies");
            }
        }
        // Tree stages carry a trunk below the crown at the coarsest band too: more than one box.
        for cover in [Cover::YoungForest, Cover::OldGrowth, Cover::Burning, Cover::Burnt, Cover::Dead] {
            assert!(one(cover, 0.5, 8).positions.len() > 20, "{cover:?} has trunk + more");
        }
    }

    #[test]
    fn destroy_t2_1_site_tiles_skip_destroyed_mask() {
        use crate::render::pixel_world::{DestructionMaskBook, MASK_DESTROYED};

        let mut book = DestructionMaskBook::default();
        book.ensure_size(8, 8);
        book.mark_tile(2, 3, MASK_DESTROYED);
        let keep = |t: IVec2| book.cell(t.x, t.y) < MASK_DESTROYED;
        assert!(keep(IVec2::new(0, 0)));
        assert!(!keep(IVec2::new(2, 3)));
    }
}

#[cfg(test)]
mod op1_pick_tests {
    use super::{ray_hit_site_boxes, SitePickBox};
    use bevy::prelude::*;

    /// OP-1: an oblique click ray over a dam (box standing above a river) hits the dam, inside its
    /// footprint — the ground-only ray used to pass over it and land on the water behind.
    #[test]
    fn op1_oblique_ray_hits_dam_box() {
        let dam = SitePickBox { min: Vec2::new(154.0, 159.0), max: Vec2::new(166.0, 161.0), lo: 18.0, hi: 36.0 };
        // Eye south of the dam (tile y 140), high up, looking north-down through the dam's face.
        let eye = Vec3::new(160.0, 60.0, -140.0);
        let target = Vec3::new(160.0, 30.0, -160.0);
        let (t, xy) = ray_hit_site_boxes(&[dam], eye, (target - eye).normalize()).expect("hits the dam");
        assert!(t > 0.0);
        assert!(xy.x >= 154.0 && xy.x <= 166.0 && xy.y >= 159.0 && xy.y <= 161.0, "{xy:?}");
        assert!(ray_hit_site_boxes(&[dam], eye, Vec3::new(1.0, -0.1, 0.0).normalize()).is_none());
    }
}

#[cfg(test)]
mod op7_bridge_tests {
    use super::bridge_deck_heights;
    use crate::render::pixel_world::PixelWorldTerrainField;
    use bevy::prelude::*;

    fn canyon() -> PixelWorldTerrainField {
        // 16 wide: banks at 30 on both sides, river cells 6..=9 with bed 5 and water surface 8.
        let mut f = PixelWorldTerrainField::default();
        f.width = 16;
        f.height = 4;
        f.ground = (0..64).map(|i| if (6..=9).contains(&(i % 16)) { 5.0 } else { 30.0 }).collect();
        f.water = (0..64).map(|i| if (6..=9).contains(&(i % 16)) { 8.0 } else { f32::NEG_INFINITY }).collect();
        f.class = vec![Default::default(); 64];
        f
    }

    /// OP-7: a road over the river becomes a deck at bank height, not a ribbon into the riverbed.
    #[test]
    fn op7_water_crossing_is_a_level_deck() {
        let f = canyon();
        let (ha, hb) = bridge_deck_heights(&f, Vec2::new(1.5, 1.5), Vec2::new(14.5, 1.5)).expect("crosses water");
        assert!(ha >= 30.0 && hb >= 30.0, "deck at bank height: {ha} {hb}");
        assert!(bridge_deck_heights(&f, Vec2::new(0.5, 1.5), Vec2::new(4.5, 1.5)).is_none(), "dry road drapes");
    }
}
