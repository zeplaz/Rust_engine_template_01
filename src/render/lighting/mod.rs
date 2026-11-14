//! Render-facing **lighting helpers** (clustering, etc.). Not simulation ECS truth.

pub mod fire_light_cluster;

pub use fire_light_cluster::{
    build_fire_light_clusters, build_fire_light_clusters_screen_capped,
    cap_fire_light_emissions_per_screen_tile, merge_emission_into_cluster, FireLightCluster,
    FireLightEmission, FireLightScreenTile, FireLightTileCapStats,
    FIRE_LIGHT_PER_SCREEN_TILE_CAP, FIRE_LIGHT_SCREEN_TILE_SIZE,
    FireLightType, CLUSTER_MERGE_RADIUS,
};
