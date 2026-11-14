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

/// Screen/world tile size for forward+-style light bins (LGT-02).
pub const FIRE_LIGHT_SCREEN_TILE_SIZE: f32 = 64.0;
/// Max pooled lights kept per screen tile before merge (LGT-02).
pub const FIRE_LIGHT_PER_SCREEN_TILE_CAP: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FireLightScreenTile {
    pub tx: i32,
    pub ty: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FireLightTileCapStats {
    pub input_count: usize,
    pub output_count: usize,
    pub max_per_tile: usize,
}

#[must_use]
pub fn screen_tile_for_emission(position: Vec3, tile_size: f32) -> FireLightScreenTile {
    let tile_size = tile_size.max(1.0);
    FireLightScreenTile {
        tx: (position.x / tile_size).floor() as i32,
        ty: (position.z / tile_size).floor() as i32,
    }
}

/// Bin a flat emission list into screen tiles and keep the top `cap` per tile by priority then luminosity.
#[must_use]
pub fn cap_fire_light_emissions_per_screen_tile(
    emissions: &[FireLightEmission],
    tile_size: f32,
    cap: usize,
) -> (Vec<FireLightEmission>, FireLightTileCapStats) {
    if cap == 0 || emissions.is_empty() {
        return (
            Vec::new(),
            FireLightTileCapStats {
                input_count: emissions.len(),
                output_count: 0,
                max_per_tile: 0,
            },
        );
    }
    use std::collections::HashMap;
    let mut bins: HashMap<FireLightScreenTile, Vec<FireLightEmission>> = HashMap::new();
    for e in emissions {
        bins.entry(screen_tile_for_emission(e.position, tile_size))
            .or_default()
            .push(*e);
    }
    let mut out = Vec::with_capacity(emissions.len().min(bins.len() * cap));
    let mut max_per_tile = 0_usize;
    for mut rows in bins.into_values() {
        rows.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    b.luminosity
                        .partial_cmp(&a.luminosity)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        rows.truncate(cap);
        max_per_tile = max_per_tile.max(rows.len());
        out.extend(rows);
    }
    let output_count = out.len();
    (
        out,
        FireLightTileCapStats {
            input_count: emissions.len(),
            output_count,
            max_per_tile,
        },
    )
}

/// LGT-02 — tile cap then world merge (consumer replaces bare [`build_fire_light_clusters`] when hot).
#[must_use]
pub fn build_fire_light_clusters_screen_capped(
    emissions: &[FireLightEmission],
    tile_size: f32,
    per_tile_cap: usize,
) -> (Vec<FireLightCluster>, FireLightTileCapStats) {
    let (capped, stats) = cap_fire_light_emissions_per_screen_tile(emissions, tile_size, per_tile_cap);
    (build_fire_light_clusters(&capped), stats)
}

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

    fn sample_at(x: f32, z: f32, priority: f32) -> FireLightEmission {
        FireLightEmission {
            position: Vec3::new(x, 0.0, z),
            heat: 0.5,
            luminosity: 1.0,
            smoke_density: 0.0,
            radius: 40.0,
            priority,
            fire_type: FireLightType::Forest,
        }
    }

    #[test]
    fn screen_tile_cap_limits_lights_per_bin() {
        let emissions: Vec<_> = (0..12)
            .map(|i| sample_at(i as f32 * 2.0, 0.0, i as f32))
            .collect();
        let (capped, stats) = cap_fire_light_emissions_per_screen_tile(
            &emissions,
            FIRE_LIGHT_SCREEN_TILE_SIZE,
            FIRE_LIGHT_PER_SCREEN_TILE_CAP,
        );
        assert!(stats.max_per_tile <= FIRE_LIGHT_PER_SCREEN_TILE_CAP);
        let mut per_tile: std::collections::HashMap<FireLightScreenTile, usize> =
            std::collections::HashMap::new();
        for e in &capped {
            *per_tile
                .entry(screen_tile_for_emission(
                    e.position,
                    FIRE_LIGHT_SCREEN_TILE_SIZE,
                ))
                .or_default() += 1;
        }
        assert!(per_tile.values().all(|&n| n <= FIRE_LIGHT_PER_SCREEN_TILE_CAP));
    }

    #[test]
    fn capped_cluster_path_preserves_tile_stats() {
        let emissions: Vec<_> = (0..8).map(|i| sample_at(10.0, i as f32 * 3.0, i as f32)).collect();
        let (clusters, stats) = build_fire_light_clusters_screen_capped(
            &emissions,
            FIRE_LIGHT_SCREEN_TILE_SIZE,
            FIRE_LIGHT_PER_SCREEN_TILE_CAP,
        );
        assert!(stats.max_per_tile <= FIRE_LIGHT_PER_SCREEN_TILE_CAP);
        assert!(!clusters.is_empty() || emissions.is_empty());
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
