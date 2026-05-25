//! Spatial industrial districts — load clusters on transformer hosts (I4-04).

use bevy::prelude::*;

use crate::entities::production::power::components::{
    ElectricalComponent, ElectricalGrid, TransformerComponent,
};
use crate::entities::structure::components::Building;
use crate::strategic::{BuildSiteTile, PlannedSite, StrategicRasterConfig};
use crate::systems::sim_control::SimControlState;
use crate::terrain::ChunkCellKey;

/// Chunk anchor for an operational facility (logistics + district grouping).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IndustrialDistrictAnchor(pub ChunkCellKey);

#[must_use]
pub fn chunk_key_from_site_tile(origin: BuildSiteTile, cells: UVec2) -> ChunkCellKey {
    let sx = cells.x.max(1);
    let sy = cells.y.max(1);
    let tx = origin.x.max(0) as u32;
    let tz = origin.z.max(0) as u32;
    let cx = (tx / sx) as i32;
    let cz = (tz / sy) as i32;
    let lx = tx % sx;
    let lz = tz % sy;
    let cell = lz * sx + lx;
    ChunkCellKey::new(IVec2::new(cx, cz), cell)
}

/// Per-transformer-host snapshot after grid membership rebuild.
#[derive(Clone, Debug)]
pub struct TransformerHostLoad {
    pub host: Entity,
    pub member_count: usize,
    pub total_load: f32,
    pub capacity: f32,
    pub load_ratio: f32,
    /// Mean distance (metres) from host to members — low = clustered district.
    pub mean_member_distance_m: f32,
}

/// Latest district measurement (proof + witness).
#[derive(Resource, Clone, Debug, Default)]
pub struct IndustrialDistrictSnapshot {
    pub hosts: Vec<TransformerHostLoad>,
}

impl IndustrialDistrictSnapshot {
    #[must_use]
    pub fn dominant_host_load_ratio(&self) -> f32 {
        self.hosts
            .iter()
            .map(|h| h.load_ratio)
            .fold(0.0f32, f32::max)
    }

    #[must_use]
    pub fn clustered_host_count(&self) -> usize {
        self.hosts
            .iter()
            .filter(|h| h.member_count >= 2 && h.mean_member_distance_m < 24.0)
            .count()
    }
}

pub fn attach_district_anchors_system(
    cfg: Option<Res<StrategicRasterConfig>>,
    mut commands: Commands,
    q: Query<(Entity, &PlannedSite), Without<IndustrialDistrictAnchor>>,
) {
    let cells = cfg
        .map(|c| c.cells_per_chunk)
        .unwrap_or(UVec2::new(32, 32));
    for (entity, planned) in &q {
        commands
            .entity(entity)
            .insert(IndustrialDistrictAnchor(chunk_key_from_site_tile(
                planned.origin, cells,
            )));
    }
}

pub fn measure_spatial_industrial_district_system(
    mut snapshot: ResMut<IndustrialDistrictSnapshot>,
    hosts: Query<
        (Entity, &Transform, &ElectricalGrid),
        With<TransformerComponent>,
    >,
    members: Query<(&Transform,), (With<Building>, With<ElectricalComponent>)>,
) {
    snapshot.hosts.clear();
    for (host_entity, host_tf, grid) in &hosts {
        if grid.members.is_empty() {
            continue;
        }
        let host_pos = host_tf.translation;
        let mut dist_sum = 0.0f32;
        let n = grid.members.len() as f32;
        for member in grid.members.iter() {
            if let Ok((m_tf,)) = members.get(*member) {
                dist_sum += host_pos.distance(m_tf.translation);
            }
        }
        let mean_dist = if n > 0.0 { dist_sum / n } else { 0.0 };
        let cap = grid.total_capacity.max(f32::EPSILON);
        snapshot.hosts.push(TransformerHostLoad {
            host: host_entity,
            member_count: grid.members.len(),
            total_load: grid.total_load,
            capacity: cap,
            load_ratio: grid.total_load / cap,
            mean_member_distance_m: mean_dist,
        });
    }
}

fn sim_running(ctrl: Res<SimControlState>) -> bool {
    ctrl.should_tick()
}

pub struct SpatialDistrictPlugin;

impl Plugin for SpatialDistrictPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IndustrialDistrictSnapshot>()
            .init_resource::<StrategicRasterConfig>()
            .add_systems(
                Update,
                (
                    attach_district_anchors_system,
                    measure_spatial_industrial_district_system,
                )
                    .chain()
                    .run_if(sim_running),
            );
    }
}

/// Run grid topology + return dominant load ratio for a smelter layout (tests / diagnostics).
#[cfg(test)]
pub fn measure_layout_load_ratio(
    smelter_offsets_m: &[(f32, f32)],
    host_capacity: f32,
    connection_radius_m: f32,
) -> f32 {
    use crate::entities::production::power::grid_topology::{
        rebuild_electrical_grid_topology,
        recalculate_grid_totals_from_members, GridConnectionRadiusSq,
    };
    use crate::entities::types::s_flagz::BuildingType;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(GridConnectionRadiusSq(connection_radius_m * connection_radius_m));
    app.init_resource::<IndustrialDistrictSnapshot>();

    app.world_mut().spawn((
        Transform::from_translation(Vec3::ZERO),
        GlobalTransform::default(),
        TransformerComponent {
            input_voltage: 138_000.0,
            output_voltage: 13_800.0,
        },
        ElectricalGrid::default(),
        ElectricalComponent {
            base_load: 0.05,
            current_load: 0.05,
            max_transfer: host_capacity,
            capacity: host_capacity,
        },
        Building {
            building_type: BuildingType::Generic,
        },
    ));

    for &(x, z) in smelter_offsets_m {
        app.world_mut().spawn((
            Transform::from_translation(Vec3::new(x, 0.0, z)),
            GlobalTransform::default(),
            Building {
                building_type: BuildingType::Generic,
            },
            ElectricalComponent {
                base_load: 2.0,
                current_load: 2.0,
                max_transfer: 2.0,
                capacity: 0.0,
            },
        ));
    }

    app.add_systems(
        Update,
        (
            rebuild_electrical_grid_topology,
            recalculate_grid_totals_from_members,
            measure_spatial_industrial_district_system,
        )
            .chain(),
    );
    app.update();

    app.world()
        .resource::<IndustrialDistrictSnapshot>()
        .dominant_host_load_ratio()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clustered_smelters_raise_load_ratio_more_than_spread() {
        let clustered = measure_layout_load_ratio(&[(4.0, 0.0), (8.0, 0.0), (12.0, 0.0), (16.0, 0.0)], 4.0, 32.0);
        let spread = measure_layout_load_ratio(
            &[(4.0, 0.0), (200.0, 0.0), (0.0, 200.0), (200.0, 200.0)],
            4.0,
            32.0,
        );
        assert!(
            clustered > spread * 1.5,
            "clustered load ratio {clustered} should exceed spread {spread}"
        );
        assert!(clustered > 1.0, "clustered layout should overload host");
    }

    #[test]
    fn chunk_key_from_adjacent_tiles_differ() {
        let cells = UVec2::new(32, 32);
        let a = chunk_key_from_site_tile(BuildSiteTile { x: 0, z: 0 }, cells);
        let b = chunk_key_from_site_tile(BuildSiteTile { x: 40, z: 0 }, cells);
        assert_ne!(a, b);
    }
}
