//! Simulation world map. Cells live in the same plane as the terrain sprite.
//!
//! Zoom is the camera. A tile center does not take a screen scale, so it cannot
//! slide off the map. The GPU minimap compositor is a separate view of this grid.
//! Egui paints the editor only.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::strategic::{ConstructionSite, SiteFootprint};

use super::visual_authority::{ConstructionVisualRequests, FootprintTileColorKind};

/// Matches the terrain sprite: it is centered on `(width/2, height/2)` and covers
/// `[0, width] × [0, height]`. Tile `(x, z)` sits at the cell center.
#[must_use]
pub fn world_map_tile_center(tile_x: i32, tile_z: i32) -> Vec3 {
    Vec3::new(tile_x as f32 + 0.5, tile_z as f32 + 0.5, 1.0)
}

/// Path sample tile. Construction stores the point as `(x, 0, row)`.
#[must_use]
pub fn path_sample_tile(world: Vec3) -> IVec2 {
    IVec2::new(world.x.floor() as i32, world.z.floor() as i32)
}

fn point_leaves_generated_map(world: Vec3, width: u32, height: u32) -> bool {
    let tile = path_sample_tile(world);
    width == 0
        || height == 0
        || tile.x < 0
        || tile.y < 0
        || (tile.x as u32) >= width
        || (tile.y as u32) >= height
}

/// True when any sample tile is outside `[0, width) × [0, height)`.
/// A zero-size map rejects every sample. An empty slice does not.
#[must_use]
pub fn path_samples_leave_generated_map(samples: &[Vec3], width: u32, height: u32) -> bool {
    samples
        .iter()
        .copied()
        .any(|world| point_leaves_generated_map(world, width, height))
}

/// Control points plus every generated-segment endpoint (curve samples included).
/// True means the Shift+click must return before the commit function.
#[must_use]
pub fn path_commit_leaves_generated_map(
    control_points: &[Vec3],
    segment_endpoints: impl IntoIterator<Item = Vec3>,
    width: u32,
    height: u32,
) -> bool {
    path_samples_leave_generated_map(control_points, width, height)
        || segment_endpoints
            .into_iter()
            .any(|world| point_leaves_generated_map(world, width, height))
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WorldMapFootprintTile {
    tile: IVec2,
}

/// Presentation-only massing stack layer (Ebbtide column silhouette).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WorldMapMassingTile {
    tile: IVec2,
    floor: u8,
}

#[derive(Resource, Default)]
pub(crate) struct WorldMapFootprintStamp {
    key: u64,
    fill_mesh: Option<Handle<Mesh>>,
    rim_mesh: Option<Handle<Mesh>>,
    materials: HashMap<u32, Handle<ColorMaterial>>,
    /// Cached stack/fill mesh sizes (quantized hundredths of a tile).
    stack_meshes: HashMap<u32, Handle<Mesh>>,
}

fn color32_bevy(c: crate::gui::UiRgba) -> Color {
    Color::srgba_u8(c.r, c.g, c.b, c.a)
}

/// Validity color for one cell. The quad stays on [`world_map_tile_center`]; only the material changes.
pub(crate) fn footprint_fill(kind: FootprintTileColorKind, fill: Option<crate::gui::UiRgba>) -> Color {
    if let Some(c) = fill {
        return color32_bevy(c);
    }
    let token = match kind {
        FootprintTileColorKind::Valid => super::ghost_visual::footprint_valid_color(),
        FootprintTileColorKind::Risky => super::ghost_visual::footprint_risky_color(),
        FootprintTileColorKind::Invalid => super::ghost_visual::footprint_invalid_color(),
    };
    color32_bevy(token)
}

fn footprint_rim(kind: FootprintTileColorKind) -> Color {
    let valid_family = !matches!(kind, FootprintTileColorKind::Invalid);
    color32_bevy(super::ghost_visual::footprint_rim_color(valid_family))
}

fn massing_rim(kind: FootprintTileColorKind) -> Color {
    let valid_family = !matches!(kind, FootprintTileColorKind::Invalid);
    color32_bevy(super::ghost_visual::massing_stack_rim_color(valid_family))
}

fn color_kind_key(kind: FootprintTileColorKind) -> u64 {
    match kind {
        FootprintTileColorKind::Valid => 1,
        FootprintTileColorKind::Risky => 2,
        FootprintTileColorKind::Invalid => 3,
    }
}

fn massing_key(columns: &[super::voxel_massing::VoxelColumnRequest]) -> u64 {
    let mut key = columns.len() as u64;
    for col in columns {
        key = key
            .wrapping_mul(0x9E37_79B9)
            .wrapping_add(col.tile.x as u64)
            .wrapping_add((col.tile.y as u64) << 32)
            .wrapping_add(u64::from(col.floors) << 16)
            .wrapping_add(color_kind_key(col.color_kind));
    }
    key
}

