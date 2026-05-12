//! Render-facing **lighting helpers** (clustering, etc.). Not simulation ECS truth.

pub mod fire_light_cluster;

pub use fire_light_cluster::{
    build_fire_light_clusters, merge_emission_into_cluster, FireLightCluster, FireLightEmission,
    FireLightType, CLUSTER_MERGE_RADIUS,
};
