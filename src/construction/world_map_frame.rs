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

#[derive(Resource, Default)]
pub(crate) struct WorldMapFootprintStamp {
    key: u64,
    mesh: Option<Handle<Mesh>>,
    materials: HashMap<u32, Handle<ColorMaterial>>,
}

fn color32_bevy(c: bevy_egui::egui::Color32) -> Color {
    Color::srgba_u8(c.r(), c.g(), c.b(), c.a())
}

/// Validity color for one cell. The quad stays on [`world_map_tile_center`]; only the material changes.
pub(crate) fn footprint_fill(kind: FootprintTileColorKind, fill: Option<bevy_egui::egui::Color32>) -> Color {
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

fn collect_footprint_tiles(
    requests: &ConstructionVisualRequests,
    sites: &Query<&SiteFootprint, With<ConstructionSite>>,
) -> Vec<(IVec2, Color)> {
    let mut tiles: HashMap<IVec2, Color> = HashMap::new();
    for foot in sites.iter() {
        for tile in &foot.tiles {
            tiles.entry(*tile).or_insert(SITE_CELL);
        }
    }
    for request in &requests.footprint_tiles {
        tiles.insert(
            request.tile,
            footprint_fill(request.color_kind, request.fill_override),
        );
    }
    let mut ordered: Vec<_> = tiles.into_iter().collect();
    ordered.sort_by(|a, b| a.0.y.cmp(&b.0.y).then(a.0.x.cmp(&b.0.x)));
    ordered
}

/// Places one quad per footprint cell on the world map. Editor keeps egui.
pub fn sync_world_map_footprints(
    mut commands: Commands,
    base: Res<State<BaseState>>,
    requests: Res<ConstructionVisualRequests>,
    sites: Query<&SiteFootprint, With<ConstructionSite>>,
    existing: Query<(Entity, &WorldMapFootprintTile)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut stamp: Local<WorldMapFootprintStamp>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        for (entity, _) in &existing {
            commands.entity(entity).despawn();
        }
        stamp.key = 0;
        return;
    }
    let colored = collect_footprint_tiles(&requests, &sites);
    let keyed: Vec<(IVec2, u32)> = colored
        .iter()
        .map(|(tile, color)| (*tile, rgba_key(*color)))
        .collect();
    let key = footprint_key(&keyed);
    if key == stamp.key && existing.iter().count() == colored.len() {
        return;
    }
    for (entity, _) in &existing {
        commands.entity(entity).despawn();
    }
    if colored.is_empty() {
        stamp.key = key;
        return;
    }
    let mesh = stamp
        .mesh
        .get_or_insert_with(|| meshes.add(Rectangle::new(0.92, 0.92)))
        .clone();
    for (tile, color) in colored {
        let rgba = rgba_key(color);
        let material = stamp
            .materials
            .entry(rgba)
            .or_insert_with(|| materials.add(ColorMaterial::from_color(color)))
            .clone();
        let center = world_map_tile_center(tile.x, tile.y);
        commands.spawn((
            WorldMapFootprintTile { tile },
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material),
            Transform::from_translation(center),
        ));
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