fn rgba_key(color: Color) -> u32 {
    let s = color.to_srgba();
    let b = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u32;
    (b(s.red) << 24) | (b(s.green) << 16) | (b(s.blue) << 8) | b(s.alpha)
}

fn footprint_key(tiles: &[(IVec2, u32)]) -> u64 {
    let mut key = tiles.len() as u64;
    for (tile, color) in tiles {
        key = key
            .wrapping_mul(0x9E37_79B9)
            .wrapping_add(tile.x as u64)
            .wrapping_add((tile.y as u64) << 32)
            .wrapping_add(*color as u64);
    }
    key
}

const SITE_CELL: Color = Color::srgb(0.72, 0.58, 0.36);

struct FootprintDraw {
    tile: IVec2,
    fill: Color,
    kind: FootprintTileColorKind,
}

fn collect_footprint_tiles(
    requests: &ConstructionVisualRequests,
    sites: &Query<&SiteFootprint, With<ConstructionSite>>,
) -> Vec<FootprintDraw> {
    let mut tiles: HashMap<IVec2, FootprintDraw> = HashMap::new();
    for foot in sites.iter() {
        for tile in &foot.tiles {
            tiles.entry(*tile).or_insert(FootprintDraw {
                tile: *tile,
                fill: SITE_CELL,
                kind: FootprintTileColorKind::Valid,
            });
        }
    }
    for request in &requests.footprint_tiles {
        tiles.insert(
            request.tile,
            FootprintDraw {
                tile: request.tile,
                fill: footprint_fill(request.color_kind, request.fill_override),
                kind: request.color_kind,
            },
        );
    }
    let mut ordered: Vec<_> = tiles.into_values().collect();
    ordered.sort_by(|a, b| a.tile.y.cmp(&b.tile.y).then(a.tile.x.cmp(&b.tile.x)));
    ordered
}

/// Places rim + fill quads per footprint cell on the world map (Bevy mesh; no egui).
/// Also paints VoxelMassing column stacks (presentation-only height silhouette).
pub fn sync_world_map_footprints(
    mut commands: Commands,
    base: Res<State<BaseState>>,
    requests: Res<ConstructionVisualRequests>,
    sites: Query<&SiteFootprint, With<ConstructionSite>>,
    existing: Query<(Entity, &WorldMapFootprintTile)>,
    massing_existing: Query<(Entity, &WorldMapMassingTile)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut stamp: Local<WorldMapFootprintStamp>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        for (entity, _) in &existing {
            commands.entity(entity).despawn();
        }
        for (entity, _) in &massing_existing {
            commands.entity(entity).despawn();
        }
        stamp.key = 0;
        return;
    }
    let colored = collect_footprint_tiles(&requests, &sites);
    let keyed: Vec<(IVec2, u32)> = colored
        .iter()
        .map(|d| (d.tile, rgba_key(d.fill)))
        .collect();
    let foot_key = footprint_key(&keyed);
    let mass_key = massing_key(&requests.massing_columns);
    let key = foot_key.wrapping_mul(0xC2B2_AE3D).wrapping_add(mass_key);
    let massing_quads: usize = requests
        .massing_columns
        .iter()
        .map(|c| usize::from(c.floors.saturating_sub(1)) * 2)
        .sum();
    // Rim + fill per footprint tile; stacked floors above base (floors>1).
    let expected = colored.len().saturating_mul(2) + massing_quads;
    if key == stamp.key
        && existing.iter().count() + massing_existing.iter().count() == expected
    {
        return;
    }
    for (entity, _) in &existing {
        commands.entity(entity).despawn();
    }
    for (entity, _) in &massing_existing {
        commands.entity(entity).despawn();
    }
    if colored.is_empty() && requests.massing_columns.is_empty() {
        stamp.key = key;
        return;
    }
    let rim_mesh = stamp
        .rim_mesh
        .get_or_insert_with(|| meshes.add(Rectangle::new(1.0, 1.0)))
        .clone();
    let fill_mesh = stamp
        .fill_mesh
        .get_or_insert_with(|| meshes.add(Rectangle::new(0.84, 0.84)))
        .clone();
    for draw in colored {
        let rim = footprint_rim(draw.kind);
        let rim_key = rgba_key(rim);
        let rim_mat = stamp
            .materials
            .entry(rim_key)
            .or_insert_with(|| materials.add(ColorMaterial::from_color(rim)))
            .clone();
        let fill_key = rgba_key(draw.fill);
        let fill_mat = stamp
            .materials
            .entry(fill_key)
            .or_insert_with(|| materials.add(ColorMaterial::from_color(draw.fill)))
            .clone();
        let center = world_map_tile_center(draw.tile.x, draw.tile.y);
        commands.spawn((
            WorldMapFootprintTile { tile: draw.tile },
            Mesh2d(rim_mesh.clone()),
            MeshMaterial2d(rim_mat),
            Transform::from_translation(center + Vec3::new(0.0, 0.0, -0.02)),
        ));
        commands.spawn((
            WorldMapFootprintTile { tile: draw.tile },
            Mesh2d(fill_mesh.clone()),
            MeshMaterial2d(fill_mat),
            Transform::from_translation(center),
        ));
    }
    let floor_off = super::ghost_visual::massing_floor_offset_world();
    let shrink = super::ghost_visual::massing_floor_shrink();
    for col in &requests.massing_columns {
        if col.floors <= 1 {
            continue;
        }
        let base_fill = footprint_fill(col.color_kind, None);
        let rim = massing_rim(col.color_kind);
        let rim_key = rgba_key(rim);
        let rim_mat = stamp
            .materials
            .entry(rim_key)
            .or_insert_with(|| materials.add(ColorMaterial::from_color(rim)))
            .clone();
        let fill_key = rgba_key(base_fill);
        let fill_mat = stamp
            .materials
            .entry(fill_key)
            .or_insert_with(|| materials.add(ColorMaterial::from_color(base_fill)))
            .clone();
        let center = world_map_tile_center(col.tile.x, col.tile.y);
        for floor in 1..col.floors {
            let t = f32::from(floor);
            let size = (0.84 - shrink * t).max(0.35);
            let size_q = (size * 100.0).round() as u32;
            let rim_q = ((size + 0.12) * 100.0).round() as u32;
            let stack_mesh = stamp
                .stack_meshes
                .entry(size_q)
                .or_insert_with(|| meshes.add(Rectangle::new(size, size)))
                .clone();
            let stack_rim = stamp
                .stack_meshes
                .entry(rim_q)
                .or_insert_with(|| meshes.add(Rectangle::new(size + 0.12, size + 0.12)))
                .clone();
            let y = center.y + floor_off * t;
            let z = center.z + 0.04 * t;
            commands.spawn((
                WorldMapMassingTile {
                    tile: col.tile,
                    floor,
                },
                Mesh2d(stack_rim),
                MeshMaterial2d(rim_mat.clone()),
                Transform::from_translation(Vec3::new(center.x, y, z - 0.01)),
            ));
            commands.spawn((
                WorldMapMassingTile {
                    tile: col.tile,
                    floor,
                },
                Mesh2d(stack_mesh),
                MeshMaterial2d(fill_mat.clone()),
                Transform::from_translation(Vec3::new(center.x, y, z)),
            ));
        }
    }
    stamp.key = key;
}

