//! Ephemeral **fire light clustering** for pooled local lights (`base_fire2_smoke.md`).
//!
//! [`FireLightEmission`] here is a **render-side snapshot** (position + scalars), not the ECS
//! [`crate::systems::fire::FireLightEmission`] on chunks. Build [`FireLightCluster`]s each frame;
//! do not spawn cluster entities.

use bevy::prelude::*;

/// Class of fire for tint / future smoke–light coupling (`base_fire2_smoke.md`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FireLightType {
    #[default]
    Forest,
    Fuel,
    Chemical,
    Electrical,
    Structure,
}

/// One **sample** before spatial merge (render extraction buffer row).
#[derive(Clone, Copy, Debug)]
pub struct FireLightEmission {
    pub position: Vec3,
    /// Thermal proxy `[0, 1]` from sim heat.
    pub heat: f32,
    /// Visible brightness proxy (e.g. sim `current_intensity`).
    pub luminosity: f32,
    /// Smoke coupling hint `[0, 1]`.
    pub smoke_density: f32,
    pub radius: f32,
    pub priority: f32,
    pub fire_type: FireLightType,
}

/// Merged **region** after greedy distance merge (transient; not an entity).
#[derive(Clone, Debug)]
pub struct FireLightCluster {
    pub centroid: Vec3,
    pub radius: f32,
    pub total_heat: f32,
    pub total_luminosity: f32,
    pub smoke_density: f32,
    pub dominant_type: FireLightType,
    pub member_count: usize,
}

/// World-space merge distance (tile-ish scale; tune with map cell size).
pub const CLUSTER_MERGE_RADIUS: f32 = 180.0;

pub fn build_fire_light_clusters(emissions: &[FireLightEmission]) -> Vec<FireLightCluster> {
    let mut clusters: Vec<FireLightCluster> = Vec::new();
    let r2 = CLUSTER_MERGE_RADIUS * CLUSTER_MERGE_RADIUS;

    'outer: for emission in emissions {
        for cluster in &mut clusters {
            let d2 = cluster.centroid.distance_squared(emission.position);
            if d2 < r2 {
                merge_emission_into_cluster(cluster, emission);
                continue 'outer;
            }
        }
        clusters.push(FireLightCluster {
            centroid: emission.position,
            radius: emission.radius,
            total_heat: emission.heat,
            total_luminosity: emission.luminosity,
            smoke_density: emission.smoke_density,
            dominant_type: emission.fire_type,
            member_count: 1,
        });
    }

    clusters
}

pub fn merge_emission_into_cluster(cluster: &mut FireLightCluster, emission: &FireLightEmission) {
    let count = cluster.member_count as f32;
    cluster.centroid = (cluster.centroid * count + emission.position) / (count + 1.0);
    cluster.total_heat += emission.heat;
    cluster.total_luminosity += emission.luminosity;
    cluster.smoke_density = cluster.smoke_density.max(emission.smoke_density);
    cluster.radius = cluster.radius.max(emission.radius);
    cluster.member_count += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_combines_nearby_into_one_cluster() {
        let a = FireLightEmission {
            position: Vec3::ZERO,
            heat: 0.5,
            luminosity: 1.0,
            smoke_density: 0.2,
            radius: 100.0,
            priority: 1.0,
            fire_type: FireLightType::Forest,
        };
        let b = FireLightEmission {
            position: Vec3::new(50.0, 0.0, 0.0),
            heat: 0.5,
            luminosity: 1.0,
            smoke_density: 0.4,
            radius: 120.0,
            priority: 1.0,
            fire_type: FireLightType::Forest,
        };
        let c = build_fire_light_clusters(&[a, b]);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].member_count, 2);
        assert!((c[0].total_heat - 1.0).abs() < 1e-5);
    }

    #[test]
    fn far_emissions_stay_separate() {
        let a = FireLightEmission {
            position: Vec3::ZERO,
            heat: 1.0,
            luminosity: 1.0,
            smoke_density: 0.0,
            radius: 50.0,
            priority: 1.0,
            fire_type: FireLightType::Forest,
        };
        let b = FireLightEmission {
            position: Vec3::new(CLUSTER_MERGE_RADIUS * 2.0, 0.0, 0.0),
            heat: 1.0,
            luminosity: 1.0,
            smoke_density: 0.0,
            radius: 50.0,
            priority: 1.0,
            fire_type: FireLightType::Forest,
        };
        assert_eq!(build_fire_light_clusters(&[a, b]).len(), 2);
    }
}