#[cfg(test)]
mod tests {
    use super::world_map_tile_center;
    use bevy::prelude::Vec3;

    #[test]
    fn wmap_tile_center_is_the_cell_and_ignores_zoom() {
        let near = world_map_tile_center(4, 9);
        let far = world_map_tile_center(4, 9);
        assert_eq!(near, far);
        assert_eq!(near, bevy::prelude::Vec3::new(4.5, 9.5, 1.0));
        let next = world_map_tile_center(5, 9);
        assert!((next.x - near.x - 1.0).abs() < 1e-6);
        assert!((next.y - near.y).abs() < 1e-6);
    }

    #[test]
    fn cityfix_out_of_map_power_sample_is_rejected() {
        use super::{path_commit_leaves_generated_map, path_samples_leave_generated_map};
        let inside = [
            Vec3::new(3.2, 0.0, 4.1),
            Vec3::new(10.0, 0.0, 10.0),
        ];
        assert!(!path_samples_leave_generated_map(&inside, 64, 64));
        assert!(!path_commit_leaves_generated_map(
            &inside,
            std::iter::empty::<Vec3>(),
            64,
            64
        ));
        assert!(path_commit_leaves_generated_map(
            &[Vec3::new(-0.2, 0.0, 3.0)],
            std::iter::empty::<Vec3>(),
            64,
            64
        ));
        assert!(path_commit_leaves_generated_map(
            &[Vec3::new(64.0, 0.0, 1.0)],
            std::iter::empty::<Vec3>(),
            64,
            64
        ));
        assert!(path_commit_leaves_generated_map(
            &[Vec3::new(2.0, 0.0, 2.0)],
            [Vec3::new(2.0, 0.0, 2.0), Vec3::new(80.0, 0.0, 2.0)],
            64,
            64
        ));
        assert!(path_samples_leave_generated_map(
            &[Vec3::new(1.0, 0.0, 1.0)],
            0,
            64
        ));
        let src = include_str!("power_lines/input.rs");
        let gate = src
            .find("power_path_leaves_map")
            .expect("bounds gate");
        let commit = src
            .rfind("commit_power_line_to_utility_graph")
            .expect("commit call");
        assert!(gate < commit);
    }

    #[test]
    fn egr_footprint_color_kind_is_not_one_shared_tint() {
        use super::super::visual_authority::FootprintTileColorKind;
        let valid = super::footprint_fill(FootprintTileColorKind::Valid, None);
        let invalid = super::footprint_fill(FootprintTileColorKind::Invalid, None);
        assert_ne!(super::rgba_key(valid), super::rgba_key(invalid));
    }
}
